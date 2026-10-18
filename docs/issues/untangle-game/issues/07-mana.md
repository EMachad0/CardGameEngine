# Mana has an owner

Status: resolved
Category: enhancement
Type: task
Blocked by: 06
Branch: `mana`, new, from `main` after ticket 06 merges. New draft PR with base `main`.

## Why

`Player.mana` and `Player.max_mana` are `pub(super)` fields (`game/player.rs`), and three places
in two files hold the rules for them. `Game::mana` and `player_view` in `game/view.rs` read them.

- `legal_actions` in `game/mod.rs` checks whether a card is affordable: `if mana_cost > mana`.
- `apply_action` in `game/resolve.rs` pays for a played card: `mana -= mana_cost`, before `play`.
- `start_turn` in `game/mod.rs` refills: `max_mana = (max_mana + 1).min(10)`, then
  `mana = max_mana`. It runs after `TurnStarted` is reported.

The glossary defines Play as "Paying a card's cost, taking it from the hand and resolving it". The
payment lives in `apply_action` and the rest in `play`.

The doc's rules: "A step that checks what a later step will do has to know that step's rule."
"When an interface needs separate steps, keep them as entry points, and move each rule they share
into one module that both ask." "An owner answers questions about its own data."

## Behavior to watch

- The refill runs after `TurnStarted` is reported, and the payment before `play`.

## Out of scope

- How a card's cost is computed (`mana_cost` and its modifiers): ticket 08.
- The payment relying on the legality check, through `expect` and a subtraction with no guard:
  ticket 11.

## Done when

- One module holds the rules for spending and refilling mana, and the places that check, pay and
  refill ask it or command it.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/44
