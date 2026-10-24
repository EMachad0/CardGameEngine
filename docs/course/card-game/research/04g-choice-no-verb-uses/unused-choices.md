# Unused choices in card data: what real games and engines do

## Summary
Printed MTG cards almost always do something with every target they declare, but the effect does not have to change the target. Rabid Bite uses its first target as the damage source, Soul's Majesty reads its target's power, and Mirrorweave copies its target. I could not find an MTG or Hearthstone card whose target is declared and then never used at all. The engines I checked (Forge, XMage, Spellsource) all ask the player for a declared target whether or not any effect reads it. None of them has a linter for "declared target that nothing uses". The nearest checks are XMage's comparison of targets against the card text and Forge's draft check for unused SVars.

---

## 1. MTG: targets that are not the thing being acted on

**Cards whose target is used as something other than the object acted on (Oracle text from Scryfall):**

- **Rabid Bite**: *"Target creature you control deals damage equal to its power to target creature you don't control."* The first target is only the damage source and the place the number comes from. [Scryfall API](https://api.scryfall.com/cards/named?exact=Rabid%20Bite)
  - Ruling (2020-09-25): *"If either creature is an illegal target as Rabid Bite tries to resolve, the creature you control won't deal damage."* [Rulings](https://api.scryfall.com/cards/53a73200-b798-4bfd-a431-8b94e17b70be/rulings)
- **Soul's Majesty**: *"Draw cards equal to the power of target creature you control."* The target is only read for a number, and the action is done to you. [Scryfall](https://api.scryfall.com/cards/named?exact=Soul%27s%20Majesty&format=text)
- **Mirrorweave**: *"Each other creature becomes a copy of target nonlegendary creature until end of turn."* The target is the one creature this card does not change. It is only the model being copied. [Scryfall API](https://api.scryfall.com/cards/named?exact=Mirrorweave)
- **Hunter's Insight**: *"Choose target creature you control. Whenever that creature deals combat damage to a player or planeswalker this turn, draw that many cards."* The target is only remembered for a later trigger. [Scryfall](https://api.scryfall.com/cards/named?exact=Hunter%27s%20Insight&format=text)
- **Soul's Fire** works like Rabid Bite: *"Target creature you control deals damage equal to its power to any target."* [Scryfall](https://api.scryfall.com/cards/named?exact=Soul%27s%20Fire&format=text)
- **Spellskite** is a target that can end up doing nothing: *"Change a target of target spell or ability to this creature."* A Reddit answer (not an official ruling) says that if Spellskite isn't a legal new target, *"Spellskite's ability does nothing when it resolves."* [Scryfall](https://api.scryfall.com/cards/named?exact=Spellskite&format=text), [r/magicTCG](https://www.reddit.com/r/magicTCG/comments/173dw30/can_spellskite_really_target_spells_that_dont/)

**Abilities that react just to being targeted:**

- **Heroic**, on Akroan Crusader: *"Whenever you cast a spell that targets this creature, create a 1/1 red Soldier creature token with haste."* [Scryfall](https://api.scryfall.com/cards/named?exact=Akroan%20Crusader&format=text)
  - Heroic triggers when the spell is cast, not when it resolves: *"The heroic triggers upon being targeted, not resolution of the targeting spell."* (Reddit, not official) [r/mtgjudge](https://www.reddit.com/r/mtgjudge/comments/1mvaa7/heroic_triggers_and_spell_resolving/)
- **Valiant**, on Heartfire Hero: *"Whenever this creature becomes the target of a spell or ability you control for the first time each turn, put a +1/+1 counter on it."* [Scryfall](https://api.scryfall.com/cards/named?exact=Heartfire%20Hero&format=text)
- **Illusions**, on Phantasmal Bear, Gossamer Phantasm, Phantasmal Dragon and others: *"When this creature becomes the target of a spell or ability, sacrifice it."* [Scryfall search](https://api.scryfall.com/cards/search?q=o%3A%22becomes+the+target%22+o%3Asacrifice+t%3Aillusion&unique=cards)
- **Ward** (CR 702.21a): *"Whenever this permanent becomes the target of a spell or ability an opponent controls, counter that spell or ability unless that player pays [cost]."* [CR TXT](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)

In every case the act of targeting is itself something that happens in the game. A target that the effect never uses still triggers these abilities.

**Not found:** I could not find a printed, black-bordered card whose text declares a target that no instruction refers to at all. I also could not find a card whose only purpose is to target something for heroic, valiant or ward. The cheap heroic enablers I looked at, such as Defiant Strike, still do something to their target.

## 2. MTG Comprehensive Rules (effective September 25, 2026)
Source: [MagicCompRules 20260925.txt](https://media.wizards.com/2026/downloads/MagicCompRules%2020260925.txt)

- **No rule says the effect must use a target.** 115.1 only describes what targets are supposed to be: *"The targets are object(s) and/or player(s) the spell or ability will affect."* 115.1a: a spell *"is targeted if its spell ability identifies something it will affect by using the phrase 'target [something]'"*. These rules describe what targets are. They do not make an unused target illegal.
- **115.10 / 115.10a:** *"Spells and abilities can affect objects and players they don't target… those objects and players aren't chosen until the spell or ability resolves."* This is the rule behind your "pick", next to 608.2d.
- **601.2c (target, chosen while casting):** *"The player announces their choice of an appropriate object or player for each target the spell requires."*
- **608.2d (pick, chosen while resolving):** *"If an effect of a spell or ability offers any choices other than choices already made as part of casting the spell… the player announces these while applying the effect."*
- **608.2b, confirmed:** *"If all its targets, for every instance of the word 'target,' are now illegal, the spell or ability doesn't resolve. It's removed from the stack…"* and *"Illegal targets, if any, won't be affected by parts of a resolving spell's effect for which they're illegal. Other parts of the effect for which those targets are not illegal may still affect them."*
  - Consequence for your engine: if every target is illegal, the rest of the card does not happen either. So a declared target always matters, even when no verb uses it.
- **The ruling you asked for, Defiant Strike** (*"Target creature gets +1/+0 until end of turn. Draw a card."*), WotC ruling dated 2020-06-23: *"If the target creature is an illegal target by the time Defiant Strike tries to resolve, the spell won't resolve. You won't draw a card."* [Card](https://api.scryfall.com/cards/named?exact=Defiant%20Strike), [Rulings](https://api.scryfall.com/cards/5c23869b-c99a-49dd-9e29-fcc0eb63fad1/rulings)

## 3. Hearthstone

- **Play requirements are data on the card, separate from the effect.** The Fireplace engine wiki: *"Hearthstone powers can have one or more `PlayRequirement` assigned to them… Targeting is a separate step from the power triggering, and as such, once targeting is successful, an action can proceed on its target even if the target is no longer valid."* The requirements live in CardDefs.xml. [fireplace wiki](https://github.com/jleclanche/fireplace/wiki/PlayRequirements)
  - The enum includes `REQ_TARGET_TO_PLAY = 11`. [fireplace enums.py](https://github.com/jleclanche/fireplace/blob/8a4b50c18da4bb1b721ca321c2b2b9bc6d9031d9/fireplace/enums.py)
  - HearthDb reports that the PlayRequirement section has been missing from the card data since client 16.0.0.37060. [HearthDb #18](https://github.com/HearthSim/HearthDb/issues/18)
- **The player can't skip a target:** *"Players cannot take an action and choose 'no target' if it normally requires one"*.
  - Battlecries are the exception. *"If a minion has a player-targeted Battlecry but there are no valid targets, it is played as if it had no Battlecry at all."* [wiki.gg Target](https://hearthstone.wiki.gg/wiki/Target)
- **Targets are not rechecked at resolution:** *"If it targets, the target is remembered (and its validity is not checked again)."* This is the opposite of MTG 608.2b. [Advanced rulebook](https://hearthstone.wiki.gg/wiki/Advanced_rulebook)
- **Targets used only as a reference:**
  - Faceless Manipulator: *"Battlecry: Choose a minion and become a copy of it."* [wiki.gg](https://hearthstone.wiki.gg/wiki/Faceless_Manipulator)
  - Shadowcaster: *"Battlecry: Choose a friendly minion. Add a 1/1 copy to your hand that costs (1)."* [wiki.gg](https://hearthstone.wiki.gg/wiki/Shadowcaster)
- **Not found:** a Hearthstone card that requires a target and does nothing with it.
- **Cards that react to being targeted (Spellburst excluded):**
  - Fjola Lightbane: *"Whenever you target this minion with a spell, gain Divine Shield."* [wiki.gg](https://hearthstone.wiki.gg/wiki/Fjola_Lightbane)
  - Eydis Darkbane: *"Whenever you target this minion with a spell, deal 3 damage to a random enemy."* [wiki.gg](https://hearthstone.wiki.gg/wiki/Eydis_Darkbane)
  - Dragonkin Sorcerer: *"Whenever you target this minion with a spell, gain +1/+1."* Its wiki note says it does not trigger from enemy spells or from random/AoE spells. [wiki.gg](https://hearthstone.wiki.gg/wiki/Dragonkin_Sorcerer)
  - The Voraxx: *"After you cast a spell on this minion, summon a 1/1 Plant and cast another copy on it."* [wiki.gg](https://hearthstone.wiki.gg/wiki/The_Voraxx)
  - Myrmidon: *"After you cast a spell on this minion, draw a card."* [wiki.gg](https://hearthstone.wiki.gg/wiki/Myrmidon)
  - Stormwind Avenger: *"After you cast a spell on this minion, it gains +2 Attack."* This text comes from hearthpwn, not wiki.gg. [hearthpwn](https://www.hearthpwn.com/cards/904853-stormwind-avenger)
  - Related, but about any minion: Zentimo, *"Whenever you target a minion with a spell, cast it again on its neighbors."* [wiki.gg](https://hearthstone.wiki.gg/wiki/Zentimo)

## 4. Engines

### (a) Forge
- **Asking for a target depends only on `ValidTgts$`.**
  - In `AbilityFactory`: `TargetRestrictions abTgt = mapParams.containsKey("ValidTgts") ? readTarget(mapParams) : null;` [AbilityFactory.java](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-game/src/main/java/forge/game/ability/AbilityFactory.java)
  - In `SpellAbility`: `public boolean usesTargeting() { return targetRestrictions != null; }`. Nothing here looks at whether an effect reads the target. [SpellAbility.java](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-game/src/main/java/forge/game/spellability/SpellAbility.java)
  - From these lines I infer the player is asked whenever `ValidTgts$` is present. I did not trace the GUI casting path itself.
- **Forge scripts deliberately carry targets on an effect that does nothing with them.**
  - Soul's Majesty: `A:SP$ Pump | ValidTgts$ Creature.YouCtrl | … | SubAbility$ DBDraw` with `SVar:X:Targeted$CardPower`. It is a Pump with no pump parameters, used only to hold the target. [soul's majesty](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/s/souls_majesty.txt)
  - Rabid Bite does the same, with `DamageSource$ ParentTarget`. [rabid bite](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/r/rabid_bite.txt)
  - Hunter's Insight: `SP$ Effect | ValidTgts$ … | RememberObjects$ Targeted`. [hunters insight](https://raw.githubusercontent.com/Card-Forge/forge/master/forge-gui/res/cardsfolder/h/hunters_insight.txt)
- **Validation:** I found no Forge check for an unused target.
  - The closed draft PR #10984 listed its checks. They include *"SVar that's never referenced"* and a duplicate `ValidTgts$`, but not an unused target. [PR #10984](https://github.com/Card-Forge/forge/pull/10984)
  - Its replacement, PR #12030, was an open draft when I fetched it. It checks params the engine never reads, unknown or misspelled params, required-but-missing params, and *"unused and undefined SVars"*. *"239 warnings, such as params only other APIs read and unused SVars"* are reported. It lists no target-use check. [PR #12030](https://github.com/Card-Forge/forge/pull/12030)

### (b) XMage
- **Targets are chosen from the ability's own target list, whatever the effects do.**
  - `AbilityImpl.activate`, next to the 601.2c comment: `if (!getTargets().chooseTargets(outcome, this.controllerId, this, noMana, game, canCancel))`. [AbilityImpl.java](https://raw.githubusercontent.com/magefree/mage/master/Mage/src/main/java/mage/abilities/AbilityImpl.java)
  - Soul's Majesty adds `TargetControlledCreaturePermanent` and reads it in a custom effect through `source.getFirstTarget()`. [SoulsMajesty.java](https://raw.githubusercontent.com/magefree/mage/master/Mage.Sets/src/mage/cards/s/SoulsMajesty.java)
- **`VerifyCardDataTest` is the closest real precedent for linting choices**, but it compares targets against the Oracle text, not against which effects read them. [VerifyCardDataTest.java](https://raw.githubusercontent.com/magefree/mage/master/Mage.Verify/src/test/java/mage/verify/VerifyCardDataTest.java) It fails on:
  - *"wrong target settings (targeted ability found but no target in text)"*
  - *"wrong target settings (must be targeted, but is not)"*
  - *"target count text discrepancy"*
  - *"notTarget should not be used as ability target, should be inside ability effect"*. This last one enforces your target/pick split: choices made at resolution must live inside the effect.
  - *"wrong target tags"* when an ability has two or more targets.

### (c) Spellsource / Metastone
- **The target prompt comes from the card, not from the spell.**
  - Card authoring doc: `targetSelection` is the *"Player targeting prompt"*, `NONE` means *"no target prompt"*, and `TARGET` is *"The player's chosen target"*.
  - On battlecries: *"The `targetSelection` on a battlecry determines what the player picks. The spell receives the chosen target."* [card-authoring.md](https://raw.githubusercontent.com/hiddenswitch/Spellsource/develop/docs/card-authoring.md)
  - SpellArg `TARGET` *"Specifies the EntityReference on which this spell should be cast, overriding the `target` received from the … or selected by the user."* (javadoc text taken from a search snippet; the page now returns 404). [SpellArg javadoc](https://www.playspellsource.com/javadoc/net/demilich/metastone/game/spells/desc/SpellArg.html)
  - My inference: a card with `targetSelection` whose spell sets its own `target` still asks the player, and the answer is simply ignored. I did not run this to confirm it.
- **Validation:** `CardValidation.validateCard` checks:
  - that the card parses;
  - that it has a hero class;
  - that glow conditions don't crash;
  - that "Battlecry:/Opener:" cards have `BATTLECRY` and no `spell`;
  - that "Deathrattle:/Aftermath:" cards have `DEATHRATTLES`;
  - that a deathrattle doesn't target `ADJACENT_MINIONS`.

  `validateCardReferences` checks that referenced card IDs exist. Nothing checks `targetSelection` against how the spell uses the target. [CardValidation.java](https://raw.githubusercontent.com/hiddenswitch/Spellsource/develop/spellsource-testutils/src/main/java/com/hiddenswitch/spellsource/testutils/CardValidation.java)
  - The repo's failure analysis of 215 failures has no category for unused targets either. [card-validation-errors.md](https://raw.githubusercontent.com/hiddenswitch/Spellsource/develop/docs/card-validation-errors.md)

### (d) Any engine that lints "declared choice nobody uses"
**Not found.** The closest real checks are:
- Forge's unused/undefined SVar warnings (draft PR #12030);
- XMage's checks that targets match the card text, and that resolution-time choices sit inside effects;
- Manabrew's `forge-card-script` parser/LSP, which has diagnostics. I did not check whether it covers unused targets. [crates.io](https://crates.io/crates/forge-card-script)

## 5. Yu-Gi-Oh! (brief)
Not found in a quick search. PSCT text puts targeting in the activation part, before the semicolon (*"CONDITIONS : ACTIVATION ; RESOLUTION"*). [yugioh-card.com PSCT 3](https://www.yugioh-card.com/en/play/psct/psct-3/) I did not find a specific card that targets something and then does nothing to it.

---

## Summary for the design question
- **Don't reject targets that no verb acts on.** Real cards use targets only as a number source (Soul's Majesty), a damage source (Rabid Bite), a copy model (Mirrorweave) or a remembered object (Hunter's Insight). Forge and XMage support all of these, Forge through no-op Pump effects. The safe rule is that every declared target must be *referenced somewhere*: by a verb, a value, a source, or a later trigger. Not that a verb must act on it.
- **Don't skip asking for an unreferenced target.** In MTG, targeting happens during casting (601.2c). It triggers heroic, valiant, Illusions and ward, and per 608.2b, if every target is illegal at resolution the whole spell is cancelled, including the "Draw a card" (Defiant Strike ruling). Hearthstone "whenever you target this minion" cards depend on it too. Skipping the prompt changes the game.
- **A target nothing references at all is best treated as a load-time lint warning, not an error.** No engine I checked lints this. The nearest precedents are XMage (targets vs text) and Forge (unused SVars, warnings that never fail a build).
- **A "pick" that nothing references can be a hard error.** It is chosen during resolution (608.2d / 115.10), nothing observes it, and nothing reacts to it. XMage already enforces that resolution-time choices belong inside effects ("notTarget should not be used as ability target").
- **If you lint targets, allow an explicit "target-only" or "reference" marker on the card data.** Forge's no-op `SP$ Pump` holder is a sign that such cards need one.