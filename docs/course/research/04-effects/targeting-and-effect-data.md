# Targeting and effects as data: verified engine facts (session 04)

Researcher report, 2026-10-03. Local check added at the end.

## 1. Hearthstone: targeted spell vs targeted Battlecry with no valid target (verified)
- https://hearthstone.wiki.gg/wiki/Target, Minion Battlecries: "If a minion has a player-targeted Battlecry but there are no valid targets, it is played as if it had no Battlecry at all - once the board position is selected, the minion is summoned and the Battlecry has no effect." / "Unlike spells however, minions are never prevented from being played due to a lack of Battlecry targets."
- Same page: "Players cannot take an action and choose "no target" if it normally requires one". "When there are no valid targets at all, different consequences occur depending on the card and type of targeting."
- https://hearthstone.wiki.gg/wiki/Battlecry: "An appropriate target must be selected, if possible… If no valid target exists, the Battlecry will not take effect, although any visual-audio effects will still be triggered."

## 2. Hearthstone play requirements (verified; not in the current HearthstoneJSON feed)
- SabberStone `Card.cs`: `REQ_TARGET_TO_PLAY` sets `MustHaveTargetToPlay = true; needsTarget = true;` ("True if playing this card requires at least one valid target."). `REQ_TARGET_IF_AVAILABLE` sets only `needsTarget = true;`.
- Old build https://api.hearthstonejson.com/v1/25770/enUS/cards.collectible.json:
  - Execute (CS2_108): `{"REQ_DAMAGED_TARGET":0,"REQ_ENEMY_TARGET":0,"REQ_MINION_TARGET":0,"REQ_TARGET_TO_PLAY":0}`
  - Fire Elemental (CS2_042): `{"REQ_TARGET_IF_AVAILABLE":0}`
  - Elven Archer (CS2_189): `{"REQ_NONSELF_TARGET":0,"REQ_TARGET_IF_AVAILABLE":0}`
- https://hearthstonejson.com/docs/cards.html: "The `playRequirements` attribute contains an array of `key: param` values which determine various requirements which have to be met for the card to be played and what it can target."
- The current `/v1/latest/` feed has no `playRequirements` field.

## 3. SabberStone (verified)
- `PlayCardTask(Controller controller, IPlayable source, ICharacter target = null, int zonePosition = -1, int chooseOne = 0, bool skipPrePhase = false)`.
- `Controller.cs`, `GetPlayCardTasks`: `if (!card.IsPlayableByCardReq(this)) return; Character[] targets = GetTargets(card);` With no targets: `if (card.MustHaveTargetToPlay) return;` else one `PlayCardTask` per target (minions: per target x board position). `GetTargets`: "Returns null if targeting is not required // Returns 0 Array if there is no available target".
- `Card.cs`: `Dictionary<PlayReq, int> PlayRequirements`, `RequiresTarget => PlayRequirements.ContainsKey(PlayReq.REQ_TARGET_TO_PLAY)`, `MustHaveTargetToPlay`, `IsPlayableByCardReq`, `TargetingAvailabilityPredicate`.

## 4. Metastone / Spellsource (verified, demilich1/metastone)
- `TargetSelection`: `NONE, AUTO, ANY, MINIONS, ENEMY_CHARACTERS, FRIENDLY_CHARACTERS, ENEMY_MINIONS, FRIENDLY_MINIONS, HEROES, ENEMY_HERO, FRIENDLY_HERO`.
- `SpellCard.java`: `setTargetRequirement(desc.targetSelection);`, `play()` returns `new PlaySpellCardAction(getSpell(), this, getTargetRequirement())`.
- `ActionLogic.rollout`: NONE/AUTO adds the action as is; otherwise one cloned action per `targetLogic.getValidTargets(...)`, so zero valid targets means zero actions.
- `MetaSpell.java` casts each child of `SpellArg.SPELLS` in order. Shield Block: `"class": "MetaSpell", "spells": [ {"class": "BuffHeroSpell", "target": "FRIENDLY_HERO", "armorBonus": 5}, {"class": "DrawCardSpell", "value": 1} ]`.
- Hellfire: `"targetSelection": "NONE", "spell": { "class": "DamageSpell", "target": "ALL_CHARACTERS", "value": 3 }`.

## 5. MTG Comprehensive Rules (2025-11-14 text, verified)
- 601.2c: "The player announces their choice of an appropriate object or player for each target the spell requires." 601.2h (later): "The player pays the total cost." 115.1a: "The target(s) are chosen as the spell is cast; see rule 601.2c."
- No legal target means the cast is illegal, via 601.2: "If a player is unable to comply with the requirements of a step listed below while performing that step, the casting of the spell is illegal; the game returns to the moment before the casting of that spell was proposed." Exception 115.6 (spells that allow zero targets).
- 608.2b: "If all its targets, for every instance of the word "target," are now illegal, the spell or ability doesn't resolve."
- 704.3: SBAs are checked "Whenever a player would get priority". 704.4: "Unlike triggered abilities, state-based actions pay no attention to what happens during the resolution of a spell or ability."

## 6. Forge (verified)
- Electrolyze: `A:SP$ DealDamage | ValidTgts$ Any | … | NumDmg$ 2 | TargetMin$ 1 | TargetMax$ 2 | DividedAsYouChoose$ 2 | SubAbility$ DBDraw | …` and `SVar:DBDraw:DB$ Draw`.
- Chandra's Outrage: `A:SP$ DealDamage | ValidTgts$ Creature | NumDmg$ 4 | SubAbility$ DBDealDamage` and `SVar:DBDealDamage:DB$ DealDamage | Defined$ TargetedController | NumDmg$ 2`.
- Wiki: "`Defined$` ... if the ability describes on what it's applied" / "`ValidTgts$` ... if the ability targets"; Targeting page: "A `Defined` parameter states what is receiving the action. Remember this is non-targeted!"
- `AbilityUtils`: `sa.resolve();` then `resolveSubAbilities(sa, game);`. Parent first, then the chain.

## 7. Rust function pointers (verified)
- https://doc.rust-lang.org/std/primitive.fn.html: PartialEq, Eq, PartialOrd, Ord, Hash, Pointer, Debug, Clone, Copy are implemented for all function pointers, higher-ranked ones included (built-in `FnPtr`, 1.70).
- Same page: "comparing function pointers is unreliable".
- Lint `unpredictable_function_pointer_comparisons`, warn by default since 1.85; fires inside `#[derive(PartialEq)]` since 1.89 (rust-lang/rust#134536).
- Local check on rustc 1.92: `#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] struct DefFn { on_play: fn(&mut Game) }` compiles with the lint warning (an error under `-D warnings`), and `Debug` prints an address (`DefFn { on_play: 0x100650cc0 }`). `Box<dyn Fn(&mut Game)>` fails to derive Debug, Clone and PartialEq (E0277, E0277, E0369).
