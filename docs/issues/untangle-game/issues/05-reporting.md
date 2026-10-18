# Reporting an event can reach the whole game

Status: resolved
Category: enhancement
Type: task
Blocked by: 04
Branch: `reporting`, new, from `main` after ticket 04 merges. New draft PR with base `main`.

## Why

- **One command, the whole game.** `Game::emit` in `game/triggering.rs` records history, reports
  to the observer and matches triggers, which pushes pending effects. It reads the turn order, the
  objects, the players' zones and the binder, and it writes history and the effect stack. It takes
  `&mut self`, so it can reach every field of `Game`, and its signature doesn't say what it reads.
- **Thirteen calls.** Ten in nine functions of `game/zone_move.rs` (`draw` twice, then `play`,
  `kill`, `reveal`, `pick`, `add_to_hand`, `bury`, `character_entered`, `damage`), plus
  `decide_outcome` in `game/state_check.rs`, and `end_turn` and `start_turn` in `game/mod.rs`.
- **Any caller can report any event.** Three events are the writes of a rule. `Died` is reported
  only by the state check, `Played` only when a card is played, and `GameEnded` only when the state
  check decides the outcome. `emit` takes the public `Event`, so any of its callers can report any
  of the three.
- **Own death triggers, known twice.** `death_rattles` in `game/lookup.rs` filters for
  `Died(Itself)`, which `kill` reads before the dying leave. `fulfill_trigger_subject` in
  `game/triggering.rs` matches `Itself` again when listeners are matched.

The doc's rules: "A decision that combines several owners' data gets its own module with no data,
under their lowest common ancestor." "A rule about one value of a shared command restricts who can
build that value." From the Rust section: "A module with no data holds free functions that take
what they read as parameters." "For a rule about one value of a shared command, put the token
inside the value." "When the command's input is also public output, decouple them."

## Rulings

- Matching stays in the command that reports. A match made later sees a different board: a played
  minion would hear its own play.

## Leave alone

- A dead hero hears its own death a second time. See the spec's known bugs.

## Out of scope

- The layout of history entries: ticket 03. The effect stack's interface: ticket 04.
- Who may run the state check or kill: ticket 06. This ticket restricts who reports `Died` and
  `GameEnded`; ticket 06 decides where the state check lives.

## Done when

- Reporting can't reach data it doesn't read or write.
- Only the code each rule names can report `Died`, `Played` and `GameEnded`, and the build
  enforces it.
- Pending effects come out in the same order as before.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/45
