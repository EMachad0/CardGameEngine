# Rules around the resolution loop hold only by convention

Status: resolved
Category: enhancement
Type: task
Blocked by: 05
Branch: `state-check-and-drain`, new, from `main` after ticket 05 merges. New draft PR with base
`main`.

## Why

ADR 0015 says "One loop resolves everything. It steps the resolving effect until it finishes, then
runs the state check, then takes the next pending effect, and it stops when the game is over, a
question is pending or nothing is left." Several rules hang off that loop, and each holds only
because today's callers keep it.

- **Only `apply` starts the loop.** `drain_effect_queue` is private in `game/resolve.rs`, and
  `apply` is its only caller. That holds while both share a file.
- **The state check runs once at each boundary.** `check_state` is `pub(crate)`, so any code in the
  crate can run it, mid-effect included. When a resolving effect finishes,
  `continue_resolving_effect` clears `resolving_effect` and runs the state check, and then
  `drain_effect_queue` sees no resolving effect and runs it again. No test catches the second run.
- **Only the state check removes the dead.** `kill`, `destroy` and `entomb` in `game/zone_move.rs`
  are `pub(crate)`, and each has one caller: `remove_dead`, `kill`, `destroy`. Ticket 01's
  boundary for zones may let other code make the same removal. `kill` keeps an order rule: it
  reads every dying object's own death triggers before any of them leaves the board (ADR 0012).
- **Only the state check ends the game.** `Game.outcome` is written only by `decide_outcome` and
  `Player.playing` only by `remove_dead`, yet both are open to every child of `game`.
- **Elimination, known twice.** `remove_dead` eliminates a player who is `playing` with hero
  health at or below 0, and `decide_outcome` counts the players still `playing` with hero health
  above 0.
- **Effects run only as a step.** `apply_effect` in `game/resolve.rs` is `pub(crate)`, and its one
  caller is the loop. `reveal` is `pub(crate)`, and its one caller is `apply_effect`.
- `apply_action` is `pub(super)`, and its one caller shares its file.

The doc's rules: the "Owners guard invariants" and "Rules no owner guards" sections, in particular
"Give a rule an owner whenever its data can share one", "For a rule a pass restores, the write to
restrict is running the pass", and "An order rule, X before Y, reads as 'if Y runs, X ran first',
so restrict Y". The Rust section's "Restricting callers".

## Leave alone

- The death pass reports one death at a time. See the spec's known bugs.

## Behavior to watch

- If you drop the second state check, show that it finds nothing to do in every reachable state,
  or keep it and say why.

## Out of scope

- Who reports `Died` and `GameEnded`: ticket 05, already merged. Build on it.

## Done when

- Each rule above has an owner or a keeper, as the doc decides, and the build stops other code
  from making the writes it guards.
- Elimination is decided in one place.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/47
