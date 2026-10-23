# Picks as choices: effects as subjects with verb lists, and picks read in place

## Goal

Build ADR 0019 and ADR 0020 in `crates/rules`.

Today each `Effect` variant carries its own selector. A resolution-time pick is two effects joined
by a `PickId`: `Reveal` takes cards off the deck into no zone, and `ResolvePick` moves them later.
`PlayerInput.picks` keeps every answer by source and never clears it, and a load check pairs each
`Reveal` with its `ResolvePick`.

When the effort finishes:

- An effect is a subject (players, characters or cards) with one selector and a list of verbs.
- A target and a pick are one `Choice`, answered at different times.
- A pick reads its options in place, and the resolving effect holds the picking until its verbs
  run.

## Tickets

| # | Ticket | Who | Blocked by |
|---|---|---|---|
| 01 | `issues/01-target-form-and-choice.md` | An agent | |
| 02 | `issues/02-subjects-and-verb-lists.md` | An agent | 01 |
| 03 | `issues/03-card-subject-and-card-targets.md` | The learner, with an agent | 02 |
| 04 | `issues/04-card-picks.md` | The learner, with an agent | 03 |
| 05 | `issues/05-character-picks.md` | The learner, with an agent | 04 |

## Before any code

Read in full: `AGENTS.md`, `docs/conventions/file-structure.md`, `docs/conventions/testing.md`,
`docs/CONTEXT.md`, ADRs 0015, 0017, 0018, 0019 and 0020, this spec and your ticket.

## Roles

- Tickets 01 and 02 are refactors. Every existing test passes with its assertions unchanged, and a
  behavior change needs a justification in the PR.
- In tickets 03 to 05 the learner writes the design-bearing code: types, traits, key function
  signatures and bodies. The agent working with him writes scaffolding and tests against his API
  when he asks, reviews his code against `docs/conventions/file-structure.md`, and refactors on
  request.
- In every ticket, each behavior under "Tests" gets a test before the code that makes it pass. Add
  a test card wherever no existing card reaches a behavior.

## Working AFK, tickets 01 and 02

The learner authorized these two tickets to run without him.

- Work on a new branch named for the ticket, from `main` once the tickets it is blocked by have
  merged, with a draft PR against `main`.
- Don't ask questions. Where the ticket leaves a choice open, pick the option
  `docs/conventions/file-structure.md` supports best, and record it under "Choices" in the PR body.
- Where the doc gives no answer or two answers, pick one and record it under "Doc gaps".
- Commit, push and update your PR without waiting for review. For these two tickets, this
  overrides "never commit before review", "plan before implementing" and "stop and ask" in
  `AGENTS.md`. His review happens on the PR, against the intentions you state there.
- Push only your ticket's branch. Never push to main, never merge, and never close or retarget a
  PR other than your own.
- If the ticket can't be finished without a change this spec keeps out, do the largest part that
  can be done, and say what's missing under "Left out".

Tickets 03 to 05 follow `AGENTS.md` as written.

## Not in this effort

Each waits for a card that needs it:

- A chooser other than the source's owner, and the side filter measured from that chooser.
- An order for the cards in `others`, as in "in any order".
- `others` on `Random`.
- `AtMost` picks, with an action that ends the pick early.
- `sample`, `DeckWindow::Every` and `DeckWindow::Bottom`.
- Discover from a pool of definitions.
- The hand target, and the rule that the card being played is never one of its own options.
- `All` and `Random` on cards, and `Modify` on cards.
