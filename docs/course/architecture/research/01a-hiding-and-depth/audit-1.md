# Audit 1: the rules crate against the file-structure convention

## Summary

- Ten findings. The strongest: trigger order is spread over five files (F1, F2), the death matcher depends on a call order that has already broken (F3, reproduced), and the draft rules are copied and merge two questions (F5).
- Most violations sit in `game/`, where eight files extend `impl Game` and share every private field, so "module boundary" is unclear there.
- Reporting, visibility, side filters and ids already follow the rules.
- Ten doc feedback items, mostly on what "module", "place" and "home" mean.

## Decisions likely to change

| Decision | Evidence |
|---|---|
| Effect stack order | ADR 0013 superseded by 0014; drain rewritten in #7, moved to `resolve.rs` in #8 |
| Play-order stamp: what moves it, what it orders | Added in #7; ADR 0012 says "set at each entry", the #7 commit says it "moves on every zone change" |
| What the death matcher sees | ADR 0012, after research across ten games; 89b6131 moved deathrattles after the death; `remove_dead` rewritten in #7 |
| Step ends: state check and checkpoints | ADR 0012, 0015; 89b6131 reordered board entry; Drillmaster step test added in #7 |
| Draft rules | ADR 0007, 0009, 0010 (two supersessions); `AtMost`, `AtLeast` commented out at `choice.rs:18-19` |
| Where a paused unit's context lives | ADR 0015 ("its position and its context"); `pick_targets` added in #8 |
| Card shape: kinds, where effect sequences live | Hero kind in #4 (ADR 0008); triggers and `GrantTrigger` in #7; `ResolvePick` in #8; ADR 0004 expects growth |
| History records and queries | ADR 0005; "feat: history", "feat: giant"; recording moved into `emit` in #7 |
| Card source and validation | TODO at `loader.rs:89`; "may explode" at `game.rs:59`; ADR 0004 expects loading from a file |
| What each player may see | ADR 0006; ADR 0009 superseded by 0010 |

## Findings

F marks formal knowledge, I informal. Each file counts as a module (doc feedback 1).

### F1. Effect stack order is known in five files

- Decision: newest first; a play's effects and a death's own triggers go under what they set off; one event's matches are pushed oldest first (ADR 0014).
- Known at: `game.rs:41`, a bare `Vec<PendingEffect>` any child module pushes to (I); `zone_move.rs:63-68`, play effects before `Played` (I); `zone_move.rs:96-103`, own death triggers before `Died` (I); `triggering.rs:34` (I); `game.rs:225` and `lookup.rs:140` sort oldest first so the newest pops first (I); `resolve.rs:233,239` (I).
- Breaks: one place knows each decision; informal across module boundaries.
- Home: an `EffectStack` in `effect.rs` with a private `Vec` and commands named after the ADR's cases (push a play, push one event's matches, push a death's own triggers, take the next).
- Confidence: high.

### F2. The play-order stamp relies on every mover remembering it

- Decision: when the stamp moves, and that smaller means older.
- Known at: `object.rs:23` `pub order: usize` and `object.rs:75-79` (F by name, but a bare integer); `zone_move.rs:35,138,224,240,254`, each move stamps on its own (I). `play` (`zone_move.rs:57-61`) and `reveal` (`153-163`) move objects without stamping, and `draw` stamps after `emit` (`34-35`) while the others stamp before. `game.rs:225`, `lookup.rs:140` and `state_check.rs:42` sort by the raw field (I).
- Breaks: a required call order (move, stamp, emit) across `zones.rs`, `cards/object.rs` and `game/zone_move.rs`; one place.
- Home: one move command that changes the zone, stamps and reports. One query in `object.rs` returns ids in play order, with the stamp behind a private newtype.
- Confidence: high.

### F3. What the death matcher sees depends on the caller's order

- Decision: the dead leave before their deaths are matched (ADR 0012).
- Known at: `state_check.rs:38-43` collects the pass and calls `kill` (F); `zone_move.rs:105-112` removes and reports one object at a time (I); `triggering.rs:17` matches against whatever is on the board when `emit` runs (I).
- It has drifted. A throwaway program outside the repo played Recruit, then Mourner, pinged the Mourner and cast Blast. It printed `Died` Recruit, `Died` Mourner, then `Drew` for the Mourner's owner. `triggers.rs:97` tests only an older Mourner.
- Breaks: a required call order across modules; split by step (collect, commit and match in three files).
- Home: one command owns the pass: read each death's own triggers, remove all, then report each `Died`.
- Confidence: high.

### F4. Checkpoint placement is spread over three files

- Decision: where one step ends.
- Known at: `resolve.rs:106,111,118,130,135,259`; `zone_move.rs:76,80`; `state_check.rs:51,78`. All informal, pinned by tests in `events.rs`.
- Breaks: one place knows each decision.
- Home: the resolution loop and `apply_action`, as coordinator, decide where steps end. A `disallowed_methods` ban on `Observer::checkpoint`, like `clippy.toml:16`, makes it formal.
- Confidence: medium. Play's two steps (`zone_move.rs:76,80`) may need play split first.

### F5. Draft rules are copied, and one function answers two questions

- Decision: fill the first open choice in declaration order; when a choice is done; when it takes more.
- Known at: `game.rs:163-189` and `condition.rs:113-134` repeat one loop with the same `break` (I); `condition.rs:18-25` recounts (I); the `chosen` layout (`player.rs:22`) is read at `game.rs:164-167`, `condition.rs:18-22,30,114-117`, `resolve.rs:89-93`, `view.rs:171` (I). `check_choice_count_fulfilled` (`condition.rs:35-45`) answers "may it take another target" at `condition.rs:23` and "is it done" at `game.rs:168` and `condition.rs:118`.
- Breaks: one place; give each question its own home. Under `AtMost` the answers differ; `upper_bound` (`choice.rs:29`) is unused.
- Home: a `Draft` type with a private `chosen`, answering `next_open_choice`, `accepts_more`, `is_complete`, `candidates` and `targets`, asked by `legal_actions`, `apply_action` and the view.
- Confidence: high.

### F6. A unit's context is handed forward on `Object`

- Decision: where chosen targets and pick results live until the effect that uses them.
- Known at: `object.rs:20-21` (F); `resolve.rs:84-95` writes `choice_targets` (I); `resolve.rs:360-368` reads them with `expect` (I); `player.rs:15-19` carries `source` and `pick_id` so `zone_move.rs:202-207` can write `pick_targets` (I); `resolve.rs:171-186` reads the first match (I).
- `choice_targets` is cleared on the next play, `pick_targets` never. By reading, a trigger that picks and fires twice on one object, which the loader accepts, would read its stale first entry. No current card does this.
- Breaks: don't split by step (readers know when the data was computed).
- Home: the resolving unit (`effect.rs:81`), as ADR 0015 says. Play builds it with its targets, a pick writes into it, and both go when it finishes.
- Confidence: medium.

### F7. Two validators walk the card shape by hand

- Decision: where a card definition holds effect sequences.
- Known at: `loader.rs:645-672` and `678-698` each walk `triggers`, then minion auras, through `if let` (I); `lookup.rs:119-142` at run time (F). Neither validator looks inside `Effect::Buff` or `Effect::AddFriendlyAura` (`effect.rs:31-38`), whose `ModifierEffect` can carry a `GrantTrigger`, so a choice or an unresolved pick nested there passes.
- Breaks: informal across the boundary between `loader.rs` and `definition.rs`, `modifier.rs`, `effect.rs`; one place.
- Home: one walker in `cards/definition.rs`, built from exhaustive matches, yields every sequence and whether it runs in a draft. Both validators ask it.
- Confidence: high.

### F8. Card kinds are mapped to rules at each call site

- Decision: what a kind implies: character or not, where it lives, what auras reach, where it goes when destroyed.
- Known at: `binder.rs:34-48,62-100` (F); `condition.rs:96-104` maps filter kinds to card kinds with `||` (I); `lookup.rs:56-63` and `158` (I); `zone_move.rs:107` (I); `zone_move.rs:127` asks `has_deck_presence` to mean "goes to the graveyard" (I). Characters in play are listed at both `game.rs:216-227` and `condition.rs:47-67`. A new kind compiles unchanged at every `game/` site.
- Breaks: informal across `cards/` and `game/`; give each question its own home.
- Home: `Binder` answers each rule by name with an exhaustive match (`is_character`, `kind_matches`, `goes_to_graveyard`); one `Game` query lists characters in play.
- Confidence: medium.

### F9. History is written in one module and read in another

- Known at: `history.rs:43-46` `pub entries` (F); `triggering.rs:64-98` maps events to entries with a catch-all arm, so a new `Event` needs no decision (I); `resolve.rs:285-320` counts, with the turn filter copied at `295-300` and `310-315` (I).
- Breaks: one place; informal across boundaries.
- Home: `history.rs` owns `record(&Event)`, with an exhaustive match, and `count(query)`.
- Confidence: medium.

### F10. Holding a `Binder` doesn't prove the cards were validated

- Known at: `game.rs:59-61` must pick `load_and_validate` over `load_all` (I); `load_all` (`loader.rs:94`) is exported (`lib.rs:30`); `binder.rs:19` accepts any `Vec<CardDef>` (I); `definition.rs:6,38` imports the loader's `PLACEHOLDER`.
- Breaks: a caller that must pick the right call still knows the decision.
- Home: the loader returns a validated table only it can construct, and `Binder::new` takes only that. Whether the shell supplies it (the TODO) changes `Game::new`, so it is your call.
- Confidence: medium.

## Already following the rules

- Reporting: `emit` is the only caller of `Observer::event`, enforced by `clippy.toml:16`.
- Visibility: `view.rs` decides every face (`view.rs:115,149`); moving from ADR 0009 to 0010 touched only `view.rs` (#6).
- Effect interpreters: `resolve.rs:147-222`, `loader.rs:626-643` and `700-723` match `Effect` with no `_` arm.
- Side: `fulfill_side_filter` (`filter.rs:45-48`) is the one home, called by name.
- Ids: `PlayerId::new` is `pub(super)` (`ids.rs:8`); only `ObjectBag` mints an `ObjectId` (`object.rs:10,34-38`).
- Legality: `apply` asks `legal_actions` (`resolve.rs:45`).
- Stable, easy decisions left in place: mana cap (`game.rs:261`), opening hand (`game.rs:103`), fatigue amount (`zone_move.rs:38`).

## Doc feedback

1. Module. "Informal knowledge inside one module is cheap, because whoever changes the decision is already editing that module." In `game/`, eight files each add `impl Game` and read every private field: one module by privacy, eight by editing. I took the editing reading, since the sentence's reason is about editing. The doc should say which.
2. Place. "A change to a decision costs one edit for each place that knows it". Five `update_order` calls in one file are five places for rule 2 and none for rule 1, which counts modules. I couldn't tell whether to count lines, functions or modules.
3. Home. "Give each question its own home" and "Merge the decisions that change together" pull apart in F8: the kind questions are separate but change together when a kind is added. I read "home" as a function inside one module; the doc never defines it.
4. Formal by name. "Formal knowledge is a reference, such as a name, a type or a match that lists every case." A call to `is_minion` is a name, yet `condition.rs:96-104` assumes heroes and minions are the only characters. I ruled the call formal and the `||` chain informal. The doc needs a test: formal only if changing the decision breaks the reference.
5. Test versus one place. "Where neither can, pin it with a test." Checkpoints (F4) are pinned by tests, satisfying rule 1, but known in three files, breaking rule 2. The doc doesn't say whether a test settles both.
6. Steps versus an ADR. "A step that checks what a later step will do has to know that step's rule." `legal_actions` checks what `apply_action` will do, which relies on it (`unreachable!` at `resolve.rs:74,127`). ADR 0002 chose that split, so I didn't report it. The doc doesn't say whether an ADR outranks the rule.
7. Coordinator. "It knows the order and nothing about any format." The drain loop (`resolve.rs:225-243`) also knows the unit's format, and whether a checkpoint counts as order is unclear. I couldn't tell if F4's home is allowed.
8. Decision. "Before drawing a boundary, list the decisions the code makes." Stack order, the stamp and death timing changed in the same PRs. I split them along ADR lines; the doc gives no test for where one decision ends.
9. Changed before. "A decision is likely to change when it has changed before". In a course nearly everything changed while being built. I counted only reversals and supersessions.
10. Rust tools. "A newtype for a value with a meaning, such as an ID or an amount". Read as a rule, every `u8` amount needs a newtype, against "Leave stable, easy decisions where they are." I used one only where knowledge crosses a boundary informally (F2). The section should say it is a menu for rule 1.
