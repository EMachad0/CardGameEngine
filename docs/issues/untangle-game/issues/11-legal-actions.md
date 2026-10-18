# `apply` checks legality, then assumes it

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 10
Branch: `legal-actions`, new, from `main` after ticket 10 merges. New draft PR with base `main`.

## Why

`Game::apply` asks `legal_actions(player_id).contains(&action)` and gets a yes or no. It then
hands the same `Action` to `apply_action`, whose module docstring says "nothing here re-checks
legality". The code that runs the action relies on the check through `expect`, `unreachable!` and
a write with no guard. After ticket 02, in `game/resolve.rs`, `game/player_input.rs` and
`game/zone_move.rs`:

- Play after targeting. `apply_action` asks whether targeting is open, then `commit_targeting`
  takes it with `let ... else { unreachable!() }`. It writes the answers for the targeting's
  `object_id` and plays the action's `object_id`, relying on the two being one card.
- Play. `mana_cost(object_id).expect(..)` relies on the card being in its owner's hand, and
  `mana -= mana_cost` relies on the player being able to pay. `play` then removes the card from
  the hand with `expect("legal actions guarantee playable cards are in hand")`.
- Target. `PlayerInput::target`'s `let Some(Awaiting::Targets(..)) = .. else { unreachable!() }`
  relies on targeting being open.
- Pick. `pick` takes the pending pick with `let ... else { unreachable!() }`, and finds the picked
  object among its options with `expect("a legal pick should name one of the options")`.

Tickets 01, 02 and 07 move this code, so names and files may differ when you start. The reliance
is the problem. ADR 0002 makes `legal_actions` the one definition of legality.

The doc's rules: "A check validates when it returns nothing, or hands back the type it was given.
What it learned is lost, and every later step that relies on it holds "the check passed" as
informal knowledge." "Keep each fact true wherever its type can go." "Don't hand a parsed type to
code that can keep it while the data changes, or use it on other data. Take that code's input
plain, and parse it where it arrives."

## Leave alone

- Which actions are legal, and the order of each list `legal_actions` returns.
- `apply` takes a plain `Action` from outside the crate, and an `Err` leaves the game unchanged.

## Done when

- No step after `apply`'s check relies on it through `expect`, `unreachable!` or a write with no
  guard.
- Legality still has one definition.
- The checks pass, and the PR justifies every behavior change.
