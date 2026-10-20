# One walk over a card's targets

Status: resolved
Category: enhancement
Type: task
Blocked by: 08
Branch: `targeting-walk`, new, from `main` after ticket 08 merges. New draft PR with base `main`.

## Why

- **Two walks.** Whether targeting can still be finished is computed by walking the card's
  targets. `can_finish_targeting` in `game/condition.rs` walks them depth-first. `legal_actions` in
  `game/mod.rs` walks them too: in its targeting arm it lists candidates for the first unfinished
  target, and for each candidate it copies `targeted`, adds the candidate and asks
  `can_finish_targeting`. Its idle arm asks `can_finish_targeting` with nothing targeted to decide
  whether a card with targets can be played at all. Both walks know how `targeted` stores answers.
- **One function, two questions.** `check_target_count_fulfilled` in `game/condition.rs` evaluates
  a target's count and asks `AmountBound::is_satisfied` in `target.rs`. Its one answer serves as
  both "is this target full", where `scan_for_valid_targets` offers no more candidates, and "may
  the player stop", where `legal_actions` moves on and `can_finish_targeting` treats the target as
  done. With only `Exactly` bounds the two answers coincide. An `AtMost` bound would offer no
  `Target`. `AmountBound::upper_bound` has no callers.

ADR 0002 makes `legal_actions` the one definition of legality, and ADRs 0007, 0009 and 0010 hold
targeting's decisions. They call it a draft.

The doc's rules: "Give each question its own function, even when two share an answer today. The
second can be built on the first. The day their answers differ, each changes alone." "Ideally one
place knows it, and every other place asks it a stable question or gives it a stable command, by
name."

## Leave alone

- Which actions are legal, and the `is_satisfied` answers for `Exactly`. Don't add a bound.

## Behavior to watch

- The order of each list of legal actions.

## Out of scope

- Where targeting is stored: ticket 02 built its owner.

## Done when

- One walk knows how targeting's answers are stored, and `legal_actions` asks it.
- "Is this target full" and "may the player stop" are separate questions, each with its own
  function.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/49
