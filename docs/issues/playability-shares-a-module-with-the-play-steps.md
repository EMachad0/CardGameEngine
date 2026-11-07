# Playability shares a module with the play steps

Status: needs-triage
Category: enhancement

`crates/rules/src/game/play.rs` holds two decisions that change independently:

- When a card is playable, and which targeting it is played from: `WasPlayable::new` and
  `WasPlayable::with_targeting`, which `legal_actions.rs` asks.
- What a play reads before its first write, which steps it takes and what each writes: `prepare`,
  `PlayingStep` and `write`.

The play steps changed three times on PR #89 without touching `WasPlayable`. Playability rules the
domain may add, such as a full board or a card's own play condition, leave the steps alone.
`docs/conventions/file-structure.md`: "A boundary that holds many decisions that change
independently doesn't. Its code has to be sorted by decision before a change."

The mix was already on main before PR #89.

Fix: make `game/play/` a directory with a `mod.rs` that holds only `mod` declarations, one leaf for
playability and one for the play steps. The steps' leaf takes `WasPlayable` apart through a
`pub(super)` accessor of its sibling.
