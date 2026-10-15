# One question answers which characters are in play

Status: resolved
Category: enhancement
Type: task
Blocked by: 03
Branch: `characters-in-play`, new, from `card-def-kinds` after ticket 03. New draft PR with base
`card-def-kinds`.

## Why

Which objects are characters in play? Today the answer is every player's hero plus every object
on a board, and three places compute it on their own:

- `Game::board_characters` in `game/mod.rs`: heroes, then each board, sorted by order of play. These
  are the listeners `emit` matches triggers against.
- `Game::scan_characters_with_filter` in `game/condition.rs`: heroes in seat order, then each
  board in seat and board order, filtered. These are the targets for `All`, `Random` and draft
  candidates.
- the in-play test in `Game::health`, in `game/lookup.rs`.

A change to the answer means finding every copy by hand. The card-game roadmap plans two such
changes: dead heroes leaving play, and mortally wounded minions.

The doc's rule: "Give each question its own function, even when two share an answer today. The
second can be built on the first."

## What to build

- One function in `game` answers membership. Where it ends up belongs to a later split of
  `game`, so give it no module of its own. Put it in one of `game`'s child files, not in
  `game/mod.rs`.
- `board_characters`, `scan_characters_with_filter` and the in-play test in `health` build on it.
- Keep both orders exactly. A random pick draws from the candidate list by index, so any change
  to the order of targets changes seeded games, even when every test still passes.

## Don't fix

Two latent bugs sit in this code. Both change behavior, and the course fixes them test-first:

- A dead hero stays among the listeners and hears its own death a second time. It's recorded in
  `docs/issues/dying-mourner-hears-older-death.md`.
- With three players or more, an eliminated player's hero stays a target. It can't happen yet,
  because setup takes two decks.

## Out of scope

- The death scan in `game/state_check.rs`. A filed bug lives in the death pass, and its fix comes
  test-first.
- `Game::attack`'s board test, aura sources in `Game::modifiers`, and who owns an object.

## Done when

- One function knows which objects are characters in play, and the three places ask it.
- Every listener list and every candidate list comes out in the same order as before.
- The checks pass, with `tests/spec/` unchanged.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/28
