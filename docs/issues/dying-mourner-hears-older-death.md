# A dying Mourner hears an older minion's death in the same state check

Status: ready-for-human
Category: bug

ADR 0012 says the state check removes every dead object together, then emits a `Died` for each in
order of play, so a removed object hears no other death. `Game::kill` in
`crates/rules/src/game/zone_move.rs` breaks this. Its second loop calls `destroy` and then `emit`
one object at a time, and `emit` matches against `board_characters()`. An object later in the pass
is still on the board when `emit` matches an earlier object's `Died`.

## Reproduce

Play a Recruit, then a Mourner, whose trigger is "a friendly minion died: its owner draws 1". Ping
the Mourner, then cast Blast so both die in one pass. The events are `Died` Recruit, `Died`
Mourner, then `Drew` for the Mourner's owner.

Expected: no draw.

## Why the suite passes

`a_mourner_dying_in_the_same_pass_draws_nothing` in `crates/rules/tests/spec/triggers.rs` puts the
Mourner on the board before the Recruit. The Mourner is older, so `kill` removes it first. The test
needs a twin with the Recruit older than the Mourner. That twin should fail today.

## Related, latent

`kill` never removes a dead hero from the board, and `board_characters()` always includes heroes.
On the hero's own `Died`, `emit` would match its `Died(Itself)` triggers a second time, after
`kill` already queued them. No hero has triggers yet.

The fix changes behavior and sits in design-bearing code, so it is built test-first under the
course roles.
