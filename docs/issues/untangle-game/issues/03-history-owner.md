# History has an owner

Status: resolved
Category: enhancement
Type: task
Blocked by: 02
Branch: `history-owner`, new, from `main` after ticket 02 merges. New draft PR with base `main`.

## Why

`History.entries` is `pub` (`history.rs`), and two files know its layout.

- `history_from_event` in `game/triggering.rs` builds the entries and pushes them. `emit` is its
  only caller.
- `history_query` in `game/resolve.rs` counts them, for `EffectAmount::History`.
- `HistoryKind` has `CardPlayed`, `CharacterDied` and `CharacterEntered`, and each holds a full
  `Object` clone. The entry also stores `player_id` and `turn`.

A change to what an entry keeps, or to how a query counts, has to be made in both files.

The doc's rules: "An owner answers questions about its own data." "Data a module takes in or hands
out is part of its interface: its layout, what each field means and the rules for writing it."

## Facts to work with

- `kill` reports a minion's `Died` after it leaves the board (ADR 0012), and `health` returns
  `None` off the board. Anything an entry needs about a dead minion that only exists in play has
  to be read before the removal. A hero never leaves: `kill` skips `destroy` for objects that
  don't leave their zone on death.
- An earlier draft, PR #16 on `origin/history-private`, was closed before the ownership rules were
  in the doc. Reuse only what fits the doc as it stands.

## Out of scope

- Who reports events and in what order: ticket 05.

## Done when

- One module knows an entry's layout, and counting asks it.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/42
