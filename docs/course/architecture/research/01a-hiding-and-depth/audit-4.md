# Audit 4: the rules crate against the file-structure convention

## Summary

Determinism, event reporting, visibility and effect interpretation each have one home, made formal by a lint or a type.
The decisions most likely to change, such as order of play, what is in play, the draft rule and mana, each live in three to six files, mostly informally.
Much of the spread follows processing steps, such as an affordability check in `legal_actions` and the payment in `resolve.rs`.
Twelve findings, five of high confidence. Nine doc feedback items.

## Decisions likely to change

| Decision | Evidence |
|---|---|
| Order of simultaneous things | ADR 0014 superseded 0013 here and weighs Runeterra's board order. |
| What is in play | Hearthstone locations and Runeterra landmarks are in play but not characters. |
| What each card kind does | Hearthstone adds weapons and hero cards, Magic artifacts and lands. |
| Mana | Runeterra banks spell mana, Hearthstone has overload, Magic has colors. |
| When a draft is complete | `AtMost` and `AtLeast` sit commented out in `choice.rs:18-19`. ADRs 0007, 0009 and 0010 changed the draft twice. |
| What the game waits on | Hearthstone Discover and Choose One, mulligans, Magic's scry. |
| Effect stack direction | ADR 0013 and 0014 alternatives: a queue, a sort by order of play, own death triggers first. |
| Trigger patterns | Hearthstone end of turn and after summon, Magic enters and leaves the battlefield. |
| Where a card holds effects | `ae79835` replaced `on_death` and two siblings with triggers and aura grants. |
| History counters | Hearthstone counts draws, spells and deaths, per game or per turn. |
| Turn structure | Magic phases, Runeterra rounds, Hearthstone end-of-turn triggers. |
| Which side an object is on | Mind Control. ADR 0011 names change of control as an input. |
| What each player may see | ADR 0010 superseded 0009 to make the draft public. |

## Findings

Formal and informal are judged against the change in the table.

### 1. Order of play

- Decision: simultaneous things go oldest first, by a stamp on each object.
- Known at: `cards/object.rs:23` `pub order`, informal, stamped in one module. The same sort sits in `game.rs:225` (trigger scan), `game/lookup.rs:140` (one object's triggers) and `game/state_check.rs:42` (deaths), all informal:
  ```rust
  death_pass.sort_by_key(|o| self.objects.get(*o).order);
  ```
- Rule broken: one place knows each decision. Switching to board order compiles with any of the three left behind.
- Proposed home: `ObjectBag` keeps the stamp private and answers `play_order(id)`. Deaths, the trigger scan and trigger sorting each get a named question built on it.
- Confidence: high.

### 2. What is in play

- Decision: the objects in play are each hero plus each board, and every board object is a minion.
- Known at, all informal, each walking `zones.hero` and `zones.board` itself: `game.rs:216-227`, `game/condition.rs:47-67`, `game/state_check.rs:11-12,29-36`, `game/lookup.rs:56-63` and `game/lookup.rs:158-160` (auras come from every board object and reach only minions).
- Rule broken: one place knows each decision. A location that is not a character means searching five modules.
- Proposed home: `zones.rs` answers `characters_in_play()` and `is_in_play(id)`, asked by the trigger scan, filters, death scan and aura pass.
- Confidence: medium.

### 3. Card kind questions

- Decision: what each card kind does on death, and which kinds a card filter can name.
- Known at: `cards/binder.rs:34-76`, exhaustive matches, formal. `game/zone_move.rs:127` asks `has_deck_presence` to decide whether a destroyed object goes to the graveyard, informal. `game/zone_move.rs:107` keeps a dead hero in place via `is_hero`, informal. `cards/filter.rs:18-22` copies the variants of `CardDefKind`, mapped by `||` in `game/condition.rs:100-105`, informal: a new kind compiles and no filter can name it.
- Rule broken: "Give each question its own function", and a copy across a boundary.
- Proposed home: `Binder` answers `goes_to_graveyard(def)`, `leaves_zone_on_death(def)` and `matches_kind(def, filter)`, each with its own exhaustive match on `CardDefKind`.
- Confidence: high.

### 4. Mana

- Decision: a card is affordable when its cost is at most the player's mana, and playing it subtracts the cost.
- Known at: `game.rs:130` `if mana_cost > mana { continue; }`, informal. `game/resolve.rs:101` `mana -= mana_cost`, informal. `game.rs:261-262` refill over bare `u8` fields.
- Rule broken: "A step that checks what a later step will do has to know that step's rule." Spell mana changes both sites.
- Proposed home: a mana type in `player.rs` with private fields, answering `can_pay(cost)` and taking `pay(cost)` and `refill()`. `legal_actions` asks, the `Play` arm tells.
- Confidence: high.

### 5. Draft completion

- Decision: the next choice is the first unfilled one, and a draft is complete when every count is met exactly.
- Known at: `game.rs:157-198` and `game/condition.rs:107-137` run the same walk over the choices, an informal copy. `game/condition.rs:18-25` counts again. `game/resolve.rs:77-95,125-129` and `game/view.rs:171` read the bare `Vec<(ObjectId, ChoiceId)>` from `game/player.rs:22`, informal. Enabling `AtMost` changes both walks and the build points at neither.
- Rule broken: one place knows each decision.
- Proposed home: a `Draft` type in `choice.rs` with private fields, answering `next_choice`, `is_complete` and `can_finish`, and taking `choose` and `into_targets`. `legal_actions`, `apply` and `view` ask it.
- Confidence: high.

### 6. What the game waits on

- Decision: the states of `PlayerInteractionState` (`game/player.rs:12-24`) and the moves between them.
- Known at: `game.rs:116`, exhaustive, formal. `game/resolve.rs:69-75,125-128` and `game/zone_move.rs:188-195` destructure with `else { unreachable!() }`, relying on `legal_actions`, informal. `game/zone_move.rs:172-176` and `game/resolve.rs:114-117,134,269-276` build or test states, and `game/view.rs:144-153,166-178` uses `_` arms, all informal.
- Rule broken: informal knowledge across a boundary.
- Proposed home: `player.rs` owns the moves as commands (`open_draft`, `choose`, `cancel`, `post_pick`, `take_pick`, `take_draft`) that return the data, so no caller destructures the enum.
- Confidence: medium.

### 7. Effect stack direction

- Decision: the newest pending effect resolves first, and a source's own effects are pushed before its event so reactions resolve first.
- Known at: `game.rs:41`, a bare `Vec`. `game/resolve.rs:233` `self.effect_stack.pop()`, informal. `game/zone_move.rs:63-74` (play effects before `Played`) and `:96-111` (deathrattles before `Died`) rely on that pop, informal.
- Rule broken: informal across a boundary. A queue is a one-line change in `resolve.rs` that silently inverts both push sites.
- Proposed home: an `EffectStack` type in `effect.rs` with a private `Vec`, plus one `triggering.rs` command that emits an event with the source's own effects scheduled behind its reactions, for `play` and `kill`.
- Confidence: medium.

### 8. Trigger pattern matching

- Decision: which event each `TriggerPattern` (`trigger.rs:15-19`) matches.
- Known at: `game/triggering.rs:46-61`, one match over `(pattern, event)` ending in `(_, _) => false`. Informal: a new pattern compiles and never fires.
- Rule broken: informal across a boundary. ADR 0004 wants "one match arm to each interpreter".
- Proposed home: stays in `triggering.rs`, matching `pattern` with no `_` and testing the event inside each arm.
- Confidence: medium.

### 9. Where a card holds effects

- Decision: which fields of `CardDef` carry effect sequences and triggers.
- Known at: `cards/loader.rs:645-672,678-698`, two walkers, each using `if let CardDefKind::Minion` for aura grants, informal. `game/lookup.rs:119-148`, where live triggers also come from `CardDef.modifier_effects`, informal. Neither validator walks `modifier_effects` (`definition.rs:29`) or a `GrantTrigger` inside `Effect::Buff`, so a trigger there skips the pick and choice checks yet fires in play.
- Rule broken: one place knows each decision. The layout decided in `definition.rs` is walked informally in two other modules.
- Proposed home: `definition.rs` answers `triggers()` (own and granted) and `effect_sequences()` by destructuring every field. The validators and `lookup.rs` ask it.
- Confidence: high.

### 10. History

- Decision: which events become entries, and how a query counts them.
- Known at: `history.rs:3-46`, `pub` fields. `game/triggering.rs:64-98` records, informal. `game/resolve.rs:285-320` counts with `matches!` on `HistoryKind`, informal.
- Rule broken: decisions, not steps. Recording and counting share one layout, so a new counter edits three files.
- Proposed home: `history.rs` takes `record(event, turn)` from `emit` and answers `count(query, ...)` for `effect_amount`.
- Confidence: medium.

### 11. Turn transition

- Decision: ending a turn ends the active player's turn, then starts the next.
- Known at: `game/resolve.rs:109-110` calls `end_turn` then `start_turn`, and `game.rs:106` calls `start_turn` after the opening draws. Informal order.
- Rule broken: "A caller that must call two functions in order still knows the decision."
- Proposed home: one `pass_turn` command in `game.rs`, called by the `EndTurn` arm.
- Confidence: low.

### 12. Side of an object

- Decision: an object's side comes from its owner.
- Known at: `game/condition.rs:57,63` take the player whose zone holds it. `game/triggering.rs:110`, `game/resolve.rs:257` and `game/lookup.rs:28,57,89` take `Object.player_id`. Both informal, and Mind Control splits them.
- Rule broken: one place knows each decision.
- Proposed home: one question in `lookup.rs`, `side_of(object_id)`, asked by the filters and the trigger matcher.
- Confidence: low.

## Already following the rules

- Determinism: `rng.rs:7-9` keeps state private, and `clippy.toml:9-15` bans `HashMap`, `HashSet` and the clocks. Formal by lint.
- Event reporting: `clippy.toml:16` bans `Observer::event` outside `emit` (`game/triggering.rs:10-21`), which also records history and matches triggers.
- Visibility: `game/view.rs` alone decides what a viewer sees, and `Event` has no `DefId` field, so an event can't leak a face.
- Effect interpretation: `apply_effect` (`game/resolve.rs:147-222`), `chosen_id` and `validate_pick_order` (`cards/loader.rs:626-643,700-723`) match `Effect` with no `_`.
- Friendly and enemy: `fulfill_side_filter` (`cards/filter.rs:45-48`) is the one definition.
- Seat order and outcome: `TurnOrder` hides its fields (`turn.rs:6-10`), and `decide_outcome` (`game/state_check.rs:58-80`) alone decides how a game ends.

## Doc feedback

1. "Knowledge is formal for a change when that change fails the build at every place that holds it." Marking a place needs one change in mind. `binder.rs:34-48` is formal for a new kind and informal for "heroes gain attack". I marked against the table's change. The doc should say which change to test.
2. "Judge whether a decision is likely to change by reasoning about the domain: what other systems in it already vary, what the roadmap plans". Nearly everything varies across card games, so the test admits every decision and ranks none. "Hard to get right" has no evidence test, and the doc doesn't say where the roadmap lives. I brought in two rules: ADR considered options count as variance, and commented-out variants count as roadmap.
3. "A filter that only cares about some variants can use `_` or `matches!`, and a new variant passes it silently." Whether a match is a filter depends on future variants: `lookup.rs:76-79` cares only about `BuffHealth`, yet a `SetHealth` modifier must change it. A tuple match like `triggering.rs:46-61`, total on one side only, is not covered.
4. "Informal knowledge inside one module is cheap, because whoever changes the decision is already editing that module." The two walkers in `loader.rs` share a module, but the decision they copy is made in `definition.rs`. It reads both as "the decider and every holder share a module" and as "all copies share one". I took the first.
5. "Give each question its own function, even when two share an answer today" pulls against "When a change to one answer must always change the other, they are one question asked twice". The sorts in finding 1 are one question by ADR 0012 and three by ADR 0014's alternatives. No test separates the two readings.
6. "A caller that must call two functions in order still knows the decision" pulls against "When it is the procedure itself, it lives in the coordinator." The doc doesn't say how to recognize the coordinator. `apply_action`, `play` and `kill` all sequence steps, and finding 11 depends on which one is.
7. "An order belongs to whoever decides it." The order in finding 7 is decided by an ADR, not a module, so "whoever" names no home.
8. "A decision is a choice the code makes where another was possible". No rule groups choices into decisions. The ten `obs.checkpoint` calls in three modules are ten local choices or one step policy. I left them out.
9. "Passing data between modules is fine when its layout is a decision you expect to stay put." It covers data handed forward, not state written in place by several modules, such as `Object.damage` (`zone_move.rs:40,270`, `lookup.rs:82`). I treated shared fields as passed data.
