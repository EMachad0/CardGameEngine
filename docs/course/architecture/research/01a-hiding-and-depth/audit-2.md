# Audit 2: the rules crate against the file-structure convention

## Summary

Identities, randomness and event reporting are well hidden. Draft rules, pick answers, order of play and the effect stack each spread over three to six files, mostly informally.
I found 11 violations. Findings 2, 5 and 6 hide latent bugs that no current card reaches.
The doc never defines "module", "decision" or "coordinator", and those definitions decide how many findings count.

## Decisions likely to change

| Decision | Evidence |
|---|---|
| D1. Where card definitions come from, validated before use | TODO at loader.rs:89, "loading cards from disk ... should be shells job"; game.rs:59, "may explode". A file format is outside the crate's control. |
| D2. Where a card definition keeps effect sequences | definition.rs changed in every PR from #3 to #8; #7 added triggers and `GrantTrigger`. |
| D3. How a draft fills: order, counts, distinct targets | ADR 0007, 0009, 0010, superseded twice; `AtMost` and `AtLeast` commented out at choice.rs:18-19. |
| D4. Where a resolving unit keeps answers it gets mid-resolution | ADR 0015; #8 moved the pick's card moves from `pick` into `ResolvePick`. |
| D5. Which objects are characters in play | ADR 0008 (#4) edited binder, lookup, state_check, zone_move and game.rs; the ADR names games that model it otherwise. |
| D6. Order of play | ADR 0012; added in #7. It orders deaths and triggers. |
| D7. Effect stack order | ADR 0013 superseded by 0014 inside #7; ADR 0015 added a rule in #8. |
| D8. Which trigger patterns exist | New in #7; ADR 0011 lists board entry and exit, which have no pattern yet. |
| D9. What history records and what cards ask of it | ADR 0005; #7 moved recording into `emit`. |
| D10. When a checkpoint happens | Call sites changed in #4 to #8; 89b6131 ruled "not each Effect entry", resolve.rs:259 now does each. |
| Not likely: mana cap, seat order, RNG algorithm | One edit each; no ADR or TODO. |

## Findings

I read "module" as one Rust file. See doc feedback 1.

### 1. `Game::setup` builds and validates the card table

- Decision: D1.
- Known at: game.rs:60 `CardDefLoader.load_and_validate().unwrap()`, formal call, informal order (validate, then bind). binder.rs:19 `Binder::new` takes any `Vec<CardDef>`, informal. game.rs:70 `HERO`, formal. definition.rs:38 `PLACEHOLDER` from the loader, formal. definition.rs:15 `DefId(&'static str)`, informal.
- Rule: one place knows, second bullet. The caller must call two functions in order, and nothing proves a `Binder` holds a validated table.
- Home: a card table type in `cards/` whose only constructor validates; `Game::new` takes one. `DefId`'s `&'static str` is the decision a file loader will force open.
- Confidence: high.

### 2. Two walks over a card's effect sequences, both incomplete

- Decision: D2.
- Known at: loader.rs:645-672 and 678-698 walk play effects, triggers and minion auras with `if let`, informal. lookup.rs:119-176 also reads `GrantTrigger` from `def.modifier_effects` and runtime buffs, informal.
- Rule: one place knows; informal across the loader/lookup boundary. The copies disagree: neither validator visits `modifier_effects` or a `GrantTrigger` inside `Effect::Buff`, so a trigger there can name a choice or leave a pick unresolved and still load.
- Home: `CardDef` yields every effect sequence with its context, by exhaustive destructuring. Validators ask it.
- Confidence: high.

### 3. Draft rules spread over five files

- Decision: D3.
- Known at: game.rs:157-198 walks choices, counts, breaks on the first unfilled. condition.rs:107-137 repeats that loop; condition.rs:30 applies `different`. resolve.rs:63-96 converts `chosen` to `ChoiceTarget`s; resolve.rs:127 is `unreachable!` outside a draft. view.rs:165-179. player.rs:22 fixes the layout `Vec<(ObjectId, ChoiceId)>`. The per-choice count is copied at game.rs:164, condition.rs:18, condition.rs:114, resolve.rs:89. Informal, except the state matches.
- Rule: one place knows; don't split by step. `legal_actions` checks what `apply_action` will do, and `apply_action` relies on it via `unreachable!`.
- Home: a `Draft` type with a private chosen list: `next_choice`, `candidates`, `can_finish`, `choose`, `targets`. `legal_actions` asks what to offer, `apply_action` tells it `choose`, `Play` and `view` ask for targets.
- Confidence: high.

### 4. "Choice fulfilled" answers two questions

- Decision: D3, the count bound.
- Known at: condition.rs:35-45 `check_choice_count_fulfilled`, asked as "may the player stop?" (game.rs:190, condition.rs:118) and "may the player add more?" (game.rs:168, condition.rs:23). `AmountBound::upper_bound` (choice.rs:29) has no caller.
- Rule: "Give each question its own home, even when two questions share an answer today." With `AtMost`, zero chosen satisfies the bound, so `Choose` is never offered.
- Home: `AmountBound` or `Draft` answers `is_complete` and `accepts_more` apart.
- Confidence: high.

### 5. Mid-resolution answers live on the object

- Decision: D4.
- Known at: zone_move.rs:202-207, `pick` pushes a `PickTarget` onto the source object. resolve.rs:171-186, `ResolvePick` takes the first with that `pick_id`. object.rs:20-21, `pub` fields. resolve.rs:84-95 and 360-368 for `choice_targets`. Informal. Nothing clears `pick_targets`.
- Rule: don't split by step. Reveal, pick and resolve hand data forward through `Object`. ADR 0015 keeps "its context" with the unit. A source revealing twice under one pick id would move the first pick's cards again; no card does today.
- Home: `ResolvingEffect` holds chosen targets and pick answers. `pick` tells it; `Chosen` and `ResolvePick` ask it.
- Confidence: high for the rule, medium for the bug.

### 6. "Characters in play" has six answers

- Decision: D5.
- Known at: game.rs:216-227 (sorted by order of play); condition.rs:47-67 (heroes first, unsorted); lookup.rs:56-65 (`health`); state_check.rs:9-22 and 29-34; lookup.rs:158 (auras reach minions only); zone_move.rs:107 (a dead hero stays). binder.rs:50-60 build `is_hero` and kin from `matches!`, so a new card kind compiles everywhere. Informal.
- Rule: one place knows; informal across boundaries. A dead hero stays in `board_characters`, so `emit` matches its own death triggers again on `Died`, against ADR 0012. Heroes have no triggers yet.
- Home: one query beside the zones, `characters_in_play()` in order of play, plus `is_in_play(id)`, on an exhaustive `CardDefKind` match. `emit`, scans, `health` and `find_dead` ask it.
- Confidence: high.

### 7. Order of play is stamped at six sites and sorted at three

- Decision: D6.
- Known at: writes at object.rs:57, zone_move.rs:35, 138, 224, 240, 254; reads at game.rs:225, state_check.rs:42, lookup.rs:140; `pub` field at object.rs:23. Informal: no build failure if a write is missed or "lower is older" flips.
- Rule: one place knows. The code and ADR 0012 already disagree: `updated when object move between zones` against "the order objects entered the board".
- Home: `ObjectBag` keeps the stamp private, sets it on board entry only, and sorts ids for readers.
- Confidence: medium; the extra stamps are harmless today.

### 8. Every pusher knows the effect stack's order

- Decision: D7.
- Known at: game.rs:41, a bare `Vec<PendingEffect>`. Pushes at triggering.rs:34, zone_move.rs:63-68 (before `Played`), zone_move.rs:96-103 (before `Died`); pop at resolve.rs:233; within-event order from finding 7's sorts. Informal.
- Rule: one place knows; `play` and `kill` must push before they emit, a required call order.
- Home: an `EffectStack` in effect.rs with a private `Vec` and situation-named commands. `play` and `kill` pass pending effects to one emit command that pushes them first.
- Confidence: medium; spec tests pin the order.

### 9. Trigger matching ends in a `_` arm

- Decision: D8.
- Known at: triggering.rs:60 `(_, _) => false`; lookup.rs:115 `matches!` for death rattles. Informal.
- Rule: exhaustive match, Rust section. A new `TriggerPattern` builds and never fires.
- Home: stays in triggering.rs, matching `pattern` with no `_` arm and testing the event per arm.
- Confidence: high.

### 10. One module writes history, another reads it

- Decision: D9.
- Known at: triggering.rs:64-98 records, ending `_ => None`; history.rs:3-15 `pub` fields; resolve.rs:285-320 queries, copying the `TurnFilter` match and the meaning of `turn`. Informal.
- Rule: don't split by step. Writer and reader share the layout, and the reader sits in the action module. `CharacterEntered` is written and never read.
- Home: history.rs owns `record` and `count`; `Game` asks it.
- Confidence: medium.

### 11. Ten call sites choose checkpoint moments

- Decision: D10.
- Known at: resolve.rs:106, 111, 118, 130, 135, 259; zone_move.rs:76, 80; state_check.rs:51, 78. Informal.
- Rule: one place knows; "Steps still run, inside one coordinator."
- Home: `apply` and the resolution loop decide checkpoints; `play` and `remove_dead` report events only. Spec tests pin `play`'s two checkpoints, so `play` must become loop steps first.
- Confidence: low.

## Already following the rules

- Determinism (ADR 0001): clippy.toml bans `HashMap`, `HashSet` and the clock; `Rng` hides its state.
- Reporting: `emit` records, reports and matches in one command, and clippy bans `Observer::event` elsewhere.
- Identities: `PlayerId::new` and `DefId::new` are `pub(super)`; only `ObjectBag` makes an `ObjectId`.
- Effects (ADR 0004): resolve.rs:147, loader.rs:626 and 700 match `Effect` with no `_` arm.
- Printed stats: binder.rs:34-100 match `CardDefKind` exhaustively.
- Seat and deck order: `TurnOrder` and `Deck` keep fields private.
- Visibility (ADR 0006): view.rs alone decides what a viewer sees.
- Left alone: the mana cap lives only at game.rs:261, and nothing says it moves.

## Doc feedback

1. "Module" is undefined. "Informal knowledge inside one module is cheap ... Across a module boundary, make it formal." Each `game/` file is a module, but all are `impl Game` blocks reading `Game`'s private fields. If privacy draws the boundary, `game` is one 1,500-line module, and findings 3, 8 and 11 become cheap. I used the file, since "whoever changes the decision is already editing that module" fails across eight files.
2. "Decision" has no size. "Merge the decisions that change together." Stack order could be one decision or four, and ADR 0014 changed one of the four. I counted one; the doc gave no test.
3. "Formal" has two definitions. "Formal knowledge is a reference, such as a name" against "the build fails at every place that holds it." Reading `.order` by name is a reference, yet changing its meaning breaks no build. I used the build test.
4. `_` arms in filters. "An exhaustive `match` with no `_` arm, so adding a variant fails the build at every place that has to handle it." lookup.rs:38-41 filters `ModifierEffect` with `_ => None`, and most new variants should pass. I ruled filters don't "have to handle it", a rule the doc doesn't state. Same for `matches!`.
5. Coordinator against call order. "A caller that must call two functions in order still knows the decision" pulls against "Steps still run, inside one coordinator that calls the modules in order." `play` orders five calls: coordinator or leak? I ruled that an order an ADR defines belongs to one coordinator, and an order a callee needs, like validate before bind, gets taken away. The doc draws neither line.
6. The "likely to change" filter barely filters. "A decision is likely to change when it has changed before". Most files here changed in every PR. I leaned on superseded ADRs, TODOs, commented-out variants and commits that moved one rule across files. The doc names "the roadmap" without saying where it lives.
7. Two questions or one asked twice. "Give each question its own home, even when two questions share an answer today." The two character lists in finding 6 differ in order. Is that a copy or a second question? The doc can't tell me.
8. Data passing and step splits share one test. "Passing data between modules is fine when its layout is a decision you expect to stay put" sits beside "A step hands its output forward as data". Telling `PickTarget` (finding 5) from a fine pass needs the likely-to-change call again.
9. ADRs against the doc. ADR 0002 fixes `legal_actions` and `apply` as check and do steps, while the doc says "A step that checks what a later step will do has to know that step's rule." Which wins is unstated. I kept the interface and moved internals.
10. Tests as formalization. "Where neither can, pin it with a test." Spec tests pin finding 8's order. I read a pinning test as cutting the search, not as satisfying "One place knows each decision". The doc doesn't say.
