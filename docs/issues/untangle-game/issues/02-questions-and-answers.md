# A card's questions and the player's answers have an owner

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 01
Branch: `questions-and-answers`, new, from `main` after ticket 01 merges. New draft PR with base
`main`.

## Why

A card asks its player two kinds of question: choices in a draft, before it is played, and a
pending pick while it resolves. The question and its answer are stored in two places, and nothing
ties them.

- **The question.** `Player.interaction_state` is a `PlayerInteractionState` (`game/player.rs`):
  `Idle`, `PendingPick { source, pick_id, options }` or `Draft { of_object_id, chosen }`. It is
  written in `apply_action` (Play, Draft, Choose, Cancel) in `game/resolve.rs`, and in `reveal`
  and `pick` in `game/zone_move.rs`. It is read in `legal_actions` and `revealed` in
  `game/mod.rs`, `is_pending_interaction` and `apply_action` in `game/resolve.rs`, and two
  builders in `game/view.rs`.
- **The answer.** `Object.choice_targets` and `Object.pick_targets` are `pub` fields. Playing a
  card from a draft clears `choice_targets` and rebuilds it from `chosen`, in `apply_action`.
  `resolve_character_selector` reads it for `Chosen`, with `expect("Choice not fulfilled")`. `pick`
  pushes onto `pick_targets`, and the `ResolvePick` arm of `apply_effect` finds the entry by
  `pick_id`.
- **Revealed cards in no zone.** `reveal` takes cards off a deck into the pending pick's `options`.
  After `pick` they sit only in the source's `pick_targets`, until `ResolvePick` moves them. In
  between they are in no zone. This ticket owns them. Ticket 01's boundary for zones is on main to
  build on.
- **Who may act.** ADR 0015 says "Who may act is derived from the pending question and never
  stored". `legal_actions` derives it in its `Idle` arm from the active player and
  `is_pending_interaction`, which asks every player's `interaction_state`.
- **Wildcard arms.** Three matches over `PlayerInteractionState` end in `_`: `revealed`
  (`_ => &[]`), and `revealed_view` and `draft_view` in `game/view.rs`. A new kind of question
  passes all three silently.

The doc's rules: "Data tied by an invariant shares one owner." "An owner answers questions about
its own data." From the Rust section: "When several filters together decide what each variant
means, give that decision one exhaustive match."

## Leave alone

- `pick_targets` is never cleared, and two picks with one `PickId` on one card resolve from the
  first answer. See the spec's known bugs.
- The pending pick sits on the player whose deck was revealed, so that player picks. Who should
  pick is a rules question.
- The `let ... else { unreachable!() }` in `apply_action` and `pick` rely on `legal_actions`. See
  the spec's "Left for later sessions".

## Out of scope

- Walking a card's choices to list candidates, and `AmountBound::is_satisfied`: ticket 09.

## Done when

- Only the boundary you build can write a pending question, a stored answer or where revealed
  cards sit.
- No match over the kinds of question ends in `_` where it decides something for every kind.
- The checks pass, and the PR justifies every behavior change.
