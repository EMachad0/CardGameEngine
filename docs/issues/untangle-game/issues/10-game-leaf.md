# `game` holds more than how its children combine

Status: resolved
Category: enhancement
Type: task
Blocked by: 09
Branch: `game-leaf`, new, from `main` after ticket 09 merges. New draft PR with base `main`.

## Why

`game/mod.rs` defines `Game` and carries `#![expect(clippy::definition_in_module_root)]`, with the
reason "the child modules read Game's private fields, which a leaf would hide from them".
Tickets 01 to 09 move most decisions out. What is left is still a method on `Game`, open to every
child of `game`:

- Starting and ending a turn: `end_turn` reports `TurnEnded`, then advances the turn order.
  `start_turn` reports `TurnStarted`, refills mana, then draws.
- Setup: `setup` builds the objects and players, shuffles the decks, draws, and starts the first
  turn.
- Random picks and selectors: `select_random` reads the rng, and `resolve_player_selector` and
  `resolve_character_selector` in `game/resolve.rs` build the candidate lists. The rng is also read
  by the shuffle in `setup`.
- Filters: the character and card filters in `game/condition.rs`, used by targets, selectors and
  trigger matching. The side filter is already a free function in `cards/filter.rs`.
- Characters in play: `characters_in_play` in `game/lookup.rs`, and the listener list that
  reporting asks for.
- The view, in `game/view.rs`, and the public accessors: `hand`, `deck`, `player_board`, `mana`,
  `revealed`, `outcome`, `players`, `def_id`, `hero_id`, `hero_health`, `health`, `attack`,
  `mana_cost`.
- Visibility: no `pub(crate)` function in `game/` has a caller outside `game/`.

Some of this may have moved in tickets 01 to 09. Start from what is on main.

The doc's rules: "A child module counts as one item in its parent, since its decision is hidden
from the parent too. Judge a parent by its own code, which should hold one decision, often how its
children combine." "Steps still run, inside code that calls the modules in order. That code knows
the order and nothing about any format. If an order between its steps is an order rule, it is the
rule's keeper. If every order it holds is free or forced by data, it is a coordinator and holds no
rules." From the Rust section: "A directory's own code, such as its main type or how its children
combine, goes in a leaf named for that decision." "Keep a type's inherent `impl` blocks in the
module that defines it, so its whole interface sits in one file. Code elsewhere that builds on the
type takes it as a parameter."

PR #51 predates that last rule. Rework it on its branch, rebased on main, instead of opening a new
PR.

## Behavior to watch

- `Game`'s public name and its public methods' signatures.
- Random draws and their order. A seeded game replays only while they stay the same.

## Done when

- `game/mod.rs` holds only `mod` declarations, and the `expect` is gone.
- Every decision left in `game` has a home the doc supports, and the code that runs steps knows
  their order and nothing about any format. One exception: `Game`'s commands may stay `impl Game`
  blocks outside its module, since a command that reports or checkpoints can't take less than
  `Game`. Ticket 13 removes them.
- Every item in `game/` has the narrowest visibility that compiles.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/51
