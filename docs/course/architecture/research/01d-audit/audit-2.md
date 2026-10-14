# Audit 2: the rules crate against the file-structure convention

## Summary

17 findings, 11 of high confidence. One is a live bug with current cards: a minion that dies in a state-check pass hears the deaths of older minions in that pass (finding 1).
Three more are latent bugs: a new card kind panics the state check (3), validation misses triggers in two places (4), and a reused pick id reuses the first pick (5). I confirmed all four in a throwaway copy of the crate under `/tmp`. The repo was not changed.
The root cause is structural. By the doc's Rust section, `game` is one module that holds about fifteen decisions (2). Its code interprets the card vocabulary through `_` arms and boolean chains (3, 8, 9).
18 module or file names fail, and the doc left 11 cases I could not decide.

## Modules

I counted a `mod` as a module only when its private items hold the data its decision is stored in. A child file that adds `impl Game` belongs to `game`.

```
rules (lib.rs)       root: the public API list
  rng                module: Rng.state is private
  turn               module: TurnOrder fields are private
  zones              modules: the inner containers of Deck, Hand, Board and Graveyard are private; Zones is a public record
  ids                module: PlayerId.0 is private
  observer           module: Views.0 is private
  cards              parent with no code
    binder           module: Binder.defs is private
    definition       module for DefId only (DefId.0 is private); CardDef and its parts are public data
    object           module: ObjectId.0 and the ObjectBag fields are private; Object is crate-public data
    modifier         module: Modifiers.0 is private; Modifier and ModifierEffect are public data
    filter           data file: public filter types and fulfill_side_filter
    loader           data file: the card set, load-time validation, static_card_definition ids
  game               module: Game fields are private. Every child below is part of it.
    condition, lookup, resolve, state_check, triggering, zone_move    impl Game only
    player           Player and PlayerInteractionState, pub(super) fields, so nothing is hidden from game
    view             impl Game, plus public View types
  action, choice, effect, event, history, outcome, trigger    data files, every field public
tests/spec           separate crate: topic files, plus support, model and playout helpers
```

What each module holds:

- `rng`: the PRNG algorithm, and how `below` and `shuffle` derive from it.
- `turn`: seat order, who is active, the turn count.
- `zones`: the order inside each zone. The deck is top first, the hand oldest first, the board left to right.
- `ids`: the PlayerId representation, an index into `Game.players`.
- `observer`: an observer reads nothing but views at a checkpoint.
- `cards::binder`: how definitions are stored and found, and what each card kind means for health, attack, board presence, the graveyard and auras.
- `cards::definition`: the DefId representation.
- `cards::object`: object id allocation, object storage, the order-of-play counter.
- `cards::modifier`: modifiers are kept as a list.
- `game`: setup and the opening deal; turn start; legality; draft progress; applying actions; the resolution loop and stack order; effect interpretation; selectors and random choice; recording and counting history; deriving and matching triggers; deriving stats and aura reach; zone moves and order stamps; the state check and the outcome; damage and fatigue; mana; views.
- Data files: their layouts are interface data that any reader can copy. They hold the action vocabulary (ADR 0002), the event vocabulary (ADR 0006), the effect, trigger, choice and history vocabularies (ADR 0004), the filter vocabulary with the side rule, and the card set with its ids.

## Decisions likely to change

| Decision | Evidence |
| --- | --- |
| Card kinds and what each implies: presence, stats, graveyard, character, aura receiver | Hearthstone weapons, locations and hero cards. MTG artifacts and lands. LoR landmarks and equipment. Commit 5273105 added Hero and had to edit binder, lookup, resolve, state_check and zone_move. |
| Which objects are in play: listen, can be acted on, have stats, die | Hearthstone weapons, locations and secrets are in play but not on the board. Hearthstone Corrupt triggers from the hand. MTG cards act from the graveyard. |
| How a stat derives from the printed value, modifiers and damage | Hearthstone "set attack to", doubling, silence, "costs (1)". ADR 0005 lists the alternatives it rejected. |
| What each modifier variant means | Hearthstone keywords such as Taunt, Divine Shield and Windfury are enchantments. MTG counters. |
| Aura reach | Hearthstone adjacency auras (Dire Wolf Alpha), cost auras on the hand (Sorcerer's Apprentice), hero auras. |
| Trigger patterns and how they match | Hearthstone start and end of turn, on summon, on heal, on attack. MTG enter and upkeep triggers. |
| Death pass order (ADR 0012) | ADR 0012 compares six games that differ. The choice is recent. |
| Pending-effect order (ADR 0014, 0015) | ADR 0013 was superseded by 0014 within one session. Hearthstone sorts by order of play, LoR by board order. |
| Order of play: when the stamp is set, what it sorts | ADR 0012 and 0014 depend on it. The comment at `cards/object.rs:22` and ADR 0012 define it differently. |
| Draft rules: choice order, count bound, whether a draft can finish | ADR 0007 and 0009 were superseded. `AtMost` and `AtLeast` are commented out at `choice.rs:18-19`. Hearthstone Choose One and Discover. MTG "up to N targets". |
| Candidates for choose, random and all | Hearthstone Stealth and Elusive, MTG hexproof: such a character can't be chosen but is still hit by "all" and random effects. |
| Where a resolution keeps its choices and picks | ADR 0015 rejected saving only the work tied to the answer. Hearthstone Discover chains. |
| Which events history records and what a query counts | Hearthstone counts spells cast this game, minions dead this game, cards played this turn (Combo), Elementals played last turn. MTG storm. LoR "if an ally died this round". |
| How damage is marked | Hearthstone armor, which also absorbs fatigue. Hearthstone Immune and Divine Shield. LoR Barrier. |
| Mana: affordability, payment, growth | Hearthstone Coin, Overload, Wild Growth. LoR spell mana, banked for spells only. |
| Where card definitions come from | The TODO at `cards/loader.rs:89`. ADR 0003 and 0004 speak of cards loaded from a file. One place knows it (`game.rs:60`), so there is no finding. |
| Opening deal and first player | Hearthstone: the second player draws 4 and gets the Coin, and both mulligan. One place knows it (`game.rs:96-106`), so there is no finding. |
| Fatigue amount | Hearthstone fatigue grows by one per draw. One place knows it (`game/zone_move.rs:38`), so there is no finding. |
| Side: friendly means the same player | Teams: MTG Two-Headed Giant, Hearthstone Battlegrounds Duos. Weak evidence for a 1v1 core. |

Decisions I judged stable: the RNG (ADR 0001), versioned DefIds (ADR 0003), the ObjectId counter (ADR 0003), seat order, the order inside each zone, the event vocabulary and who sees what (ADR 0006, 0010), legality as one function (ADR 0002), behavior as data (ADR 0004), and the outcome rule for two players.

## Findings

### 1. The dead are reported one at a time, so a dying minion hears older deaths in its pass

- Decision: the death pass order in ADR 0012. Read each dead object's own death triggers, remove all the dead, then report each death in order of play.
- Places:
  - `game/state_check.rs:24-56` (game): finds the dead, sorts them, hands them to `kill`. Informal for a change to the order.
  - `game/zone_move.rs:94-113` (game): `kill` pushes every own death trigger, then removes and reports one object at a time (105-112). Informal: the loop compiles whether it removes all first or not.
  - `game/triggering.rs:17-20` (game): `emit` matches whatever is on the board at that moment. Informal: it relies on its caller having removed every dead object.
  - `tests/spec/triggers.rs:96-119` pins only the case where the Mourner is the older of the two.
- Rule: "An order that is a rule of the domain is a decision, and gets a module like any other." This order has no home. It is split over three files of `game`, and the part that carries it is named as a zone move.
- Fix: one command in `state_check` runs the whole pass: read all own death triggers, remove all the dead, then report each in order of play. `kill` has one caller (`game/state_check.rs:43`) and moves there.
- Confidence: high.
- Bug: live. I confirmed it in the throwaway copy. Board [Recruit, Mourner], a Ping brings the Mourner to 1, then Blast. The events were Died(Recruit), Died(Mourner), Drew. The Mourner drew for the Recruit's death, against ADR 0012: "a removed object hears no other death".

### 2. `game` holds most of the crate's decisions in one module

- Decision: about fifteen, listed under Modules.
- Places: `game.rs:32-43` declares the private fields. Every child that adds `impl Game` reads them directly: `condition.rs` (players, objects, binder), `lookup.rs` (objects, binder), `resolve.rs` (players, objects, binder, history, effect_stack, resolving_effect, turn_order, rng), `state_check.rs` (objects, outcome), `triggering.rs` (objects, history, effect_stack, turn_order), `zone_move.rs` (objects, binder, effect_stack), `view.rs` (players, turn_order, outcome). `player.rs:27-34` fields are pub(super). All of this is informal for a change to any one decision, because the build can't sort the code by decision.
- Evidence that the decisions change apart: each feature touched most of the files. 8bcdccc (draft) touched `game.rs`, condition, player, resolve and view. ae79835 (triggers) touched `game.rs`, condition, lookup, resolve, state_check, triggering and zone_move. 995b2bb (resolution loop) touched `game.rs`, lookup, player, resolve, triggering, view and zone_move.
- Rule: "A boundary that holds many decisions that change independently doesn't get this, since its code has to be sorted by decision before a change." Also: "Judge a parent by its own code, which should hold one decision, often how its children combine." The files look like modules, but the Rust section says "child files that add `impl` blocks to their parent's type are part of the parent's module".
- Fix: for each decision that changes alone (draft, history, mana, effect stack, death pass), declare a type with private data in a child, so the child becomes a module. Leave `game.rs` with `apply`, `legal_actions` and the order it calls things in.
- Confidence: high. Not a bug.

### 3. Binder answers card kinds, then `game` derives them again

- Decision: what each card kind implies.
- Places:
  - `cards/binder.rs:34-48, 62-100` (cards::binder): six exhaustive matches. Formal for a new kind.
  - `cards/binder.rs:50-60` (cards::binder): `is_spell`, `is_minion`, `is_hero`. Formal per kind, but callers combine them.
  - `game/lookup.rs:56-65` (game): health exists for a hero in its hero zone or a minion on a board. Informal for a new kind.
  - `game/lookup.rs:158` (game): only minions receive auras. Informal.
  - `game/condition.rs:90-105` (game): the kind filters are `&&` and `||` chains. Informal for a new kind and for a new filter variant.
  - `cards/filter.rs:6-22` (cards::filter): two more lists of kinds. Nothing ties them to `CardDefKind`.
  - `game/zone_move.rs:106-110` (game): a dead hero stays in its zone and anything else is destroyed. Informal.
  - `game/zone_move.rs:127` (game): asks `has_deck_presence` to decide whether the object goes to the graveyard. It asks a question other than the one the function is named for. Nothing asks whether a card may be in a deck, since `game.rs:74-79` accepts any DefId.
  - `game/state_check.rs:9-22, 29-36` (game): minions die from the boards, heroes from the players still playing. Informal.
- Rule: "Knowledge is informal for a change when the build still passes after it ... Informal knowledge in the module that hides the decision is cheap. In any other module, make it formal." Also: "Prefer taking knowledge away from callers to making it formal."
- Fix: binder answers each kind question that `game` asks with an exhaustive match: has health in play, receives auras, stays in its zone on death, goes to the graveyard, matches a kind filter. `game` stops combining `is_hero` and `is_minion` with zone checks.
- Confidence: high.
- Bug: latent. In the copy, adding `CardDefKind::Location` failed the build only at the six binder matches. Once those had arms, a Location played to the board panicked at `game/state_check.rs:15` ("board objects always have health"), because `game/lookup.rs:56-65` knows only heroes and minions.

### 4. Validation walks a card's effects in two copies, and both miss two places

- Decision: where a card keeps the effect sequences it can run.
- Places:
  - `cards/definition.rs:24-33, 57-63` (data): play effects, triggers, modifier effects, friendly and hostile aura effects. In `effect.rs:31-38` (data), Buff and AddFriendlyAura carry a ModifierEffect, and a GrantTrigger holds effects.
  - `cards/loader.rs:645-672` (cards::loader): `validate_choice_on_trigger` walks the triggers and the aura GrantTriggers. Informal.
  - `cards/loader.rs:678-698` (cards::loader): `validate_picks` walks the play effects, triggers and aura GrantTriggers. Informal, and a copy of the walk above.
  - `game/lookup.rs:119-176` (game): at runtime, an object's triggers include every GrantTrigger in its modifiers. Those include `modifier_effects` (147-148), auras, and buffs added by effects (`game/resolve.rs:195-214`).
  - Both walks skip `modifier_effects`, and both skip GrantTriggers nested inside Buff and AddFriendlyAura.
- Rule: "When no plausible change would make two answers differ, they are one question asked twice, and the two places are a copy."
- Fix: one function on CardDef in `cards::definition` yields every effect sequence the card can run, and whether it runs as a trigger. Both validators iterate it, so a new place to keep effects is added once.
- Confidence: high.
- Bug: latent. No current card puts a trigger in `modifier_effects` or inside an effect. In the copy, three test cards passed `load_and_validate`. A minion with a deathrattle GrantTrigger in `modifier_effects` that reveals without a ResolvePick left both revealed cards in no zone after the pick. A minion that buffs itself with a deathrattle naming `Chosen(ChoiceId(7))` panicked at `game/resolve.rs:366` ("Choice not fulfilled") when it died.

### 5. A pick is stored on its source object and found by first match

- Decision: where the choices and picks made during a resolution are kept, and for how long.
- Places:
  - `cards/object.rs:20-21` (cards::object, crate-public fields) and `choice.rs:48-62` (data): the storage.
  - `game/zone_move.rs:202-207` (game): `pick` appends a PickTarget. Informal.
  - `game/resolve.rs:171-177` (game): ResolvePick takes the first PickTarget with that pick id and never removes it. Informal: it assumes a source resolves each pick id once.
  - `cards/loader.rs:700-723` (cards::loader): `validate_pick_order` accepts a pick id reused after it is resolved. The validator and the resolver disagree on the rule.
  - `game/resolve.rs:84-95, 360-368` (game): choice targets are written at Play and read by Chosen. Informal, same storage.
- Rule: "Data a module takes in or hands out is part of its interface: its layout, what each field means and the rules for writing it. A decision whose data crosses the boundary isn't hidden, because any caller can copy it."
- Fix: keep picks and choices with the unit being resolved, in private fields of `ResolvingEffect`, with commands to record and take a pick. They then end with the unit. ADR 0015 already says the unit holds "its position and its context".
- Confidence: high for the bug, medium for the home.
- Bug: latent. In the copy, I gave a spell Forage's two effects twice, both with `PickId(0)`, and validation accepted it. After both picks, the first Bolt picked was in the hand twice, the first Blast buried was in the deck twice, and both cards of the second reveal were in no zone. A minion whose trigger reveals each time it is damaged would take the same path.

### 6. One function answers "may this choice stop" and "may it take more"

- Decision: when a choice is complete (the count and bound in ADR 0010).
- Places:
  - `choice.rs:22-33` (data): `is_satisfied` and `upper_bound`. `upper_bound` has no caller.
  - `game/condition.rs:35-45` (game): `check_choice_count_fulfilled` asks `is_satisfied`.
  - `game/condition.rs:23` (game): stops offering candidates once the choice is satisfied. This asks "may it take more".
  - `game/condition.rs:118-122` and `game.rs:168-174` (game): skip a satisfied choice, and offer Play once all are satisfied. This asks "may it stop".
  - Informal for the planned bounds. `AtMost` and `AtLeast` are commented out at `choice.rs:18-19`. Once each gets an arm, the build passes.
- Rule: "Give each question its own function, even when two share an answer today. The second can be built on the first. The day their answers differ, each changes alone."
- Fix: two questions, may stop (built on `is_satisfied`) and may add (built on `upper_bound`). Candidate scans ask may add. Offering Play asks may stop.
- Confidence: high.
- Bug: not today. In the copy, I added `AtMost` and a card that chooses up to two minions, then faced two minions. After Draft, the actions were Play and Cancel. No Choose was offered.

### 7. The draft walk is written twice

- Decision: which choice the next Choose fills (ADR 0010: the first unfilled choice in declaration order), and whether a draft can still finish.
- Places:
  - `game.rs:157-198` (game): `legal_actions` walks the choices in order, counts the targets chosen for each, skips the filled ones, offers candidates that keep the draft finishable, and stops at the first unfilled one. Informal for a change to the order.
  - `game/condition.rs:107-137` (game): `can_finish_choices` repeats the same walk. Informal.
  - The count of targets per choice is repeated at `game.rs:164-167`, `game/condition.rs:18-22` and `game/condition.rs:114-117`. `game/resolve.rs:86-94` groups the targets by choice. Informal.
  - `game/player.rs:20-23` (game): the draft is a bare `Vec<(ObjectId, ChoiceId)>`, read by all of these and by `game/view.rs:171`.
- Evidence: ADR 0007 and 0009 were superseded. "Choices filled in any order" is a considered option in ADR 0010.
- Rule: "Ideally one place knows it, and every other place asks it a stable question or gives it a stable command, by name."
- Fix: a Draft type in a child of `game`, with the chosen targets private, and questions such as next choice and targets for a choice. `legal_actions` and `can_finish_choices` both ask next choice.
- Confidence: high. Not a bug.

### 8. Four `_` filters decide what each ModifierEffect means

- Places:
  - `cards/modifier.rs:31-36` (data): the variants.
  - `game/lookup.rs:38-41` (cost), `:76-79` (health), `:102-105` (attack) and `:130-137` (granted triggers) (game): each picks one variant and drops the rest with a `_` arm. Informal for a new variant.
  - `cards/loader.rs:663, 691` (cards::loader): `if let ModifierEffect::GrantTrigger`. Informal.
- Rule: "When several filters together decide what each variant means, give that decision one exhaustive match." Also: "In any other module, make it formal."
- Fix: one exhaustive match over ModifierEffect, for example in `cards::modifier`. It folds a list of modifiers into attack, health, cost reduction and granted triggers, and `lookup` reads the result.
- Confidence: high.
- Bug: no. In the copy, adding `ModifierEffect::SetAttack` built with no error or warning, and the variant would do nothing.

### 9. Trigger matching ends in a catch-all `(_, _)` arm

- Places: `trigger.rs:15-19` (data) declares the patterns. `game/triggering.rs:39-62` (game) decides for every pattern whether an event matches it, with `_` at line 60. Informal for a new pattern. `game/lookup.rs:115` is a `matches!` filter for own death triggers, which the doc allows.
- Rule: "An exhaustive `match` with no `_` arm, so adding a variant fails the build at every place that has to handle it. This is for matches that decide something for every variant."
- Fix: match the pattern exhaustively, and test the event inside each arm.
- Confidence: high.
- Bug: no. In the copy, adding `TriggerPattern::Summoned` built clean. It would never match.

### 10. One character scan answers three questions

- Decision: which characters an effect may act on.
- Places: `game/condition.rs:47-67` (game), `scan_characters_with_filter`, lists heroes in seat order, then each board left to right. It answers:
  - `game/resolve.rs:351`: what an All selector hits, and in what order (pinned at `tests/spec/events.rs:307`).
  - `game/resolve.rs:354-358, 372-377`: the candidates of a Random selector. The list order feeds the rng index.
  - `game/condition.rs:28`: the candidates a draft choice offers.
  - Each caller relies on the other answers staying equal to its own. Informal.
- Evidence: Hearthstone Stealth and Elusive, MTG hexproof. Such a character can't be chosen but is still hit by "all" and random effects.
- Rule: "Give each question its own function, even when two share an answer today. The second can be built on the first."
- Fix: three functions (all targets, random candidates, choice candidates) built on one list of characters in play. A change to hit order would then leave the random picks for a seed alone.
- Confidence: high. Not a bug.

### 11. History is recorded in one file, counted in another, and stored as public data

- Places:
  - `history.rs:3-46` (data): every field is public, `History.entries` included.
  - `game/triggering.rs:64-98` (game): which events become entries, with a `_` arm at line 94. Informal for a new query.
  - `game/resolve.rs:285-320` (game): how a query counts. The player and turn filters are copied at 293-300 and 308-315. Formal for a new HistoryQueryKind (exhaustive at 292), but informal for which entries the new kind needs.
- Change: in the copy, adding `HistoryQueryKind::CardsDrawn` failed the build only at `game/resolve.rs:292`. Nothing pointed to the recorder, which records no draws.
- Rule: "Make a decision its own module when its knowledge sits in more than one place, today or in planned work, and an interface of questions and commands can replace that knowledge while asking callers to know less than the data would."
- Fix: History keeps its entries private and offers record (an event with its turn) and count (a query for a player). The turn and player filters are written once.
- Confidence: high. Not a bug.

### 12. "In play" is computed in five places

- Decision: which objects are in play, and so listen for events, can be acted on, have stats, give or receive auras, and can die.
- Places (all in game, all informal for a new in-play zone or kind):
  - `game.rs:216-227`: the listeners, heroes and boards in order of play.
  - `game/condition.rs:47-67`: the characters to act on, heroes and boards in seat and board order.
  - `game/lookup.rs:56-65, 89-92`: health only for a hero in its zone or a minion on a board, attack only on a board.
  - `game/lookup.rs:158-160`: aura givers are board objects. Receivers are any minion, by kind, in any zone.
  - `game/state_check.rs:11-12, 29-34`: the dead are board minions and the heroes of players still playing.
  - Each walks the zones itself.
- Rule: "Ideally one place knows it, and every other place asks it a stable question", and "The second can be built on the first."
- Fix: one function lists the objects in play in order of play. Listeners, targets, aura reach and the death scan build on it.
- Confidence: medium.
- Bug: not confirmed as one. In the copy, a friendly aura that reduces cost lowered a minion's cost in hand from 2 to 1 and left a spell at 2. Nothing says whether an aura reaches the hand. The glossary defines an aura as a buff to attack or health.

### 13. Pending-effect order lives in push sites and call order

- Decision: the order in which pending effects resolve (ADR 0014, 0015).
- Places (all in game, all informal for a change to the order):
  - `game.rs:41-42`: a bare Vec and an Option.
  - `game/zone_move.rs:63-74`: play effects are pushed before Played is reported, so reactions resolve first.
  - `game/zone_move.rs:95-103`: own death triggers are pushed before Died is reported.
  - `game/triggering.rs:17-20, 29-36`: matched triggers are pushed in order of play.
  - `game/lookup.rs:140`: one object's triggers are sorted by their source's stamp.
  - `game/resolve.rs:225-243`: pops the newest, and runs the state check between units.
- Rule: "An order that is a rule of the domain is a decision, and gets a module like any other."
- Fix: an effect stack type with a private Vec in `effect.rs`, whose commands name the moments the order depends on: push a unit with the reactions to its event, take the next unit.
- Confidence: medium (see doc feedback 10). Not a bug.

### 14. Every zone move must remember to restamp order of play

- Decision: when an object's order-of-play stamp is set.
- Places:
  - `cards/object.rs:22-23, 40-44, 75-79` (cards::object): the counter and `update_order`. The field comment says the stamp changes on every zone move. ADR 0012 says it is set at board entry.
  - `game/zone_move.rs:35, 138, 224, 240, 254` (game): draw, entomb, add to hand, bury and board entry each call `update_order` after moving the object. Reveal (153-164) does not. Draw stamps after reporting, the others before. Informal: a new move that forgets the call compiles.
  - `game.rs:225`, `game/lookup.rs:140`, `game/state_check.rs:42` (game): each sorts by the public `order` field itself. Informal.
- Rule: "A caller that must call two functions in order still knows the decision, even when a type enforces it. One command that does both leaves the caller nothing to get wrong."
- Fix: stamp inside the one command that puts an object on the board. Callers ask for ids in order of play instead of sorting by the field.
- Confidence: medium. Not a bug today. Only the board-entry stamp reaches a reader.

### 15. Damage is marked in two places

- Places: `game/zone_move.rs:37-46` (fatigue) and `game/zone_move.rs:262-277` (damage), both in game, each do `object.damage +=` and report different events. `cards/object.rs:17` (cards::object) is a public field. Informal: armor, Immune or a shield would need both paths, and the build finds neither.
- Rule: "When no plausible change would make two answers differ, they are one question asked twice, and the two places are a copy."
- Fix: fatigue calls the same command that marks damage, and reports its own event as well.
- Confidence: medium.
- Bug: no. A Damaged trigger never matches fatigue. No current card gives a hero a trigger, so no card reaches this, and no doc says which is right.

### 16. Mana is raw fields, checked and paid in different places

- Places (all in game, all informal for a change to how mana works):
  - `game/player.rs:29-30`: mana and max mana as `u8`, pub(super).
  - `game.rs:124-130`: a card is affordable when its cost is at most the mana.
  - `game/resolve.rs:98-101`: paying subtracts the cost.
  - `game.rs:260-262`: turn start grows max mana by one up to 10, then refills.
  - `game/view.rs:92-93`: shows both values.
- Rule: "Make a decision its own module when its knowledge sits in more than one place, today or in planned work".
- Fix: a Mana type with private fields and the commands can pay, pay and start turn, plus the two values the view shows.
- Confidence: medium. Not a bug.

### 17. The side rule is written twice

- Places: `cards/filter.rs:45-48` (cards::filter), `fulfill_side_filter`: friendly means the same player. `game/lookup.rs:165-169` (game) picks the friendly or hostile aura by `obj.player_id == other_player_id`. Informal.
- Rule: "they are one question asked twice, and the two places are a copy."
- Fix: `lookup` asks a side question that `cards::filter` answers.
- Confidence: low, because teams are unlikely in this core.

## Names

Names that fail:

| Name | Why it fails | Real function |
| --- | --- | --- |
| `game` | Any function in the crate fits it. It doesn't rule out what its siblings hold. | `Game::history_query` (`game/resolve.rs:285`) belongs to sibling `history`'s decision. `Game::fulfill_trigger_pattern` (`game/triggering.rs:39`) belongs to `trigger`'s. |
| `game::condition` | Any predicate fits it, and the glossary has no such word. | `Game::fulfill_trigger_pattern` (`game/triggering.rs:39`) is a condition in a sibling. `Game::can_finish_choices` (`game/condition.rs:107`) is a draft rule. |
| `game::lookup` | Any read fits it. | `Game::mana` (`game.rs:229`) and `Game::board_characters` (`game.rs:216`) are reads outside it. `Game::triggers` (`game/lookup.rs:119`) fits sibling `triggering`. |
| `game::resolve` | The glossary says "Resolve: To carry out a card's text." | `Game::damage` (`game/zone_move.rs:262`) carries out text from a sibling. `Game::history_query` (`game/resolve.rs:285`) and the EndTurn arm (`game/resolve.rs:108-112`) carry out none. |
| `game::triggering` | Holds event reporting and history recording too. | `Game::history_from_event` (`game/triggering.rs:64`) doesn't fit. `Game::death_rattles` and `Game::triggers` (`game/lookup.rs:112-142`) fit but sit in `lookup`. |
| `game::zone_move` | Holds things that move nothing. | `Game::damage` (`game/zone_move.rs:262`) and `Game::pick` (`game/zone_move.rs:181`). `Game::kill` (`game/zone_move.rs:94`) is part of the state check. |
| `cards` | Card text is split between `cards` and its siblings `effect`, `trigger` and `choice`. A reader can't tell where a new card-text helper goes. | `CharacterFilter::all` (`cards/filter.rs:37`) is under `cards`. `AmountBound::is_satisfied` (`choice.rs:23`) reads a choice declared on a card and sits outside. |
| `cards::definition` | Holds the selectors (the glossary says a selector is part of an effect) and a filter. | `PlayerSelectorFilter::all` (`cards/definition.rs:96`) fits sibling `filter`, next to `CharacterFilter::all` (`cards/filter.rs:37`). |
| `cards::loader::static_card_definition` | Holds DefId constants, no definitions. It has no functions to test on. | `BOLT` (`cards/loader.rs:66`) is an id. Bolt's definition is in `CardDefLoader::load_all` (`cards/loader.rs:110-123`). |
| `choice` | The glossary separates Choose from Pick, yet PickId and PickTarget live here. EffectAmount is the amount of every effect. | `impl From<u8> for EffectAmount` (`choice.rs:9`) serves every `amount: 1.into()` in `cards/loader.rs`. |
| `ids` | Holds PlayerId only. ObjectId, DefId, ChoiceId and PickId are elsewhere. | `ObjectBag::next_id` (`cards/object.rs:34`) mints ids and fits the name. |
| `turn` | Holds seat order, as its own doc says (glossary: Seat). The name invites turn-start rules, which change apart from seat order. | `Game::start_turn` (`game.rs:254`), with its mana growth and draw, fits the name and sits in `game.rs`. |
| `cards::modifier` | Not in the glossary. The glossary's Buff and Aura cover attack and health, but ModifierEffect also holds ReduceManaCost and GrantTrigger (`cards/modifier.rs:34-35`). Fails "Take names from the glossary". It excludes well otherwise. | `Game::friendly_aura` (`game/lookup.rs:183`) builds Modifiers from an aura. |
| `history` | Not in the glossary, though ADR 0005 uses the word. Fails "Take names from the glossary". | `Game::history_query` (`game/resolve.rs:285`) and `Game::history_from_event` (`game/triggering.rs:64`) are its functions and live in `game`. |
| `tests/spec/cards.rs` | Most tests are about some card. | `ping_deals_2_damage_to_the_chosen_minion_only` (`drafts.rs:222`) and `medics_deathrattle_gives_its_surviving_friendly_minions_2_health` (`triggers.rs:36`) fit it and sit in siblings. |
| `tests/spec/lookups.rs` | Any accessor test fits it. | `cards_in_hand_have_a_cost_and_no_stats` (`derived.rs:55`) is a lookup. `a_dead_minion_has_no_cost_attack_or_health` (`lookups.rs:35`) fits `derived`. |
| `tests/spec/resolution.rs` | Every card resolves. The file is about pending picks (glossary: Pending pick). | `bombers_deathrattle_deals_1_damage_to_every_minion` (`triggers.rs:226`) fits it. |
| `tests/spec/properties.rs` | Overlaps sibling `playout`. Both hold invariants over random playouts. | `assert_outcome_consistent` (`playout.rs:180`) and `hidden_cards_show_only_to_whoever_may_see_them` (`properties.rs:121`) could swap files. |

Names that pass: `rng`, `observer`, `outcome`, `event`, `action`, `effect`, `trigger`, `zones`, `cards::binder`, `cards::filter`, `cards::object`, `game::player`, `game::state_check`, `game::view`, the test files `drafts`, `derived`, `events`, `legality`, `setup`, `state_check`, `triggers`, `turns`, `views`, `support`, `model` and `playout`, and `benches/game.rs`. `cards::loader` passes for its functions. It also holds the card set, which will need its own home once loading moves to the shell.

## Already following the rules

- `rng`, `turn`, `zones`, `ids`, ObjectId and DefId keep their representation behind private fields. Callers ask by name: top, bottom, active player, next id.
- Binder keeps its storage private, and callers ask it questions. Its kind matches are exhaustive. `Binder::get` is pub(crate) but has no caller outside binder.
- `observer::Views` offers one question. `clippy.toml` bans `Observer::event` outside `emit`. That is the doc's "only this place may call that".
- `emit` is one command that records history, reports the event and matches triggers, so callers can't get the order wrong.
- `apply` runs the legality check and the mutation as one command. `apply_action` is pub(super) and relies on it.
- `Game::view` is the one place that decides what each player sees.
- The effect interpreters have no `_` arm: `game/resolve.rs:147-222`, `cards/loader.rs:626-643` and `cards/loader.rs:700-723`. So do the HistoryQueryKind match (`game/resolve.rs:292`) and `legal_actions` over PlayerInteractionState (`game.rs:116-199`).
- Some decisions likely to change sit in one place each, which the doc says to leave alone: the card source (`game.rs:60`), the opening deal (`game.rs:102-106`), the fatigue amount (`game/zone_move.rs:38`).
- `clippy.toml` bans HashMap, HashSet and the clock in one place, for determinism.

## Doc feedback

1. "A module in the general sense is a `mod` whose private items include the data its decision is stored in." Most crate-level files hold only public data: `action`, `choice`, `effect`, `event`, `history`, `outcome`, `trigger`, `cards::filter`, most of `cards::definition`. The doc doesn't say what these are, or whether "Names exclude" applies to them. I judged their names anyway and treated their layouts as interface data.
2. "a decision hides from the parent only in a type declared in a child." Private methods in a child's `impl Game` (`fn modifiers` in `lookup.rs`, `fn find_dead` in `state_check.rs`, `fn drain_effect_queue` in `resolve.rs`) are hidden from `game.rs` by the compiler. The doc counts data only. I followed it, but it could say whether hidden behavior counts for anything.
3. "One place knows each decision" and "when its knowledge sits in more than one place". The doc never defines a place: a module, a file, or a function. Inside one module, informal knowledge is "cheap". So places inside `game` count only through the "many decisions" sentence. Findings 7, 12, 13, 14 and 16 depend on this reading. I counted functions as places.
4. "A boundary that holds many decisions that change independently doesn't get this". There is no threshold for "many", no test for "independently", and no word on whether each copy inside such a boundary is its own finding. I reported the boundary once (finding 2) and the copies separately.
5. "Ideally one place knows it" against "In any other module, make it formal." ADR 0004 accepts one exhaustive match per interpreter (`game/resolve.rs:147`, `cards/loader.rs:626`, `cards/loader.rs:703`). Each is formal, and each is still a place that costs an edit. The doc doesn't say when several formal places become a break. I treated them as compliant.
6. "A test shows that a change broke something, not where the decision is known, so it doesn't reduce the places that know it." `tests/spec/model.rs` repeats every printed cost and stat, and the Captain rule, on purpose as an oracle. The doc doesn't say whether test code is held to these rules, or whether an oracle counts as a place that knows the decision.
7. "what the roadmap plans". I could see only code comments (`choice.rs:18-19`, `cards/loader.rs:89`) and ADRs. The doc doesn't say whether an ADR's considered options or a superseded ADR count as planned work. I used them as evidence of variation. It also gives no bar for "likely", which decided the confidence of finding 17.
8. "Take names from the glossary". Technical names such as `rng`, `observer`, `loader`, `lookup` and `support` have no entry and aren't domain words. The doc doesn't say whether the bullet covers them. I applied it only to names of domain concepts (`history`, `modifier`).
9. "Judge a name against its siblings, the other names a reader picks between at the same level." Some confusion crosses levels: `trigger` against `game::triggering`, and `turn` against `Game::start_turn` in `game.rs`. The doc doesn't say whether that is a name failure. I failed `turn` because the function its name invites sits elsewhere.
10. "An order that is a rule of the domain is a decision, and gets a module like any other." The stack rule (ADR 0014) is written as "push own effects, then report the event, so the reactions land on top". A stack type can't hold "before the event" unless it also owns reporting. The doc doesn't say how to give a module an order that callers express as call order around a third module. My fix for finding 13 is weaker for it.
11. "A name passes when a reader, given any function, existing or new, can tell from the name alone whether it belongs." This tests the name, not the placement. `kill` fits `state_check` but sits in `zone_move`, and `fulfill_character_filter` fits `cards::filter` but sits in `game::condition`. I counted such a misfit against the file that holds it, and only when its name invited the misfit or failed to rule it out. The doc could say which file a misplaced function counts against.
