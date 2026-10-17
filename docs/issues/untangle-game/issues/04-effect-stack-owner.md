# The effect stack and the resolving effect have an owner

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 03
Branch: `effect-stack-owner`, new, from `main` after ticket 03 merges. New draft PR with base
`main`.

## Why

- **Pushes.** `Game.effect_stack` is a bare `Vec<PendingEffect>`, and `PendingEffect`'s fields are
  `pub` (`effect.rs`). Three sites push to it, and each knows that the newest resolves first:
  - `play` in `game/zone_move.rs` pushes the card's own effects before reporting `Played`, so the
    triggers that `Played` matches land on top.
  - `kill` pushes every death rattle of every dying object before any of them leaves or is
    reported.
  - `enqueue_triggers` in `game/triggering.rs` pushes one entry per matching trigger. `emit` calls
    it once per listener.
- **The cursor.** `ResolvingEffect.idx` is `pub`. `continue_resolving_effect` in `game/resolve.rs`
  clones the current effect, advances the cursor, then applies the clone. `drain_effect_queue`
  pops the stack into `resolving_effect`. Both fields sit on `Game`, open to every child of `game`.
- `enqueue_triggers` is `pub(super)` with one caller in its own file.

ADRs 0013, 0014 and 0015 hold the decisions behind the stack, its order and the resolving effect.

The doc's rules: "Data tied by an invariant shares one owner." "Make a decision its own module
when its knowledge sits in more than one place, today or in planned work, and an interface of
questions and commands can replace that knowledge while asking callers to know less than the data
would." "A command whose only rightful caller shares its file is private."

## Rulings

- Trigger matching stays in the command that reports the event. A match made later sees a
  different board: a played minion would hear its own play.

## Facts to work with

- An earlier draft, PR #17 on `origin/effect-stack`, was closed before the ownership and order
  rules were in the doc. Reuse only what fits the doc as it stands.

## Out of scope

- Carrying out an effect (`apply_effect`), the loop in `drain_effect_queue` and
  `continue_resolving_effect`, and when it runs the state check: ticket 06. Change the loop's calls
  to the stack and the cursor, and leave its structure.
- What `emit` records and reports: tickets 03 and 05.

## Done when

- Only the boundary you build can push, pop or move the cursor.
- Every pending effect resolves in the same order as before.
- The checks pass, and the PR justifies every behavior change.
