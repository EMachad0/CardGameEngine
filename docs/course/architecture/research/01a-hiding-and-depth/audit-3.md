# Audit 3: the rules crate against the file-structure convention

## Summary

Legality, event reporting, determinism, visibility and effect interpretation each have one home.
Ten decisions are spread across modules with informal knowledge. The costliest are what counts as
a character in play, mana, the draft's choice rules and the order pending effects resolve in. Most
fixes are a type or module that owns the decision and answers named questions. The doc left ten
calls open, listed at the end.

## Decisions likely to change

| Decision | Evidence |
|---|---|
| What counts as a character in play | Hearthstone locations, MTG planeswalkers, LoR landmarks. Making heroes objects (commit 5273105) touched 15 files in `src/`. |
| Mana: affordability, payment, refill | Hearthstone overload and The Coin, LoR spell mana, MTG colored mana. |
| Draft choice rules: order, count, completion | `AtMost` and `AtLeast` commented out (`choice.rs:18-19`). ADRs 0007 and 0009 superseded. MTG's "up to two targets". |
| The order pending effects resolve in | ADR 0013 superseded by 0014, which lists how Hearthstone, MTG and LoR differ. |
| Order of play | ADR 0012 defines it. ADR 0014: LoR orders by board position. |
| Which card tables are valid | `loader.rs:89`: loading moves to the shell, so the table becomes a file outside the crate's control. |
| Where a card definition holds effect sequences | ADR 0011 grants triggers through auras. Hearthstone enchantments grant deathrattles. ADR 0004 plans `Custom`. |
| What history records and counts | Hearthstone counts spells cast this game, cards played this turn, minions that died. |
| Who an object belongs to | ADR 0011 names a change of control. Hearthstone's Mind Control. |
| When observers get a checkpoint | Tests pin each step. Commit 89b6131 reworked report order. |
| Fatigue | Hearthstone's grows each draw. MTG and LoR end the game instead. |

## Findings

### 1. What counts as a character in play

**Decision:** characters are the hero in each hero zone plus the minions on each board.

**Known at:**
- `game.rs:216-227` `board_characters`, the set `emit` matches against. Informal.
- `condition.rs:47-67` `scan_characters_with_filter`, a second copy in another order. Informal.
- `lookup.rs:56-65` (health), `89-92` (attack), `158` (auras reach minions only). Informal.
- `state_check.rs:11-12, 29-33`. Informal.
- `zone_move.rs:107`: a dead hero stays in its zone. Informal.
- `binder.rs:62-76`, exhaustive matches on `CardDefKind`. Formal.

**Breaks:** one place knows each decision; informal knowledge crosses five modules. A dead hero
stays in `board_characters`, so a hero's own `Died(Itself)` trigger would be pushed by `kill`
(`zone_move.rs:97`) and again by `emit` (`triggering.rs:17-19`). No hero has triggers yet.

**Home:** one question, "characters in play", with "in order of play" built on it and "is this
object in play" beside it. Scans, lookups, the state check and `emit` ask it.

**Confidence:** high.

### 2. Mana

**Known at:**
- `player.rs:29-30`: raw `u8` fields open to every file under `game/`. Formal for a type change,
  informal for a rule change.
- `game.rs:124-131`: affordability, `mana_cost > mana`. Informal.
- `resolve.rs:98-101`: payment, `mana -= mana_cost`. Informal.
- `game.rs:260-262`: ramp, cap of 10, refill. Informal.

**Breaks:** build modules around decisions. `legal_actions` is a step that checks what
`apply_action` will do, so it copies the payment rule. Spell mana or overload changes both, and the
build passes.

**Home:** a mana type with private fields that answers "can pay this cost" and takes "pay" and
"start turn". `legal_actions` asks it, `apply_action` and `start_turn` tell it.

**Confidence:** high.

### 3. Draft choice rules

**Known at:**
- `game.rs:163-188` and `condition.rs:113-134`: the same walk (first unfilled choice in declaration
  order, its candidates, whether the rest can still finish). Informal copy across modules.
- `condition.rs:18-25`: counts chosen targets per choice a third time. Informal.
- `condition.rs:35-45` `check_choice_count_fulfilled`: answers both "is this choice full"
  (`condition.rs:23`) and "can the card be played" (`game.rs:168-194`).
- `resolve.rs:125-128` `unreachable!` and `resolve.rs:366` `expect("Choice not fulfilled")` rely on
  the walk. Informal.
- `resolve.rs:86-94`, `view.rs:171`: the `Vec<(ObjectId, ChoiceId)>` layout. Formal.

**Breaks:** one place knows each decision, and its question bullet. With `AtMost`, "full" and
"complete" differ, and one function answers both.

**Home:** a draft type that owns the chosen targets and answers "next legal chooses", "is
complete" and "targets per choice". `legal_actions` asks it, `apply_action` tells it to choose.

**Confidence:** high.

### 4. The order pending effects resolve in

**Known at:**
- `game.rs:41`: a raw `Vec<PendingEffect>`, newest first by convention. Informal.
- `zone_move.rs:63-68`: play effects pushed before `Played`. Informal.
- `zone_move.rs:96-111`: own deathrattles pushed before `Died`. Informal.
- `triggering.rs:17-20, 29-35` with `game.rs:225` and `lookup.rs:140`: matches pushed in order of
  play. Informal.
- `resolve.rs:233`: pop. Informal.

**Breaks:** one place knows each decision. ADR 0014's order emerges from push calls in three
modules, and "an order belongs to whoever decides it".

**Home:** an effect stack type in `effect.rs` with private storage and commands named by cause:
push play effects, push own deaths, push one event's matches as a batch. It decides the order, and
the loop asks it for the next.

**Confidence:** high.

### 5. Order of play

**Known at:**
- `object.rs:22-23, 75-79`: a crate-visible `usize` stamp and a generic `update_order`.
- `zone_move.rs:35, 138, 224, 240, 254`: five writers, each must remember to renew it. Informal.
- `game.rs:225`, `state_check.rs:42`, `lookup.rs:140`: three readers sort by the raw field.
  Informal.

**Breaks:** informal knowledge across modules, and one place knows each decision. ADR 0012 sets the
stamp at board entry; the code also renews it on draw, bury, hand and graveyard. No one place says
which is meant.

**Home:** `object.rs` keeps the stamp private, takes "entered the board" as a command, and answers
"sort in order of play".

**Confidence:** medium.

### 6. Which card tables are valid

**Known at:**
- `loader.rs:565-723`: the rules.
- `game.rs:59-61`: validate, `unwrap`, then `Binder::new`. The caller holds the order. Informal.
- `binder.rs:19`: accepts any `Vec<CardDef>`.
- `condition.rs:42` `unwrap`, `resolve.rs:366` `expect`, `resolve.rs:179` silent skip, `binder.rs:31`
  `expect`: each assumes a valid table. Informal.

**Breaks:** "When it is a precondition of the callee, such as validating input before storing it,
take it away from callers with one command." Also "A caller that must call two functions in order
still knows the decision".

**Home:** the binder's only constructor validates, or takes a type only the loader can build.

**Confidence:** high.

### 7. Where a card definition holds effect sequences

**Known at:**
- `loader.rs:614-624` (play effects), `645-672` and `678-698` (play effects, triggers, aura
  `GrantTrigger`). Informal, one module.
- `lookup.rs:119-156`: turns every `GrantTrigger` modifier into a live trigger, including those in
  `modifier_effects` and those an `Effect::Buff` adds. Informal, another module.

**Breaks:** one place knows each decision. The two sides already disagree: a `GrantTrigger` in
`modifier_effects` or inside a `Buff` runs at `lookup.rs:130-137`, but no validator walks it.

**Home:** `definition.rs` answers "every effect sequence this card can run, and whether a draft
precedes it", descending into granted triggers. The validators ask it.

**Confidence:** high.

### 8. What history records and counts

**Known at:**
- `history.rs:44-46`: `pub entries`. Formal.
- `triggering.rs:64-98`: which events become entries. Informal.
- `resolve.rs:285-320`: the counts, in the module its docstring calls "the mutation half of
  `Game::apply`", asked by `lookup.rs:42` and `condition.rs:43`. Informal.

**Breaks:** build modules around decisions. A step module holds a decision two others ask, and the
entries' meaning is split between writer and reader.

**Home:** `history.rs` owns entries privately, takes "record this event" from `emit`, and answers
"count this query" for `effect_amount`.

**Confidence:** medium.

### 9. Who an object belongs to

**Known at:**
- `object.rs:16` `player_id`, and membership in a player's `Zones` (`zones.rs:107-114`): two
  representations of one fact.
- Read from the field: `lookup.rs:28, 57, 60, 89, 165`, `condition.rs:28`, `triggering.rs:18, 68,
  110`, `resolve.rs:257`, `zone_move.rs:109`. Informal for a change that splits owner from side.
- Read from the zone holding it: `condition.rs:57, 63`. Informal.

**Breaks:** one place knows each decision. A stolen minion would count as friendly to its new holder
in `scan_characters_with_filter` and to its old one in `fulfill_trigger_subject`.

**Home:** an `owner` question and a separate "whose side is it on" question built on it, per the
question bullet. `player_id` becomes private.

**Confidence:** medium.

### 10. When observers get a checkpoint

**Known at:** `resolve.rs:106, 111, 118, 130, 135, 259`, `zone_move.rs:76, 80`,
`state_check.rs:51, 78`. All informal.

**Breaks:** "When it is the procedure itself, it lives in the coordinator." The schedule is split
between the resolution loop and step functions.

**Home:** the loop in `resolve.rs` checkpoints after each step it runs, and `play`'s two halves and
the state-check pass become steps it runs.

**Confidence:** low.

## Already following the rules

- Legality: `apply` accepts exactly what `legal_actions` lists (`resolve.rs:45`). One command checks
  and acts.
- Reporting: `emit` (`triggering.rs:10-21`) records, reports and matches in one command.
  `clippy.toml:16` bans `Observer::event` elsewhere: formal by lint.
- Determinism: `Rng`'s state is private (`rng.rs:7-9`). `HashMap`, `HashSet` and the clock are
  banned (`clippy.toml:9-15`).
- Effect interpretation: `resolve.rs:147-222`, `loader.rs:626-643` and `loader.rs:700-723` match
  `Effect` with no `_` arm, so a new effect fails the build at each interpreter.
- Visibility: `view.rs:114-154` decides every face. Events carry ids only (`event.rs:2-3`).
- Identity: `ObjectId` can only be built in `object.rs:33-38`, and `PlayerId::new` is crate-only
  (`ids.rs:8`).
- Fatigue lives in `draw` alone (`zone_move.rs:36-47`). Its copy of damage marking
  (`zone_move.rs:39-40`, `269-270`) stays inside one module, which the doc calls cheap.

## Doc feedback

1. Filters that decide together. "A filter that only cares about some variants can use `_` or
   `matches!`." `lookup.rs:38-41, 76-79, 102-105, 130-137` each filter `ModifierEffect` with
   `_ => None`; together they decide which stat every variant changes, yet no single match does.
   `triggering.rs:60` `(_, _) => false` filters events but decides for every `TriggerPattern`
   whether it can fire. I could not tell whether either breaks the rule.
2. Formal for which change. "Knowledge is formal for a change when that change fails the build at
   every place that holds it." The doc doesn't say which change to test. `Object.player_id` is
   formal for a rename and informal for splitting owner from side. I tested against the change the
   domain evidence predicts, a rule the doc doesn't state.
3. What a place is. "A change to a decision costs one edit for each place that knows it" against
   "Informal knowledge inside one module is cheap." Five `update_order` calls in `zone_move.rs` are
   five edits in one module. I counted them only because readers sat elsewhere (finding 5).
4. Evidence that doesn't discriminate. "what other systems in it already vary". Almost every card
   game rule varies across Hearthstone, MTG and LoR, so this test marks nearly everything likely.
   With no roadmap I had no tie-breaker.
5. Evidence the doc doesn't list. I used superseded ADRs and git churn as signs a decision moves.
   The doc's list leaves out the decision's own history.
6. Whose order. "An order belongs to whoever decides it." The effect stack's order is a domain rule
   (ADR 0014) produced by push calls in step functions. It is neither "a precondition of the
   callee" nor plainly "the procedure itself". Giving it to a stack type was my call.
7. Unclear phrase. "Keep the steps where the interface needs them." ADR 0002 makes `legal_actions`
   and `apply` two steps the interface needs. I couldn't tell whether the phrase protects that split
   or only its entry points. The next clause, move shared rules into one module, decided findings 2
   and 3.
8. Data checked at load. Card behavior is data (ADR 0004), soon read from a file. A table that
   breaks a `loader.rs` rule passes the build and fails at startup. The formal and informal
   definitions have no slot for that.
9. One function, another question's name. "Give each question its own function".
   `has_deck_presence` (`binder.rs:70`) is asked at `zone_move.rs:127` to mean "goes to the
   graveyard when destroyed". The doc covers two questions sharing a function, not this.
10. Private fields and child modules. "A private field or constructor, so only the owning module
    can create the value". Rust lets child modules read a parent's private fields, so this lever
    can't separate `game.rs` from the files under `game/`. The doc doesn't say to move the data
    into a sibling type first.
