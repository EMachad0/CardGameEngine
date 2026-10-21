# Untangle `Game`: refactor of the rules crate

## Goal

`Game`, in `crates/rules/src/game/`, is one module that holds about fifteen decisions. Every child
file of `game` reads its fields and most of them write them, so no decision is hidden from the
others. Apply `docs/conventions/file-structure.md` to it. Each ticket takes one decision out of
the tangle, behind the boundary the doc calls for. When ticket 10 merges, `game/mod.rs` holds only
`mod` declarations. Tickets 11 and 12 then carry two checks' results forward as types. Ticket 13
gives `Game`'s data the boundary ticket 10 couldn't.

A ticket describes a problem: what is wrong, where the code shows it, and which rule of the doc it
breaks. It doesn't give the fix. The shape is yours, within the doc. Every ticket is a refactor,
and a behavior change needs a justification, as "Behavior changes" below says.

The learner judges the PR on two counts: the code improves, and the PR body shows the doc's rules
applied on purpose. He reads a choice he disagrees with as a gap in the doc, so state every choice
you make.

## Before any code

Read in full: `AGENTS.md`, `docs/conventions/file-structure.md`, `docs/conventions/testing.md`,
`docs/CONTEXT.md` (the glossary, for names), this spec and your ticket. ADRs in `docs/adr/` are
the decisions behind the rules code. Read the ones your ticket cites.

You are not in a course session. Don't read `docs/course/`, and ignore the `COURSE.md` line in
`AGENTS.md`.

## Working AFK

The learner authorized this effort to run without him.

- Don't ask questions. Where the ticket leaves a choice open, pick the option the doc supports
  best, and record it under "Choices" in the PR body.
- Where the doc gives no answer or two answers, pick one and record it under "Doc gaps".
- Commit, push and update your PR without waiting for review. For this effort's tickets, this
  overrides "never commit before review", "plan before implementing" and "stop and ask" in
  `AGENTS.md`. His review happens on the PR, against the intentions you state there.
- Push only your ticket's branch. Never push to main, never merge, and never close or retarget a
  PR other than your own.
- If the ticket can't be finished without a change this spec keeps off limits, do the largest part
  that can be done, and say what's missing under "Left out".

## Order

The tickets run one at a time. Each starts from main once the ticket before it is merged with
`Status: resolved`, so it builds on the merged code and on any doc change the learner made while
reviewing it.

| Ticket | Branch | Blocked by |
|---|---|---|
| 01 | `zones-and-order-of-play` | |
| 02 | `questions-and-answers` | 01 |
| 03 | `history-owner` | 02 |
| 04 | `effect-stack-owner` | 03 |
| 05 | `reporting` | 04 |
| 06 | `state-check-and-drain` | 05 |
| 07 | `mana` | 06 |
| 08 | `stats-and-modifiers` | 07 |
| 09 | `targeting-walk` | 08 |
| 10 | `game-leaf` | 09 |
| 11 | `legal-actions` | 10 |
| 12 | `card-text` | 11 |
| 13 | `game-data` | 12, and the Mourner fix |

The card game course fixes `docs/issues/dying-mourner-hears-older-death.md`. Ticket 13 starts once
that file is gone from main.

## Workflow

1. Create your worktree from a fresh `origin/main` with `just wk::new <branch>`.
2. Read the docs above and your ticket.
3. Before editing, list the decisions your ticket's code makes and every place that knows each one
   today. For each pair of steps you move, reorder or merge, run the doc's swap test and record
   whether the order is free, forced by data, a precondition or an order rule. That list is the
   start of your "Intentions".
4. Implement in focused commits, with messages in the style of `git log`.
5. Run `just check::all` until it passes.
6. Run the review loop below.
7. Push, and open a draft PR with `gh pr create --draft --base main`. When the loop is done, run
   `gh pr ready`.
8. In the same branch, set your ticket's `Status:` to `resolved` and add an `## Answer` with the
   PR link.

## Review loop

Run rounds of review until one brings no finding you'd fix. There is no limit on the number of
rounds, and the PR isn't finished before the first.

- Each round runs the `code-review` skill. The fixed point is `origin/main` and the spec is your
  ticket plus this file. Give the Standards reviewer `AGENTS.md`,
  `docs/conventions/file-structure.md` and `docs/conventions/testing.md` as its sources.
- In the same round, spawn a behavior reviewer. Give it the diff and ask whether any change alters
  what the crate does: events and their order, checkpoints and where they fall among the events,
  RNG draws and their order, the order of trigger listeners and of candidates for a random pick,
  panics, validation results, or the public API. It quotes each hunk it flags.
- Fix every finding you agree with. Undo each behavior change the reviewer flags, or justify it in
  the PR body. Rerun the checks, and run another round. Stop when a round brings no finding
  you'd fix. In the PR body, list each declined finding with your reason.

## Behavior changes

A behavior change is allowed when it is well justified. Make none by accident.

- Behavior is what an observer or a caller can see: events and their order, checkpoints, RNG draws
  and their order, the order of trigger listeners and of candidates for a random pick, panics,
  validation results, and the public API in `crates/rules/src/prelude.rs`.
- A justification says what an observer or caller sees differently, why the doc's rules call for
  the change over keeping today's behavior, and which tests changed. It goes under "Behavior" in
  the PR body.
- When a test in `tests/spec/` needs more than an import path change, the refactor changed
  behavior. Undo it, or justify it and change the test in a commit of its own, following
  `testing.md`.
- Off limits: the rules of the game, meaning what a card does, who acts or picks, which triggers
  fire and the order they resolve in, what dies and when, and stat values. The card game course
  decides those, so leave every known bug below and every rules question a ticket names as it is.

## Constraints for every ticket

- Unit tests that move with their code keep their names and bodies, apart from paths.
- Visibility is the narrowest that compiles. Never widen it for a test (`testing.md`).
- Comments follow `AGENTS.md`. Default is none, and no comment names a doc, a ticket, a finding
  or the course.
- Names come from the glossary, and a new module name must pass the doc's name test against its
  siblings. When a concept has no glossary term, propose a name and flag it under "Names" for the
  learner. Don't edit the glossary.
- Definitions live in leaf modules, and a `mod.rs` holds only `mod` declarations. `game/mod.rs` is
  exempt until ticket 10, so add no definitions to it.
- One path to each definition: import from the defining module, spelled from `crate::`; no
  re-exports inside the crate; nothing inside the crate imports through `prelude`.
- Don't touch what another ticket owns, and don't fix what you notice outside your ticket. List it
  under "Left out".

## Left for later sessions

The course takes these on later. Keep each as it is, even where your ticket's code touches it.

- The prelude exports `CardDef`, which nothing outside the crate uses.

## Known bugs to leave alone

Each fix is a rules change, and the card game course makes it test-first. Keep each one exactly as
it behaves today, even when your refactor makes the fix look like one line.

- A dead hero stays among the trigger listeners and hears its own death a second time.
- With three players or more, an eliminated player's hero stays a target. Setup takes two decks,
  so it can't happen yet.
- Fatigue adds damage to the hero by hand, outside `Game::damage`, so damage triggers don't see it.
- `Object.pick_targets` is never cleared, and the card validator accepts two picks with one
  `PickId` on one card.
- Setup runs no drain, so triggers matched during setup wait for the first `apply`.

## PR body

The learner reads this first and then checks the code against it. Say what you meant to do,
precisely enough to be checked.

```markdown
Ticket: `docs/issues/untangle-game/issues/NN-<slug>.md`

## Intentions

One entry per decision the PR moves, hides or makes formal.

### <the decision, in one line>

- Before: every place that knew it, as `file`: `function`.
- After: the one place that knows it, and what callers ask or command instead, with signatures.
- Boundary: the owner, keeper or module with no data that holds it, and what makes the boundary
  enforced (privacy, nesting, a token or a lint).
- Formal for: the changes that now fail the build where they must be handled.
- Informal for: the changes that still compile, and why that's acceptable under the doc.
- Rule: the sentence from `file-structure.md` this applies, quoted.

## Orders

Each order between steps you moved or kept, its kind under the swap test, and who keeps it.

## Choices

Each choice the ticket left open: the options, your pick, the reason.

## Doc gaps

Each place the doc gave no answer or two: the question, what you did, and why.

## Names

Each new module, type or function name, with one real function that belongs under it and one
that doesn't. Flag names with no glossary term.

## Behavior

Each behavior change with its justification. Then how you checked that nothing else changed.

## Left out

What you noticed and didn't do, and why.

## Review

Rounds run, findings fixed, findings declined with the reason.
```

## Launch

Each agent starts from main, with this prompt:

> Work ticket `docs/issues/untangle-game/issues/NN-<slug>.md`, following
> `docs/issues/untangle-game/spec.md`. Work fully AFK.
