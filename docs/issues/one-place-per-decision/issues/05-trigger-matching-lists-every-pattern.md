# Trigger matching lists every pattern

Status: ready-for-agent
Category: enhancement
Type: task
Branch: `trigger-matching`, new, from `main`. New draft PR with base `main`.

## Why

`Game::fulfill_trigger_pattern` in `game/triggering.rs` matches on the pair `(pattern, event)` and
ends in `(_, _) => false`. A new `TriggerPattern` variant compiles there and never matches
anything. An auditor added an `Entered` pattern in a copy of the crate and confirmed it. The
card-game roadmap plans an enter pattern.

The doc's rule: "An exhaustive `match` with no `_` arm, so adding a variant fails the build at
every place that has to handle it. This is for matches that decide something for every variant. A
filter that only cares about some variants can use `_` or `matches!`."

## What to build

- Match on the pattern alone, with no `_` arm. Inside each arm, test the event. That test is a
  filter over events, so `_`, `let ... else` or `matches!` are fine there.
- Every `(pattern, event)` pair keeps its result.
- Look for other matches over `TriggerPattern` or `TriggerSubject` that decide something for
  every variant and end in `_`, and fix them the same way. `death_rattles` uses `matches!` as a
  filter, and that's fine. List each match you checked in "Intentions".

## Out of scope

The rest of `triggering.rs`, such as `history_from_event`, and the trigger types themselves.
Ticket 03 edits `Trigger` and `enqueue_triggers` in the same file, so keep the diff to the
matching functions.

## Done when

- Adding a `TriggerPattern` or `TriggerSubject` variant fails the build where events are matched
  against it.
- The checks pass, with `tests/spec/` unchanged.
