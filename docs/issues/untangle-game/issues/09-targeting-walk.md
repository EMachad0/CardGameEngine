# One walk over a card's choices

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 08
Branch: `draft-walk`, new, from `main` after ticket 08 merges. New draft PR with base `main`.

## Why

- **Two walks.** Whether a draft can still be finished is computed by walking the card's choices.
  `can_finish_choices` in `game/condition.rs` walks them depth-first. `legal_actions` in
  `game/mod.rs` walks them too: in its Draft arm it lists candidates for the first unfinished
  choice, and for each candidate it copies `chosen`, adds the candidate and asks
  `can_finish_choices`. Its Idle arm asks `can_finish_choices` with an empty draft to decide
  whether a card with choices can be played at all. Both walks know how `chosen` stores answers.
- **One function, two questions.** `check_choice_count_fulfilled` in `game/condition.rs` evaluates
  a choice's count and asks `AmountBound::is_satisfied` in `choice.rs`. Its one answer serves as
  both "is this choice full", where `scan_for_valid_choices` offers no more candidates, and "may
  the player stop", where `legal_actions` moves on and `can_finish_choices` treats the choice as
  done. With
  only `Exactly` bounds the two answers coincide. An `AtMost` bound would offer no `Choose`.
  `AmountBound::upper_bound` has no callers.

ADR 0002 makes `legal_actions` the one definition of legality, and ADRs 0007, 0009 and 0010 hold
the draft's decisions.

The doc's rules: "Give each question its own function, even when two share an answer today. The
second can be built on the first. The day their answers differ, each changes alone." "Ideally one
place knows it, and every other place asks it a stable question or gives it a stable command, by
name."

## Leave alone

- Which actions are legal, and the `is_satisfied` answers for `Exactly`. Don't add a bound.

## Behavior to watch

- The order of each list of legal actions.

## Out of scope

- Where the draft is stored: ticket 02 built its owner.

## Done when

- One walk knows how a draft's answers are stored, and `legal_actions` asks it.
- "Is this choice full" and "may the player stop" are separate questions, each with its own
  function.
- The checks pass, and the PR justifies every behavior change.
