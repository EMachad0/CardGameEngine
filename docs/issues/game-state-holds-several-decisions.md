# One file holds every decision on `Game`'s data, and its commands are open to all of `game`

Status: needs-triage
Category: enhancement
Blocked by: PR #68 (ticket 13 of the untangling), and architecture session 02i

## Why

As of PR #68:

- **`game/state.rs` holds several decisions.** Its fields are private, so every function that
  reads or writes them lives in its one `impl Game` block, about 470 lines:
  - `Game`'s data and the public questions on it.
  - How a game starts, in `setup`: object IDs heroes first, the shuffles, three cards each, the
    first turn.
  - How a legal action becomes steps, in `apply_action`, including the order rule that a played
    minion's `Played` is matched before it enters the board.
  - The resolution loop of ADR 0015, in `drain_effect_queue` and `continue_resolving_effect`.
  - The order of the state check, in `check_state`: death passes until one finds no dead, then the
    outcome.
  - What each step is lent, in `write`, and that a step's events are reported before its
    checkpoint, in `run` (ADR 0018).

  Dealing four cards, a new pause in the loop and a new public question are unrelated changes,
  and each edits this file.
- **The keepers that main nested now share that file.** Before PR #68, `apply`, the drain, the
  state check and the death pass nested one level per link: `check_state` was
  `pub(in crate::game::resolve)`, `remove_dead` was `pub(super)` inside `resolve/aftermath/`, and
  `kill` was private to `death.rs`. Now `drain_effect_queue`, `check_state` and the death pass
  step are private methods in `state.rs`. That only the drain runs the state check, only the state
  check runs a death pass, and only `apply` starts the drain is convention inside one file.
- **Commands with one caller are open to all of `game`.** `state.rs` lends the owners, so every
  command it runs has to be visible from it:
  - `effect::prepare` and `effect::resolve` in `resolve/effect.rs`, `death::find_deaths` and
    `death::kill` in `resolve/aftermath/death.rs`, and `Standings::decide` in
    `resolve/aftermath/standings.rs` are `pub(in crate::game)`. `decide` was `pub(super)` before
    PR #68. Each has one caller, `Game::write`.
  - `play::prepare`, `play::play` and `turn_change::end_turn` are `pub(super)`, with one caller
    each, `Game::write`.
  - `zone_move::draw` is called from `state.rs`, `turn_change.rs` and `resolve/effect.rs`, and
    `zone_move::summon` from `state.rs` and `resolve/effect.rs`.

  PR #68 argued that no other module holds a game's owners, so no other module can make these
  writes. The doc names nesting as the tool for a command with one rightful caller.
- **`Game::run` names nothing it does.** It makes a step's writes, reports the events the step
  returns, then takes the one checkpoint. A reader can't tell any of that from `run`.

The doc's rules: "A module keeps the knowledge of its decision local, because all of its code
serves that decision. A boundary that holds many decisions that change independently doesn't. Its
code has to be sorted by decision before a change." "The keeper must be the only code that can
make the writes the rule is about." "Enforce the rule through nesting when the command has one
rightful caller and no reason to live anywhere else. [...] When a guarded command keeps a rule of
its own, its module is the parent of the commands it guards, so a chain of keepers nests one level
per link." "Give each item the narrowest visibility that compiles." "A name passes when a reader,
given any function, existing or new, can tell from the name alone whether it belongs."

## Facts to work with

- Privacy flows down the module tree, so whatever a module sees, every module nested under it
  sees too. Lending `&mut` to one of `Game`'s owners takes code that sees `Game`'s fields. So the
  code that lends the owners and the commands nested under it can't be told apart by privacy.
- What ticket 13 made true, and PR #68 states with its trade-offs: `Game`'s fields are open only
  to `state.rs`; each command takes `&mut` only to the owners it writes; no command takes the
  observer or history; `Game` has one inherent `impl` block; and one function, `Game::run`, takes
  every checkpoint.
- PR #68's body records the gap under "Doc gaps" as "Keepers when only one file can run steps".
  PR #51 recorded a neighbour: whether a main type may sit in a subtree of its commands.
- The doc doesn't say which gives way when the module that owns the data also has to run the steps:
  restricting callers by nesting, restricting them by who can obtain the owner, or one decision per
  module. Session 02i settles it, and its ruling goes here before an agent starts.

## Leave alone

- What each step reports, when it is matched and when checkpoints fall (ADR 0018), and the order
  of the steps.
- The known bugs in the card-game course's open threads.

## Done when

- Each module that holds `Game`'s data or runs its steps holds one decision.
- Each keeper of an order between steps is the only code that can run the steps it guards, and
  each command with one rightful caller is reachable only from it. Where the doc's tools can't do
  that, the PR says why.
- What ticket 13 made formal stays formal, or the PR justifies each place it no longer is.
- The function that takes a step, reports it and checkpoints has a name that says what it does.
- The same events come out in the same order, each matching the same triggers, with the
  checkpoints and RNG draws at the same points among them.
- The checks pass, and the PR justifies every behavior change.
