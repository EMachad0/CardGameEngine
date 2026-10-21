# Every command holds all of `Game`

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 12, and the fix for `docs/issues/dying-mourner-hears-older-death.md`
Branch: `game-data`, new, from `main` once ticket 12 has merged and that issue's file is gone from
`main`. New draft PR with base `main`.

## Why

As of ticket 10 (PR #51):

- **`Game`'s data is open to every command.** `Game` lives in `game/state.rs` with its fields
  `pub(super)`, so every `impl Game` block under `game/` can read and write every owner. Which
  command writes which owner is convention. PR #51 says so under "Doc gaps" and "Left out". Only
  the read-only card questions got a boundary, `Cards` in `game/cards.rs`.
- **Its commands sit outside its module.** Every command is an `impl Game` block in a file of its
  own, outside `game/state.rs`. Ticket 10 left them there, since a command that reports or
  checkpoints can't take less than `Game`.
- **A player's ID is open too.** `Player.id` in `game/player.rs` is `pub(super)`, and
  `get_player` finds a player at the index of its ID in `players`. Any child of `game` can break
  that invariant by writing the ID or reordering `players`.
- **Reporting runs inside the commands, between their writes.** `Game::report` in
  `game/reporting.rs` matches the event against the card state, then records it in history,
  reports it to the observer and pushes each match onto the effect stack. Eleven commands call it,
  `draw` twice: `play`; `draw`, `add_to_hand`, `bury` and `character_entered` in
  `game/zone_move.rs`; `end_turn` and `start_turn`; `pick`; `reveal` and `damage`; `kill`; and
  `check_state`.
- **So do checkpoints, at ten sites.** `obs.checkpoint(Views::new(self))` lets the observer build
  every player's view, which reads the turn order, the players, standings, zones, objects, the
  binder and player input. There are two calls in `play`, five in `apply_action`, one after each
  effect in `continue_resolving_effect`, one per death pass in `remove_dead` and one in
  `check_state`. No one place decides where they fall.
- **A command holds what the readers need as well as what it does.** A function can pass on only
  what it holds. `end_turn` writes only the turn order before it starts the next turn, yet for its
  whole run it holds the card state, and `&mut` history and effect stack, because it reports. A
  struct of borrows narrowed to `play` would hold every field but the rng. Holding a borrow in
  order to lend it is still holding it. Any command can push onto the effect stack outside
  reporting, though the order of pushes is a rule of the game (ADRs 0014 and 0015), and its
  signature doesn't show whether it does.

The doc's rules: "A module in the general sense is a `mod` whose private items include the data
its decision is stored in." "The keeper must be the only code that can make the writes the rule is
about." "Make a decision its own module when its knowledge sits in more than one place [...]".
"Keep a type's inherent `impl` blocks in the module that defines it, so its whole interface sits
in one file. Code elsewhere that builds on the type takes it as a parameter." "A function that
takes the whole holder instead hides what it reads, can start reading more with no change to its
signature, and keeps its callers from writing any part of the holder while it runs."

## Rulings

- Matching stays with reporting, against the state it sees today. A match made against a later
  state sees a different board: a played minion would hear its own play.
- Three designs are out: runtime borrow checks such as `RefCell`, which turn a build error into a
  panic; a hand-written state machine per command; and a reporter type shaped for each caller, to
  which the command lends its own borrows at every report.

## Leave alone

- The known bugs in the spec. The Mourner fix lands before this ticket starts.

## Out of scope

- Restricting one command of an owner to its rightful caller, such as `Mana::refill` to the turn
  start. Narrowing which owners a command holds doesn't do it, since `&mut Mana` reaches `refill`
  as well as `pay`.

## Done when

- `Game`'s fields are open only to code that must hold all of them, and the build enforces it.
- Which owners each command writes is formal: a command that writes an owner it doesn't write today
  fails the build.
- No command holds data only to hand it to reporting or to a checkpoint.
- `Game`'s inherent `impl` blocks sit only in the module that defines it.
- One place decides where checkpoints fall.
- The same events come out in the same order, each matching the same triggers, with the
  checkpoints and RNG draws at the same points among them.
- This PR finishes the effort. It moves any lasting decision to its home, then deletes
  `docs/issues/untangle-game/`. A decision that clears the bar in `docs/adr/README.md` gets its
  ADR in this PR.
- The checks pass, and the PR justifies every behavior change.
