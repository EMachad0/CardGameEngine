# Audit 2: rules a pass restores, and who may run a command, in `game/`

At `cf3af1e`. `Game`'s fields are private to `game/mod.rs`, so every child of `game` (eight files)
can write them. No `pub(crate)` command below has a caller outside `game`.

## Findings

### 1. When the state check runs (ADR 0012, 0015)

1. The learner's "if the pass runs, a resolving effect has just finished" is too narrow. On the
   drain's first turn, `resolve.rs:231` runs the pass after an action with no resolving effect
   (`EndTurn`, `Draft`, `Choose`, `Cancel`, `Play` before the pop), and
   `fatigue_at_turn_start_can_end_the_game` (`tests/spec/turns.rs:77`) needs it. The rule both ways:
   "if the pass runs, no resolving effect is part-way" (A: calling `check_state`) and "if a resolving
   effect or an action's own steps end, the pass runs before the next pending effect is taken"
   (A: taking it, see 3).
2. Keeper: `drain_effect_queue` (`resolve.rs:225`) with `continue_resolving_effect` (`:245`).
3. `resolve.rs:265`, then `:231` with nothing changed between, so a finished resolving effect gets
   two passes. Both inside the keeper. `check_state` is `pub(crate)` (`state_check.rs:10`).
4. Nothing. Tests catch a missing pass (`turns.rs:77`, `triggers.rs:236`) and an extra pass that
   changes deaths mid-card (`resolution.rs:62`). A duplicate passes them: the pass is idempotent.
5. Nesting, the learner's tree. `loop` is a keyword and the glossary has no term, so a placeholder:
   `game/resolution_loop/{mod.rs, drain.rs, state_check.rs}`, `check_state` `pub(super)`, helpers
   private. One rightful caller, no reason to live elsewhere. Merge the two calls into one site;
   inside the keeper that is informal knowledge with high locality. Nesting doesn't restrict the
   writes the pass makes (see 7).
6. High on facts, medium on tool (gap B).

### 2. Who starts the drain (ADR 0015, "one loop resolves everything")

1. "If the drain runs, `apply` started it after one action, and no drain is running." A: calling
   `drain_effect_queue`; a call from the interpreter would nest a loop.
2. Keeper: `apply` (`resolve.rs:40`). 3. One call, `resolve.rs:52`.
4. Visibility: private to `game::resolve`. `apply_effect` (`:140`) shares the file and could call it.
5. The learner's tree widens this: with `apply` left in `resolve.rs`, the drain's entry becomes
   `pub(in crate::game)`, open to every child. Instead move `apply` (legality check, `apply_action`,
   drain) into `resolution_loop/drain.rs` and keep the drain private. A token doesn't fit: the
   rightful caller is outside the loop (gap C).
6. Medium.

### 3. Taking a pending effect, and the resolving effect's cursor

1. "If a pending effect becomes resolving, or the resolving effect advances or clears, the loop did
   it." A: `effect_stack.pop`, writes to `resolving_effect`, `ResolvingEffect::advance`. Pushes are B
   and stay open (`zone_move.rs:67`, `:100`, `triggering.rs:39`); their order is finding 5.
2. Keeper: the drain. 3. Pop `resolve.rs:233`, set `:234`, `:264`, advance `:255`, all in the keeper.
4. Nothing: both fields open to every child; `ResolvingEffect`'s fields `pub`, `advance` `pub(crate)`
   (`effect.rs:82-106`).
5. No listed tool restricts a field write (gap A). The data can share an owner, which the doc
   prefers: a type with private fields in a leaf under `resolution_loop/`, `push` at
   `pub(in crate::game)`, `start_next`, `step`, `finish` at `pub(super)`. Changes finding 21's fix:
   the cursor goes private to that owner.
6. Medium.

### 4. `apply_effect` runs only as a step of the resolving effect

1. "If an effect is applied, it is the resolving effect's current step" (ADR 0015: pauses only
   between steps). A: calling `apply_effect`.
2. Keeper: `continue_resolving_effect`. 3. One call, `resolve.rs:258`. 4. Nothing.
5. Token. The interpreter changes per verb, the loop per resolution policy, so nesting one under the
   other makes the subtree serve two decisions. `apply_effect` takes a `Step` with a private field,
   built only by the owner in 3. Roadmap 06b's frames for `If` and `Repeat` make producing the step
   the owner's job. Extends finding 21.
6. Medium.

### 5. A pending pick exists only while a resolving effect waits on it

1. "If a player has a `PendingPick`, an unfinished resolving effect waits on it." A: `reveal`.
2. Keeper: none in `game`; the binder validator keeps half (`validity.rs:138`).
3. Write `zone_move.rs:173`; `reveal` called only at `resolve.rs:163`. 4. Nothing in `game`.
5. The `Step` token of 4 passed to `reveal`, or `reveal` private to an interpreter module.
6. Low.

### 6. Only the state check kills (ADR 0012, 0014)

1. "If an object dies, the pass removed it with its pass's other deaths, death triggers pushed before
   any `Died`." A: calling `kill` (`zone_move.rs:96`, `pub(crate)`).
2. Keeper: `remove_dead` (`state_check.rs:30`). 3. One call, `state_check.rs:49`.
4. Nothing. `medic_cannot_save_a_minion_dying_in_the_same_pass` and
   `a_mourner_dying_in_the_same_pass_draws_nothing` catch a changed order, not an extra caller.
5. Nesting: `kill` private in the state check leaf; it is the pass's procedure. `destroy` stays with
   the location moves.
6. Medium-high.

### 7. Outcome and elimination are written only by the pass

1. "If the outcome is set or a player stops playing, the pass decided it." A: writing `Game.outcome`,
   `Player.playing`.
2. Keeper: `check_state`. 3. `state_check.rs:80`, `:53` only. 4. Nothing: open to every child.
5. No listed tool fits (gap A). Hero health can't share their owner, so a keeper stays, but the two
   fields can move into a type with private fields in the state check leaf, read by question.
6. Medium.

### 8. `apply_action` and `pick` run only after the legality check (ADR 0002)

1. "If an action is carried out, `legal_actions` listed it, and the drain follows." A: calling
   `apply_action`, and `pick`, whose `unreachable!` (`zone_move.rs:195`) relies on it.
2. Keeper: `apply`. 3. `resolve.rs:51` (`pub(super)`, open to every child); `pick` `resolve.rs:105`.
4. Nothing. 5. `apply_action` private today: its keeper shares the file. `pick` private beside the
   action handling once that is a module (gap E). 6. High for `apply_action`, medium for `pick`.

### 9. Setup, turns, and commands with no caller rule

`setup` runs `draw` and `start_turn` (`mod.rs:108`, `:111`) with no drain or pass; harmless until a
start-of-turn trigger, which would wait for the first `apply` (low). `end_turn` then `start_turn`
(`resolve.rs:109-110`) is a call order for one command (General), not a caller rule. No rule names
the callers of `draw`, `summon`, `zone_move`, `add_to_hand`, `bury`, `destroy`, `entomb`. `damage`'s
rule is on a write: A is `Object.damage +=` (`zone_move.rs:43`, `:271`), a `pub` field. That is
finding 18, whose fix becomes "restrict the field write" (gap A). `enqueue_triggers` (`pub(super)`,
one call, `triggering.rs:23`) can be private. Precedent for the third tool: the `Observer::event`
ban, `#[expect]` at `triggering.rs:16`. High on facts.

### 10. Checkpoints (ADR 0006, "at set moments"), refines finding 20

1. Implied by tests, written nowhere: "if a checkpoint is reported, a step just ended" (an action
   with no card, a card leaving the hand, a minion entering, an effect, a pass that removed
   something, the end), and the converse. A: `Observer::checkpoint`. The observer is outside the
   state, so no owner.
2. Keeper: none. 3. `zone_move.rs:79`, `:82`; `resolve.rs:106`, `:111`, `:118`, `:130`, `:135`,
   `:259`; `state_check.rs:57`, `:84`. "Only where a resolving effect's step ends" fails: only
   `:259` is one.
4. `properties.rs:105` catches a missing final checkpoint; `events.rs:330` and `events.rs:795` catch
   some extra ones. Most tests pin presence only.
5. Clippy ban on `Observer::checkpoint`, `#[expect]` at each site, as with `event`: the sites span
   four concerns and the method is on a trait, so neither tree nor signature carries it.
6. Low.

## Doc gaps

A. "The keeper must be the only code that can make the writes the rule is about," yet "Restricting
callers" covers only commands: "a keeper that must be the only caller of a command picks its tool
by where the command lives." Fields of a type defined in a parent (`Game`) fit no tool (3, 7, 9).

B. "Its module and the keeper's share a parent whose subtree serves only the keeper's decision."
Does a child hiding its own decision (what the pass removes) serve the keeper's (when it runs)? (1)

C. "The command takes a value with a private field, which only the keeper's subtree can build."
This assumes the keeper is the caller. For the drain the rule's module is the callee (2).

D. "The keeper must be the only code that can make the writes the rule is about." For a rule a pass
restores ("no object on the board at 0 health"), the writes are damage, open by design. The doc
doesn't say the restricted A is then the pass's call.

E. Neither section says how often or where inside the keeper A may happen. The duplicate pass (1)
is judged only by the Coupling section's locality test. Nor is a private item in the keeper's own
file (8) among the tools, which start from siblings.

F. "A Clippy ban fits when neither the tree nor a signature should carry the rule." No test for
"should" (10). A ban on an inherent method by private path is unverified (architecture open thread).
