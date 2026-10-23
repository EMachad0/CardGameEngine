## Summary
In XMage, casting-time targets and resolution-time choices use the same `Target` classes (`TargetCard`, `TargetPermanent`, `TargetCardInLibrary`, etc.). A boolean `notTarget` flag (`withNotTarget(true)`) is the only thing that marks an object as a choice rather than a target. That flag changes the rules wording, turns off hexproof/shroud/protection checks and stops TARGET events from firing. The *timing* of the question depends on where the Target object is placed. If it is added to the ability with `ability.addTarget(...)`, the player answers while casting. If it is built inside `Effect.apply`, the player answers during resolution. A 2025 refactor (PR #13651) added `OneShotNonTargetEffect`, a wrapper that moves a declared non-target choice to resolution time, which is close to your "one declaration, different answer time" model. Effects in one ability share chosen objects through `TargetPointer`s. The default is `FirstTargetPointer`, which reads `ability.getTargets().get(0)`. Things chosen during resolution are passed on by creating `FixedTarget(id, zcc)` pointers on newly built effects, or through `setValue`/`getValue`. For "look at the top N", XMage takes a snapshot of library card IDs into a temporary `Cards` set. The cards stay in the library zone while the player picks, and then two `PutCards` enum values send "picked" and "rest" to their destinations. Private looking goes through a per-player `LookedAt` map in `GameState`. That map only drives the UI window and is cleared frequently. Within what I read, XMage keeps no lasting record that a player has seen a card.

## Findings

### Q1. Shared Target classes for targets and choices; `notTarget`

1. **One class hierarchy for both uses.** The same `Target` subclasses work in both places:
   - **Casting time:** `RaiseDead` does `this.getSpellAbility().addTarget(new TargetCardInYourGraveyard(StaticFilters.FILTER_CARD_CREATURE_YOUR_GRAVEYARD));`. [RaiseDead.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/r/RaiseDead.java)
   - **Resolution time, same class family:** `WildfireDevilsEffect.apply` does
     ```java
     TargetCardInGraveyard targetCard = new TargetCardInGraveyard(StaticFilters.FILTER_CARD_INSTANT_OR_SORCERY);
     targetCard.withNotTarget(true);
     if (!randomPlayer.choose(Outcome.Discard, randomPlayer.getGraveyard(), targetCard, source, game)) {
     ```
     [WildfireDevils.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/w/WildfireDevils.java)
   - **Library search:** `SearchLibraryPutInHandEffect` takes a `TargetCardInLibrary` in its constructor and asks for it in `apply` with `controller.searchLibrary(target, source, game)`. [SearchLibraryPutInHandEffect.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/effects/common/search/SearchLibraryPutInHandEffect.java)
   - **The `Player` entry points** all take a `Target`:
     ```java
     boolean choose(Outcome outcome, Target target, Ability source, Game game);
     boolean choose(Outcome outcome, Cards cards, TargetCard target, Ability source, Game game);
     boolean chooseTarget(Outcome outcome, Target target, Ability source, Game game);
     boolean chooseTarget(Outcome outcome, Cards cards, TargetCard target, Ability source, Game game);
     ```
     [Player.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/players/Player.java)

2. **The engine's own definition of `notTarget`.** The Javadoc on `Target.withNotTarget` says:
   > "Mark it as non target (e.g. card's rules do not contain a "target" word) … Non targeted abilities are unaffected by protection/hexproof and other target related effects. Non targeted spells can't be fizzled on resolve with invalid targets. Non targeted spells chooses targets on resolve, targeted spells chooses targets on activate. All costs must be non targeted."

   [Target.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/Target.java)

   `TargetImpl` stores it as `protected boolean notTarget = false; // is the target handled as targeted spell/ability (notTarget = true is used for not targeted effects like e.g. sacrifice)`. `withNotTarget(boolean)` is the current setter. Older code used `target.setNotTarget(true)`, for example [MetamorphicAlteration @ deb675a](https://github.com/magefree/mage/blob/deb675a74a5c93d3a86ec28599b07e2096eadb68/Mage.Sets/src/mage/cards/m/MetamorphicAlteration.java). Some constructors also take the flag directly, e.g. `TargetPermanent(int min, int max, FilterPermanent filter, boolean notTarget)`. [TargetImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/TargetImpl.java), [TargetPermanent.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/TargetPermanent.java)

3. **What the flag actually changes in code:**
   - **Hexproof/shroud/protection.** `TargetPermanent.canTarget` only runs `canBeTargetedBy` when the target is real: `if (!isNotTarget()) { if (!permanent.canBeTargetedBy(...)) return false; } return filter.match(...)`. `Target.keepValidPossibleTargets` also returns `true` straight away `if (source == null || … || isNotTarget())`, so protection filtering is skipped. [TargetPermanent.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/TargetPermanent.java), [Target.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/Target.java)
   - **TARGET events.** `addTarget(id, source, game)` calls `addTarget(id, source, game, notTarget)`, so `skipEvent = notTarget` and no TargetEvent is fired. `isLegal()` has the guard `if (!notTarget && game.replaceEvent(new TargetEvent(targetId, source)))`. [TargetImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/TargetImpl.java)
   - **Rules text.** `getDescription()` only prepends `"target "` when `!isNotTarget()`. For a single non-target choice it adds an article instead ("a land you control"). `AbilityImpl` logs "targeting" for real targets and "choosing" for non-targets. [TargetImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/TargetImpl.java), [AbilityImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/AbilityImpl.java)
   - **Re-check on resolution.** `TargetImpl.isLegal` (rule 608.2b) still checks the zone-change counter and calls `stillLegalTarget → canTarget` for non-targets. Only the TargetEvent check is skipped. `Spell.resolve` fizzles through `Targets.stillLegal` ("countered if all its targets … are now illegal"). That check looks at every Target in the ability without looking at `notTarget`. [Targets.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/Targets.java), [Spell.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/stack/Spell.java)
     - **UNVERIFIED:** whether any non-target Target is still added to an ability's `Targets` after the refactor below. If one is, it could in principle cause a fizzle, which would contradict the Javadoc.

4. **Timing is set by placement, and the project moved non-targets to resolution.** `AbilityImpl.activate` calls `getTargets().chooseTargets(...)` for every Target added to the ability. Before 2025, `notTarget` Targets added this way were therefore answered while casting, which is wrong.
   - PR #13651 ("remove notTarget targets from abilities", merged 2025-06-01) added `OneShotNonTargetEffect`. The commit message says: "Fixes the target timing of spells/abilities, as non-targets should be chosen on resolution." [PR #13651](https://github.com/magefree/mage/pull/13651)
   - PR #13647 added a verify test that checks the word "target" in rules text against real target declarations. [PR #13647](https://github.com/magefree/mage/pull/13647)
   - How `OneShotNonTargetEffect.apply` works:
     ```java
     Target target = notTarget.copy();
     if (source.getTargetAdjuster() != null || !source.getTargets().isEmpty()){
         throw new IllegalStateException("source ability already has target but is using OneShotNonTargetEffect");
     }
     source.addTarget(target);
     ...
     if (source.getTargets().choose(outcome, source.getControllerId(), source.getId(), source, game)) {
         result = effect.apply(game, source);
     }
     source.getTargets().clear();
     ```
     Its constructor forces `this.notTarget.withNotTarget(true)`. [OneShotNonTargetEffect.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/effects/OneShotNonTargetEffect.java)
   - Card usage, Wormfang Newt:
     ```java
     new OneShotNonTargetEffect(new ExileTargetForSourceEffect().setText("exile a land you control"),
             new TargetPermanent(StaticFilters.FILTER_CONTROLLED_PERMANENT_A_LAND))
     ```
     [WormfangNewt.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/w/WormfangNewt.java). Yorion uses `new TargetPermanent(0, Integer.MAX_VALUE, filter, true)` inside the same wrapper. [YorionSkyNomad.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/y/YorionSkyNomad.java)
   - This is the closest XMage gets to your model: one Target declaration, and the effect that consumes it (a `*TargetEffect`), stay the same. Only the moment the question is asked changes.
   - The PR has a caveat: the guard means it cannot be combined with real targets on the same ability. The PR also mentions Lodestone Bauble as a remaining exception.

### Q2. Several effects acting on the same object; passing resolution-time choices

5. **Default sharing goes through `FirstTargetPointer`.** `EffectImpl` has `private TargetPointer targetPointer = new FirstTargetPointer();`. Its comment says "warning, do not change it directly, use setTargetPointer instead". [EffectImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/effects/EffectImpl.java)
   - `FirstTargetPointer extends NthTargetPointer` with `super(1)`. `NthTargetPointer.getFirst` reads `source.getTargets().get(this.targetIndex).getFirstTarget()` and returns `null` if the zone-change counter is outdated (`isOutdatedTarget`). [FirstTargetPointer.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/targetpointer/FirstTargetPointer.java), [NthTargetPointer.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/targetpointer/NthTargetPointer.java)
   - Concrete card, Run Amok: both effects resolve against Target #0 with no extra wiring.
     ```java
     this.getSpellAbility().addEffect(new BoostTargetEffect(3, 3, Duration.EndOfTurn)...);
     this.getSpellAbility().addEffect(new GainAbilityTargetEffect(TrampleAbility.getInstance(), Duration.EndOfTurn)...);
     this.getSpellAbility().addTarget(new TargetAttackingCreature());
     ```
     [RunAmok.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/r/RunAmok.java)

6. **`FixedTarget` pins one object at one zone-change counter.** For example `new FixedTarget(Card card, Game game)` sets `zoneChangeCounter = card.getZoneChangeCounter(game)`. `getFirst` then returns `null` if the card has since changed zones. A `FixedTargets` class exists for multiple objects (fetched, but its body was not reviewed). [FixedTarget.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/targetpointer/FixedTarget.java), [FixedTargets.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/targetpointer/FixedTargets.java)
   - The `TargetPointer` Javadoc says `getFirstTargetPermanentOrLKI` "is only used if the target pointer is used to transfer information about a related permanent (often from triggered abilities)". [TargetPointer.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/targetpointer/TargetPointer.java)

7. **Passing a resolution-time choice to a later effect.** I found three patterns in source:
   - **(a) Build the follow-up effect inside `apply` and point it at the chosen object.** Peer Pressure picks a creature type during resolution, then for each matching permanent:
     ```java
     ContinuousEffect effect = new GainControlTargetEffect(Duration.EndOfGame);
     effect.setTargetPointer(new FixedTarget(permanent, game));
     game.addEffect(effect, source);
     ```
     [PeerPressure.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/p/PeerPressure.java)
   - **(b) Rewrite the pointers of all the ability's effects at once.** `Effects.setTargetPointer(tp)` copies the pointer into every effect, and `Effects.setValue(key, value)` does the same for values. Triggers do this in `checkTrigger`: `getAllEffects().setValue("damagedPlayer", event.getPlayerId()); if (setTargetPointer) getAllEffects().setTargetPointer(new FixedTarget(event.getPlayerId()));`. That runs at trigger time, not resolution time. [Effects.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/effects/Effects.java), [DealsCombatDamageToAPlayerTriggeredAbility.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/common/DealsCombatDamageToAPlayerTriggeredAbility.java)
   - **(c) Store the choice in game state.** The old Metamorphic Alteration did `game.getState().setValue(source.getSourceId() + INFO_KEY, chosenPermanent.copy())` and a static effect read it back later. [MetamorphicAlteration @ deb675a](https://github.com/magefree/mage/blob/deb675a74a5c93d3a86ec28599b07e2096eadb68/Mage.Sets/src/mage/cards/m/MetamorphicAlteration.java)
   - **Limit of `OneShotNonTargetEffect`:** it calls `source.getTargets().clear()` after its wrapped effect runs, so sibling effects that come later in the same ability cannot see that choice through `FirstTargetPointer`.
   - **UNVERIFIED:** a card whose first effect, during resolution, calls `setTargetPointer` on a *sibling* effect in `source.getEffects()`. I did not find one. The usual practice seems to be one custom `OneShotEffect` that does the whole sequence, as in Peer Pressure, Wildfire Devils and Wild Swing.

### Q3. "Look at top N, pick some, the rest go elsewhere"

8. **The cards stay in the library while the player chooses.** `LookLibraryControllerEffect.apply`:
   ```java
   Cards cards = new CardsImpl(controller.getLibrary().getTopCards(game, numberOfCards.calculate(game, source, this)));
   if (revealCards) { controller.revealCards(source, cards, game); }
   else { controller.lookAtCards(source, null, cards, game); }
   boolean result = actionWithLookedCards(game, source, controller, cards);
   ```
   `Library.getTopCards` only walks `library.iterator()` into a `LinkedHashSet<Card>` and removes nothing. The neighbouring `getFromTop` doc says "The card is still in the library, until/unless some zone-handling code moves it". So `Cards` is a set of IDs, not a zone. The effect also temporarily turns off `setTopCardRevealed` (for Future-Sight-style effects) and restores it afterwards. [LookLibraryControllerEffect.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/effects/common/LookLibraryControllerEffect.java), [Library.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/players/Library.java)

9. **Picking and routing.** `LookLibraryAndPickControllerEffect.actionWithLookedCards`:
   ```java
   TargetCard target = new TargetCard((upTo ? 0 : number), number, Zone.LIBRARY, filter);
   if (!player.chooseTarget(putPickedCards.getOutcome(), cards, target, source, game)) {
       return actionWithPickedCards(game, source, player, new CardsImpl(), cards);
   }
   Cards pickedCards = new CardsImpl(target.getTargets());
   if (revealPickedCards) player.revealCards(source, pickedCards, game);
   cards.removeAll(pickedCards);
   return actionWithPickedCards(game, source, player, pickedCards, cards);
   ```
   `actionWithPickedCards` then does `putPickedCards.moveCards(player, pickedCards, …)` followed by `putLookedCards.moveCards(player, otherCards, …)`.
   - The "pick" is a plain `TargetCard` restricted to the `Cards` subset.
   - The "rest" is the set difference.
   - Each part gets its own `PutCards` destination.
   - Extras: `optional` (a "may" prompt), `upTo`, a `filter`, automatic reveal when the picked zone is not public (`revealPickedCards = !putPickedCards.getZone().isPublicZone()`), and `withOtherwiseEffect(...)`.

   [LookLibraryAndPickControllerEffect.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/abilities/effects/common/LookLibraryAndPickControllerEffect.java)

10. **The `PutCards` enum** (Outcome, Zone, text). Values: `HAND, GRAVEYARD, BATTLEFIELD, BATTLEFIELD_TAPPED, BATTLEFIELD_TAPPED_ATTACKING, BATTLEFIELD_TRANSFORMED, EXILED, TOP_OR_BOTTOM, TOP_ANY, BOTTOM_ANY, BOTTOM_RANDOM, SHUFFLE`.
    - `moveCards` maps `BOTTOM_ANY` to `player.putCardsOnBottomOfLibrary(cards, game, source, true)` and `BOTTOM_RANDOM` to the same call with `false` (the flag is "any order").
    - `TOP_OR_BOTTOM` throws for multiple cards.
    - The enum also produces the rules text ("on the bottom of your library" + " in any order").

    [PutCards.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/constants/PutCards.java)

11. **The example cards are one-liners:**
    - Impulse: `new LookLibraryAndPickControllerEffect(4, 1, PutCards.HAND, PutCards.BOTTOM_ANY)`
    - Anticipate: `(3, 1, PutCards.HAND, PutCards.BOTTOM_ANY)`
    - Ancestral Memories: `(7, 2, PutCards.HAND, PutCards.GRAVEYARD)`

    [Impulse.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/i/Impulse.java), [Anticipate.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/a/Anticipate.java), [AncestralMemories.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/a/AncestralMemories.java)

### Q4. Random selection

12. **Random as a flag on `Target`.** `TargetImpl` has `protected boolean atRandom = false;` and the interface has `setRandom(boolean)`. The `Target.setRandom` Javadoc warns: "if you need random choice then call it by target's choose method, not player's choose — see #11933".
    - In the choose loop, `if (isRandom())` it does `UUID possibleTarget = RandomUtil.randomFromCollection(randomPossibleTargets); … this.addTarget(possibleTarget, source, game);`.
    - `getTargetName()` appends `" chosen at random"`.
    - Because the random result lands in the same Target, the ability's later effects read it through `FirstTargetPointer` exactly as they would a player's choice.
    - Example, Goblin Test Pilot:
      ```java
      Target target = new TargetAnyTarget(); target.setRandom(true); ability.addTarget(target);
      ```
      paired with `DamageTargetEffect(2)`.

    [TargetImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/target/TargetImpl.java), [GoblinTestPilot.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/g/GoblinTestPilot.java)

13. **Ad-hoc `RandomUtil` inside custom effects.** `RandomUtil` wraps a single seedable `java.util.Random` (`setSeed`, `nextInt`, `randomFromCollection`).
    - Wild Swing: `target.getTargets().get(RandomUtil.nextInt(target.getTargets().size()))` among 3 targets, then destroy that one.
    - Capricious Efreet: the same idea over two Targets.
    - Wildfire Devils: `players.get(RandomUtil.nextInt(players.size()))`, then `game.informPlayers("The chosen random player is …")`.

    The random result is used directly inside the same `apply` and is not passed to a sibling effect. [RandomUtil.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/util/RandomUtil.java), [WildSwing.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/w/WildSwing.java), [CapriciousEfreet.java](https://github.com/magefree/mage/blob/master/Mage.Sets/src/mage/cards/c/CapriciousEfreet.java)

14. **UNVERIFIED:** a specific card for "a random creature gets +1/+1 and gains X". Based on the patterns above, it would either be (i) a Target with `setRandom(true)` plus `BoostTargetEffect` and `GainAbilityTargetEffect` sharing `FirstTargetPointer`, or (ii) a custom effect that picks with `RandomUtil.randomFromCollection` and then adds two continuous effects, each with `new FixedTarget(permanent, game)` as in Peer Pressure. I could not find a concrete card file to confirm either (GitHub code search and grep.app were not reachable).

### Q5. "Look at" vs "reveal"; record of seen cards

15. **Looking is per-player and short-lived.** `PlayerImpl.lookAtCards(Ability source, String titleSuffix, Cards cards, Game game)` does `game.getState().getLookedAt(this.playerId).add(CardUtil.createObjectRelatedWindowTitle(source, game, titleSuffix), cards); game.fireUpdatePlayersEvent();`.
    - `GameState` keeps `private final Map<UUID, LookedAt> lookedAt` (per player) next to a shared `Revealed revealed` ("revealed cards <Name, <Cards>>, will be reset if all players pass priority").
    - `LookedAt extends HashMap<String, Cards>`, keyed by window title.

    [PlayerImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/players/PlayerImpl.java), [GameState.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/GameState.java), [LookedAt.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/LookedAt.java)

16. **It is not a knowledge record.** `GameImpl.fireUpdatePlayersEvent()` fires the UI update and then immediately calls `getState().clearLookedAt(); getState().clearRevealed();`. The priority prompt clears both as well. So `LookedAt` is a buffer for sending "show this player these cards" to the client, not a memory of who has seen what. [GameImpl.java](https://github.com/magefree/mage/blob/master/Mage/src/main/java/mage/game/GameImpl.java)
    - **UNVERIFIED:** that no other "known to player" mechanism exists anywhere, for example for face-down cards or the `GameView` per-player projection. I did not review those.

## Sources
**Kept** (primary source on the `master` branch, or merged PRs):
- `Target.java`, `TargetImpl.java`, `TargetPermanent.java`, `Targets.java`, `Spell.java`, `AbilityImpl.java`: the `notTarget` semantics, legality checks and choose timing.
- `OneShotNonTargetEffect.java` and PRs #13651 / #13647: the move of non-target choices to resolution.
- `EffectImpl.java`, `Effects.java`, `TargetPointer.java`, `NthTargetPointer.java`, `FirstTargetPointer.java`, `FixedTarget.java`: how pointers are passed and shared.
- `LookLibraryControllerEffect.java`, `LookLibraryAndPickControllerEffect.java`, `PutCards.java`, `Library.java`: look-and-pick.
- `RandomUtil.java`: the randomness helper.
- `PlayerImpl.java`, `Player.java`, `GameState.java`, `GameImpl.java`, `LookedAt.java`: looking at and revealing cards.
- Card files: Impulse, Anticipate, AncestralMemories, RaiseDead, RunAmok, PeerPressure, WildfireDevils, WildSwing, CapriciousEfreet, GoblinTestPilot, WormfangNewt, YorionSkyNomad, and MetamorphicAlteration at an old commit (to show `setNotTarget`).

**Dropped:**
- javatips.net / massapi.com mirrors: old copies of the code.
- MTG Salvation and Reddit custom-card posts: not relevant.
- grep.app: blocked by a Vercel checkpoint.

## Gaps
- I found no card where a resolution-time choice sets the pointer on a *sibling* effect of the same ability, and no verified card for "random creature +1/+1 and gains X". Next step: run a local `git clone` plus `grep -rn "setRandom(true)\|randomFromCollection" Mage.Sets` and `grep -rn "getEffects().get(1).setTargetPointer"`.
- I did not check whether any non-target Target is still added to ability `Targets`, which matters for fizzling.
- I did not review `GameView` or face-down "known card" handling.
- All links point to `master`, which keeps changing. Pin to a commit SHA if you need stable citations.