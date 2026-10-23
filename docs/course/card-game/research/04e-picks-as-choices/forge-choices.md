# Forge (Card-Forge/forge): how card scripts handle choices, targets, Dig, random picks and spellbooks

## Summary
In Forge, a casting-time target and a resolution-time choice are written two different ways. A target uses `ValidTgts$` on the ability, which builds a `TargetRestrictions` object, and later sub-abilities refer back to it with `Defined$ Targeted`, `ParentTarget` or `TargetedController`. A resolution-time choice is a separate verb (`ChooseCard`, or the choice built into `Dig`/`Draft`/`MakeCard`). That verb stores the result on the host card's "chosen" or "remembered" list, and later sub-abilities read it with `Defined$ ChosenCard` or `Defined$ Remembered`. One selection can feed several verbs: each `SubAbility` in the chain reads the same stored list. The two mechanisms share the final step (`AbilityUtils.getDefinedCards`), but I found no single declaration or flag that switches a selection between targeted and non-targeted. For your design, Forge is a counter-example: the picks are kept as host-card state, not as typed declarations.

All files are on `master`. Raw URL pattern: `https://raw.githubusercontent.com/Card-Forge/forge/master/<path>`.

---

## Q1. `Dig` (DigEffect.java)

**Files:** `forge-game/src/main/java/forge/game/ability/effects/DigEffect.java` ([link](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/effects/DigEffect.java)), plus the `Dig` section of the [AbilityFactory wiki](https://github.com/Card-Forge/forge/wiki/AbilityFactory).

**Impulse:** `forge-gui/res/cardsfolder/i/impulse.txt` ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/i/impulse.txt))
```
A:SP$ Dig | DigNum$ 4 | ChangeNum$ 1 | NoReveal$ True | SpellDescription$ Look at the top four cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order.
```

**Anticipate:** `forge-gui/res/cardsfolder/a/anticipate.txt` ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/a/anticipate.txt))
```
A:SP$ Dig | DigNum$ 3 | ChangeNum$ 1 | NoReveal$ True | SpellDescription$ Look at the top three cards of your library. Put one of them into your hand and the rest on the bottom of your library in any order.
```
Neither card sets a destination parameter. They rely entirely on the defaults: Hand for the picked cards, bottom of the library for the rest.

**How the picked cards and the rest each get a destination.** Dig always splits the dug cards into two groups, and each group has its own zone and position parameters:
```java
final ZoneType destZone1 = sa.hasParam("DestinationZone") ? ZoneType.smartValueOf(sa.getParam("DestinationZone")) : ZoneType.Hand;
final ZoneType destZone2 = sa.hasParam("DestinationZone2") ? ZoneType.smartValueOf(sa.getParam("DestinationZone2")) : ZoneType.Library;
final int libraryPosition = sa.hasParam("LibraryPosition") ? Integer.parseInt(sa.getParam("LibraryPosition")) : -1;
final int libraryPosition2 = sa.hasParam("LibraryPosition2") ? Integer.parseInt(sa.getParam("LibraryPosition2")) : -1;
```
- `ChangeNum$` is a number, `All` or `Any`. `ChangeValid$` filters which dug cards can be picked. The picked cards go to `destZone1` at `libraryPosition`.
- "The rest" is not declared anywhere. It is simply what is left after the picks are removed:
  ```java
  rest.addAll(top);
  ...
  for (Card c : movedCards) { ... rest.remove(c); }
  // now, move the rest to destZone2
  ```
- For library or graveyard, the rest is ordered either at random (`RestRandomOrder` → `CardLists.shuffle(afterOrder)`) or by the player (`orderMoveToZoneList`), unless `SkipReorder` is set. They are then moved with `moveTo(destZone2, c, libraryPosition2, ...)`. A position of `-1` means the bottom.
- `RememberMovedToZone$ 1|2` / `RememberChanged` save either group into the host's remembered list for later sub-abilities. `DestZone2Optional` makes the second move a "may".

**`Reveal`.** `Reveal$ True` calls `game.getAction().reveal(top, p, false)` before the choice. Otherwise, unless `NoLooking` is set, a `DelayedReveal` (private look) is used. `NoReveal` only suppresses the post-choice reveal of the picked cards when `ChangeValid` is present:
```java
if (!changeValid.isEmpty() && !sa.hasParam("ExileFaceDown") && !sa.hasParam("NoReveal")) { game.getAction().reveal(movedCards, ...
```

**`NoMove`: UNVERIFIED.** The string `NoMove` does not appear in the current `DigEffect.java`. It may be an old or removed parameter, or belong to another effect.

**Where the dug cards are while the player chooses.** They stay in the library. `top` is only a collection that points at library cards: `CardCollection all = new CardCollection(p.getCardsIn(srcZone)); ... top.add(all.get(i));`. Nothing moves before the choice. The UI shows the cards temporarily:
```java
if (p == chooser) { // the digger can still see all the dug cards when choosing
    chooser.getController().tempShowCards(top);
}
...
movedCards.addAll(chooser.getController().chooseEntitiesForEffect(valid, min, max, delayedReveal, sa, prompt, p, null));
chooser.getController().endTempShowCards();
```
Cards move only after the choice, picks first and the rest second. There is no "limbo" or "looked-at" zone. The source zone is configurable with `SourceZone$`, and `FromBottom` reverses the list.

---

## Q2. Resolution-time choices: `ChooseCard` and Remember

**File:** `forge-game/src/main/java/forge/game/ability/effects/ChooseCardEffect.java` ([link](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/effects/ChooseCardEffect.java))

**Where the options come from:**
- **`ChoiceZone$`** sets the zone(s) to choose from. The default is the battlefield:
  ```java
  List<ZoneType> choiceZone = Lists.newArrayList(ZoneType.Battlefield);
  if (sa.hasParam("ChoiceZone")) { choiceZone = ZoneType.listValueOf(sa.getParam("ChoiceZone")); }
  CardCollectionView choices = sa.hasParam("AllCards") ? game.getCardsInGame() : game.getCardsIn(choiceZone);
  if (sa.hasParam("Choices")) { choices = CardLists.getValidCards(choices, sa.getParam("Choices"), activator, host, sa); }
  ```
- **`DefinedCards$`** replaces the pool with any defined set, for example `Targeted`. This is how a choice can be made among targets (see Q4).
- **`Defined$`** sets who chooses. Each player chooses in turn, and `ControlledByPlayer$ Chooser` limits each player to their own cards. `Amount`/`MinAmount` set the count, and `Mandatory` disables cancel.

**`AtRandom$`.** The engine picks with no player prompt:
```java
} else if (sa.hasParam("AtRandom") && !pChoices.isEmpty()) {
    chosen = new CardCollection(Aggregates.random(pChoices, validAmount));
```

**Where the result is stored.** On the host card, not on the ability:
```java
host.setChosenCards(allChosen);
if (sa.hasParam("ForgetOtherRemembered")) { host.clearRemembered(); }
if (sa.hasParam("RememberChosen")) { host.addRemembered(allChosen); }
...
if (sa.hasParam("ImprintChosen")) { host.addImprintedCards(allChosen); }
```
There is also `ChosenMap`, which stores choices per player.

**How later sub-abilities read it.** They go through `AbilityUtils.getDefinedCards` ([AbilityUtils.java](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/AbilityUtils.java)):
```java
} else if (defined.equals("ChosenCard")) {
    for (final Card chosen : hostCard.getChosenCards()) { cards.add(game.getCardState(chosen)); }
...
} else if (defined.equals("Remembered") || defined.equals("RememberedCard")) { ... newCard.getRemembered() ...
```
`ChosenCard` and `IsRemembered` can also be used as property filters, e.g. `ValidCards$ Creature.ChosenCard` ([Targeting wiki](https://github.com/Card-Forge/forge/wiki/Targeting): "ChosenCard/ChosenType — For cards that were chosen with ChooseCard/Type API"). Because the state lives on the host card, scripts have to clear it themselves with `DB$ Cleanup | ClearChosenCard$ True / ClearRemembered$ True`.

**`ForgetOnMoved`.** This belongs to the `Effect` API (`EffectEffect.java`), not to `ChooseCard`. It is used together with `RememberObjects$`:
```java
if (sa.hasParam("ForgetOnMoved")) {
    addForgetOnMovedTrigger(eff, sa.getParam("ForgetOnMoved"));
```
`SpellAbilityEffect.addForgetOnMovedTrigger` installs the trigger `Mode$ ChangesZone | ValidCard$ Card.IsRemembered | Origin$ <zone> ...`. The wiki describes it as: "gets rid of the effect if all remembered cards have left the affected zone". Example from a commit diff of Glamorous Outlaw: `AB$ Effect | ... ValidTgts$ Land | RememberObjects$ Targeted,Self | ... ForgetOnMoved$ Exile` ([commit 8afa17c](https://github.com/Card-Forge/forge/commit/8afa17c510d34c5f71cf6929aad2f889fd62b2b2)). I did not fetch the full Glamorous Outlaw script.

**Example: choose during resolution, then two effects on the chosen card.** Garruk, Wrath of the Wilds (Alchemy), `forge-gui/res/cardsfolder/g/garruk_wrath_of_the_wilds.txt` ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/g/garruk_wrath_of_the_wilds.txt)):
```
A:AB$ ChooseCard | Cost$ AddCounter<1/LOYALTY> | Planeswalker$ True | ChoiceZone$ Hand | Choices$ Card.Creature+YouOwn | ChoiceTitle$ Choose a creature card in your hand | Mandatory$ True | SubAbility$ DBPump | ...
SVar:DBPump:DB$ Pump | PumpZone$ Hand | Defined$ ChosenCard | NumAtt$ +1 | NumDef$ +1 | Duration$ Perpetual | SubAbility$ DBAnimate
SVar:DBAnimate:DB$ Animate | Defined$ ChosenCard | staticAbilities$ ReduceCost | Duration$ Perpetual | SubAbility$ DBCleanup1 | StackDescription$ None
SVar:DBCleanup1:DB$ Cleanup | ClearChosenCard$ True
```
One choice (from the hand, at resolution) feeds two verbs, then cleanup runs.

**A variant with a hand-off between lists.** Agitator Ant ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/a/agitator_ant.txt)):
```
SVar:DBChoice:DB$ ChooseCard | Defined$ Player | Amount$ 1 | MinAmount$ 0 | Choices$ Creature | ControlledByPlayer$ Chooser | SubAbility$ DBPutCounter
SVar:DBPutCounter:DB$ PutCounterAll | Placer$ Controller | ValidCards$ Creature.ChosenCard | CounterType$ P1P1 | CounterNum$ 2 | RememberPut$ True | SubAbility$ DBGoad
SVar:DBGoad:DB$ Goad | Defined$ Remembered | SubAbility$ DBCleanup
```
Effect 2 does not act on "the chosen cards". It acts on "the cards that actually received counters": the first verb writes Remembered, and the second reads it. This is a precise way to express "those that had counters put on them this way".

---

## Q3. Reusing a target across sub-abilities

**What the `Defined$` keywords resolve to** (`AbilityUtils.java`):
- **`Targeted`** collects targets from every target choice in the chain:
  ```java
  } else if ((defined.equals("Targeted") || defined.equals("TargetedCard")) && sa instanceof SpellAbility) {
      for (TargetChoices tc : ((SpellAbility)sa).getAllTargetChoices()) {
          for (Card tgt : tc.getTargetCards()) { cards.add(game.getChangeZoneLKIInfo(tgt)); }
  ```
- **`ParentTarget`** uses only the nearest parent that targeted a card:
  ```java
  } else if (defined.equals("ParentTarget") && sa instanceof SpellAbility) {
      final SpellAbility parent = ((SpellAbility)sa).getParentTargetingCard();
  ```
  `ThisTargetedCard` excludes the parents.
- **`TargetedController`** maps each target to its controller:
  ```java
  } else if (defined.equals("TargetedController")) {
      for (final Card c : getDefinedCards(card, "Targeted", sa)) { players.add(c.getController()); }
  ```
  `TargetedOwner`, `ParentTargetedController` and `ThisTargetedController` also exist.

**Rule for an ability that has both.** If an ability has its own `ValidTgts`, it uses its own targets. Otherwise it uses `Defined` (`SpellAbilityEffect.getCards`):
```java
final boolean useTargets = sa.usesTargeting() && (!definedFirst || !sa.hasParam(definedParam));
```

**Example: one target, two effects, the second conditional.** Temur Battle Rage ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/t/temur_battle_rage.txt)):
```
A:SP$ Pump | ValidTgts$ Creature | KW$ Double Strike | SubAbility$ DBPump | ...
SVar:DBPump:DB$ Pump | Defined$ Targeted | KW$ Trample | ConditionPresent$ Creature.YouCtrl+powerGE4 | ConditionCompare$ GE1 | ...
```

**Example: target, then that target's controller.** Chandra's Outrage ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/c/chandras_outrage.txt)):
```
A:SP$ DealDamage | ValidTgts$ Creature | NumDmg$ 4 | SubAbility$ DBDealDamage | DamageMap$ True | ...
SVar:DBDealDamage:DB$ DealDamage | Defined$ TargetedController | NumDmg$ 2 | SubAbility$ DBDamageResolve
```

**Example: two separate targets in one chain.** Political Trickery ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/p/political_trickery.txt)). The root `Pump` does nothing except declare the first target. The sub-ability declares a second target and also pulls in the parent's target:
```
A:SP$ Pump | ValidTgts$ Land.YouCtrl | TgtPrompt$ Choose target land you control | SubAbility$ DBExchange | ...
SVar:DBExchange:DB$ ExchangeControl | Defined$ ParentTarget | ValidTgts$ Land.OppCtrl | TgtPrompt$ Choose target land an opponent controls
```

**Another option: one verb with several modifiers.** Act of Treason handles "gain control, untap, haste" as one verb with flags (`GainControl | ValidTgts$ Creature | LoseControl$ EOT | Untap$ True | AddKWs$ Haste`) ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/a/act_of_treason.txt)).

**Bridging a target into Remembered.** Flicker uses `RememberTargets$ True`, then `Defined$ Remembered`. This is needed because the exiled card becomes a new object ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/f/flicker.txt)).

---

## Q4. Reusing a random selection

**Pattern A: `ChooseCard | AtRandom$ True`, then `Defined$ ChosenCard`.** Tibalt's Boss Effect is an Adventure-mode custom card, not a printed card: `forge-gui/res/adventure/common/custom_cards/tibalt_boss_effect.txt` ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/adventure/common/custom_cards/tibalt_boss_effect.txt)):
```
SVar:DBDamage:DB$ ChooseCard | AtRandom$ True | Choices$ Creature | SubAbility$ DBDamageBis | ...
SVar:DBDamageBis:DB$ DealDamage | NumDmg$ 7 | Defined$ ChosenCard | SubAbility$ DBChangeZone
...
SVar:DBCleanup:DB$ Cleanup | ClearChosenCard$ True
```
Any number of later sub-abilities could use `Defined$ ChosenCard`. Garruk (Q2) shows the two-verb form with a manual pick.

**Pattern B: pick at random among the targets.** Wild Swing ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/w/wild_swing.txt)). Targets are declared at cast time, the random pick happens at resolution, and a third verb acts on the result:
```
A:SP$ Pump | ValidTgts$ Permanent.nonEnchantment | TgtPrompt$ Select target nonenchantment permanent | TargetMin$ 3 | TargetMax$ 3 | IsCurse$ True | SubAbility$ DBChooseRandom | ...
SVar:DBChooseRandom:DB$ ChooseCard | Defined$ You | Amount$ 1 | AtRandom$ True | DefinedCards$ Targeted | SubAbility$ DBDestroy
SVar:DBDestroy:DB$ Destroy | Defined$ ChosenCard
```
This is the closest Forge gets to "one declaration feeding a later pick".

**Pattern C: random targeting.** `TargetsAtRandom$ True` sets `randomTarget` in `TargetRestrictions` ([TargetRestrictions.java](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/spellability/TargetRestrictions.java)): `if (mapParams.containsKey("TargetsAtRandom")) { setRandomTarget(true); }`. `RandomNumTargets` also exists. Goblin Test Pilot ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/g/goblin_test_pilot.txt)):
```
A:AB$ DealDamage | Cost$ T | NumDmg$ 2 | ValidTgts$ Any | TargetsAtRandom$ True | ...
```
A random target is still stored as a target, so `Defined$ Targeted` on a sub-ability should reuse it. **UNVERIFIED:** I did not find a printed card that chains a second effect after `TargetsAtRandom`. I also did not find a printed card with "two effects on the same random creature" (e.g. "+1/+1 and haste"). The Tibalt boss card is the only verified `AtRandom` → `ChosenCard` chain, and its second step (`DBChangeZone`) does not act on the chosen card.

---

## Q5. Does a choice at resolution share code with targeting?

**Mostly not.**
- **Targeting** uses `TargetRestrictions`, which is built from `ValidTgts`, `TargetMin`/`TargetMax`, `TgtZone`, `TgtPrompt`, `TargetsAtRandom` and `TargetingPlayer`. The zone defaults to the battlefield: `private List<ZoneType> tgtZone = Arrays.asList(ZoneType.Battlefield);`. Targets are chosen at cast time and stored in `TargetChoices` on the ability.
- **`ChooseCard`** does not use `TargetRestrictions`. It has its own parameter names that mirror the targeting ones: `Choices$` (instead of `ValidTgts`), `ChoiceZone$` (instead of `TgtZone`), `Amount`/`MinAmount` (instead of `TargetMin`/`Max`), and `ChoiceTitle` (instead of `TgtPrompt`). It asks the player through `PlayerController.chooseCardsForEffect` / `chooseSingleEntityForEffect`.
- **What they do share:**
  - The "Valid" filter language (`CardLists.getValidCards(..., "Creature.YouCtrl", ...)`).
  - Read-back through `AbilityUtils.getDefinedCards`.
  - The `SpellAbilityEffect` helpers `getTargetCards` / `getDefinedCardsOrTargeted`. Effects call these and do not care whether the cards came from a target or a definition.

  The wiki states the split directly: "`Defined$ {Defined}` if the ability describes on what it's applied – `ValidTgts$ {Valid}` if the ability targets" ([AbilityFactory wiki](https://github.com/Card-Forge/forge/wiki/AbilityFactory)), and "A `Defined` parameter states what is receiving the action. Remember this is non-targeted!" ([Targeting wiki](https://github.com/Card-Forge/forge/wiki/Targeting)).
- **No switch parameter.** I found no parameter that turns a `ChooseCard` into a targeted choice or the reverse (UNVERIFIED as an absence, based on reading `ChooseCardEffect` and `TargetRestrictions`). To change a card from one form to the other, the script author rewrites it: `ValidTgts$X` becomes `ChooseCard | Choices$X` + `Defined$ ChosenCard`.
- **Related mechanisms, not toggles:**
  - Many effects have their own built-in resolution-time choice, e.g. `PutCounter | Choices$` or `Attach | Choices$`. The wiki says this is required for CR 608.2d compliance and helps the AI.
  - `TargetingPlayer$` changes who chooses the target.
  - `DefinedCards$ Targeted` lets a choice be made among targets (Wild Swing).
  - `getDefinedCardsOrTargeted` gives an ability's own `Defined` priority over its targets.

---

## Q6. Discover, Seek, Draft, MakeCard: zone objects or card names?

It depends on the verb:

- **Discover** (`DiscoverEffect.java`, [link](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/effects/DiscoverEffect.java)) uses real library objects with no choice among N. It exiles from the top until it finds a nonland card with mana value ≤ N (`for (final Card c : library) { exiled.add(c); if (!c.isLand() && c.getCMC() <= num) { found = c; ...`). The player's only decision is "Cast" or "Hand" (`confirmAction(sa, null, prompt, options, found, params)`). The rest goes to the bottom of the library in random order.
- **Seek** (`SeekEffect.java`, [link](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/effects/SeekEffect.java)) uses real library objects picked at random with no prompt: `for (final Card c : Aggregates.random(pool, seekNum)) { ... game.getAction().moveToHand(c, sa, moveParams);`. `DefinedCards` can replace the pool. `RememberFound` hands the result to later sub-abilities.
- **Draft from a spellbook** (`DraftEffect.java`, [link](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/effects/DraftEffect.java)) starts from card names and turns three of them into temporary card objects with no zone:
  ```java
  List<String> spellbook = Arrays.asList(sa.getParam("Spellbook").split(","));
  Collections.shuffle(spellbook, MyRandom.getRandom());
  for (String name : spellbook.subList(0, 3)) { ... Card cardOption = Card.fromPaperCard(pc, player); draftOptions.add(cardOption); }
  Card chosenCard = player.getController().chooseSingleCardForZoneChange(ZoneType.None, ..., new CardCollection(draftOptions), ...);
  game.getAction().moveTo(ZoneType.None, chosenCard, sa, moveParams);
  ```
  So the options are created as zone-less `Card` objects, the player picks one through the normal card-choice UI, and it then moves to Hand. The other two are discarded. `RememberDrafted` hands the result on. Script example from Cursebound Witch ([raw](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/c/cursebound_witch.txt)): `SVar:TrigDraft:DB$ Draft | Spellbook$ Witch's Cauldron,Witch's Vengeance,...`. Garruk's −1 chains `Draft | RememberDrafted$ True` into `ChangeZone | Origin$ Hand | Destination$ Battlefield | Defined$ Remembered`.
- **MakeCard / conjure** (`MakeCardEffect.java`, [link](https://github.com/Card-Forge/forge/blob/master/forge-game/src/main/java/forge/game/ability/effects/MakeCardEffect.java)) chooses among names and definitions (`ICardFace`), not objects. `Spellbook$`/`Choices$` are parsed to faces, and the player picks a name (or `AtRandom` picks one):
  ```java
  chosen = player.getController().chooseCardName(sa, faces, message);
  ```
  Only after the choice is a `Card` built (`Card.fromPaperCard(pc, player)`) and moved to `Zone$` (default Library). `Name$ ChosenName` builds from a name picked earlier by `NameCard` (`ChooseCardNameEffect`). `RememberMade` hands the result on.
- **`ChooseCard` itself** only chooses among in-game objects (`game.getCardsIn(choiceZone)`, or `AllCards` → `game.getCardsInGame()`). It cannot offer cards from outside the game.

**Takeaway for your design.** Forge has two kinds of "outside the game" choice. `MakeCard` chooses a name or definition and creates the card afterwards. `Draft` creates the candidate cards first, with no zone, and then reuses the normal card-choice UI.

---

## Sources
- **Kept:**
  - DigEffect.java, ChooseCardEffect.java, DiscoverEffect.java, SeekEffect.java, DraftEffect.java, MakeCardEffect.java, EffectEffect.java, SpellAbilityEffect.java, AbilityUtils.java, TargetRestrictions.java and ApiType.java (raw on `master`): primary engine source.
  - Card scripts: impulse, anticipate, garruk_wrath_of_the_wilds, agitator_ant, temur_battle_rage, chandras_outrage, political_trickery, act_of_treason, flicker, wild_swing, goblin_test_pilot, cursebound_witch, tibalt_boss_effect (adventure custom card).
  - Forge wiki pages [Targeting](https://github.com/Card-Forge/forge/wiki/Targeting) and [AbilityFactory](https://github.com/Card-Forge/forge/wiki/AbilityFactory): official scripter documentation.
- **Dropped:** slightlymagic.net forum threads (old and informal), magicmultiverse/reddit custom-card results (not Forge), the Hugging Face dataset (derived, not primary), and the third-party manabrew doc.

## Gaps
- `NoMove` was not found in the current `DigEffect.java` (UNVERIFIED). It may be removed or belong to a different effect. Searching git history would settle it.
- No printed card was found that applies two effects to the same random creature, or that chains `TargetsAtRandom` into `Defined$ Targeted` (UNVERIFIED). Code search requires a GitHub login. Next step: `grep -rl "AtRandom\$ True" forge-gui/res/cardsfolder | xargs grep -l "Defined\$ ChosenCard"` on a local clone.
- `TargetSelection.java` (how random targets are filled in) and `SpellAbility.getParentTargetingCard` were not read in full.
- The Glamorous Outlaw `ForgetOnMoved` example was quoted from a commit diff, not from the current file.