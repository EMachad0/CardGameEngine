# One place per decision: refactor of the rules crate

## Goal

Apply `docs/conventions/file-structure.md` to `crates/rules`. Each ticket fixes audit findings
whose fix the doc already settles. Every ticket is a refactor with no behavior change, except
where the ticket names one.

The work has two aims, both judged by the learner on the PR: the code improves, and the PR body
shows the doc's rules applied on purpose, not by accident.

## Before any code

Read in full: `AGENTS.md`, `docs/conventions/file-structure.md`, `docs/conventions/testing.md`,
`docs/CONTEXT.md` (the glossary, for names), this spec and your ticket. ADRs in `docs/adr/` are
the decisions behind the rules code; read the ones your ticket cites.

For background, `docs/course/architecture/research/01d-audit/audit-1.md` to `audit-4.md` hold the
audit each ticket draws on, with line numbers from main at audit time. You are not in a course
session: don't read the rest of `docs/course/`, and ignore the `COURSE.md` line in `AGENTS.md`.

## Working AFK

The learner authorized this effort to run without him.

- Don't ask questions. When the ticket leaves a choice open, pick the option the doc supports
  best, and record it under "Choices" in the PR body.
- Commit, push and update your PR without waiting for review. For this effort's tickets, this
  overrides "never commit before review" and "stop and ask" in `AGENTS.md`. His review happens on
  the PR, against the intentions you state there.
- Push only your ticket's branch. `--force-with-lease` is allowed on it, since tickets 01 to 03
  rebuild branches that already exist. Never push to main, never merge, and never close or
  retarget a PR other than your own.
- If the ticket can't be finished without a behavior change or a design call it doesn't settle,
  do the largest part that can be done, and say what's missing under "Left out".

## Branches and order

| Ticket | Branch | Base | PR | Blocked by |
|---|---|---|---|---|
| 01 | `loader-split` | `main` | #10 | |
| 02 | `grammar-walk` | `main`, once 07 is merged | #14 | 01, 06, 07 |
| 03 | `card-def-kinds` | `grammar-walk` | #15 | 02 |
| 04 | `characters-in-play` | `card-def-kinds` | new | 03 |
| 05 | `trigger-matching` | `main` | #23 | |
| 06 | `leaf-definitions` | `main`, once #24 is merged | new | |
| 07 | `one-path` | `main`, once #10 and 06 are merged | new | 01, 06 |

01 and 05 are merged or in review. 06 starts once #24 is merged, and 07 once #10 and 06 are
merged. 02 starts once 07 is merged, so the checks cover its layout and its imports. 03 and 04
each start once the ticket before has pushed and finished its review loop.

Branches 01 to 03 hold earlier drafts, built on layouts that are gone. Rebuild each from its new
base and reuse what fits. The old commits stay reachable from `origin/modifiers-stats`:

- #10: `e608183`
- #14: `faf4990`, `7b0bb2f`, on top of `c9c211d` (a closed split of `game/`)
- #15: `17c73ac`, `11bec33`, `8965cc5`

## Workflow

1. `git fetch origin`, then create your worktree from the base:
   `git worktree add -B <branch> ../main.<branch> origin/<base>`.
2. Read the docs above and your ticket.
3. Before editing, list the decisions the ticket's code makes and where each is known today.
   That list is the start of your "Intentions".
4. Implement in focused commits, with messages in the style of `git log`.
5. Run `just check::all` until it passes.
6. Run the review loop below.
7. Push. Update the PR's base and body, or open a draft PR for a new branch with
   `gh pr create --draft --base <base>`. When the loop is done, run `gh pr ready`.
8. In the same branch, set your ticket's `Status:` to `resolved` and add an `## Answer` with the
   PR link.

## Review loop

After the checks pass, run rounds of review with subagents:

- Run the `code-review` skill. The fixed point is your base branch and the spec is your ticket
  plus this file. Give the Standards reviewer `AGENTS.md`, `docs/conventions/file-structure.md`
  and `docs/conventions/testing.md` as its sources.
- In the same round, spawn a behavior reviewer. Give it the diff and ask whether any change
  alters what the crate does: events and their order, checkpoints, RNG draws and their order,
  the order of candidates for a random pick, panics, validation results, or the public API beyond
  what the ticket allows. It quotes each hunk it flags.
- Fix every finding you agree with, rerun the checks, and run another round. Stop when a round
  brings no finding you'd fix, or after five rounds. In the PR body, list each declined finding
  with your reason, and anything still open after five rounds.

## Constraints for every ticket

- No behavior change unless the ticket names it. The crate tests in `tests/spec/` pass unchanged,
  apart from import paths. If one needs more than that, the refactor changed behavior: undo it,
  don't edit the test.
- Unit tests that move with their code keep their names and bodies, apart from paths.
- Visibility is the narrowest that compiles. Never widen it for a test (`testing.md`).
- Comments follow `AGENTS.md`. Default is none, and no comment names a doc, a ticket, a finding
  or the course.
- Names come from the glossary. A new module name must pass the doc's name test against its
  siblings.
- Definitions live in leaf modules, and a `mod.rs` holds only `mod` declarations. `game/mod.rs`
  is exempt until `game` is split, so add no definitions to it.
- One path to each definition: import from the defining module, spelled from `crate::`; no
  re-exports inside the crate; nothing inside the crate imports through `prelude`. The Rust
  section of `file-structure.md` says why for both rules.
- Don't touch what another ticket owns, and don't fix what you notice outside your ticket. List
  it under "Left out".

## PR body

The learner reads this first and then checks the code against it. Say what you meant to do,
precisely enough to be checked.

```markdown
Ticket: `docs/issues/one-place-per-decision/issues/NN-<slug>.md`

## Intentions

One entry per decision the PR moves or makes formal.

### <the decision, in one line>

- Before: every place that knew it, as `file`: `function`.
- After: the one place that knows it, and what callers ask or command instead, with signatures.
- Formal for: the changes that now fail the build where they must be handled.
- Informal for: the changes that still compile, and why that's acceptable under the doc.
- Rule: the sentence from `file-structure.md` this applies, quoted.

## Choices

Each choice the ticket left open: the options, your pick, the reason.

## Names

Each new module, type or function name, with one real function that belongs under it and one
that doesn't.

## Behavior

Any behavior change, only as the ticket allows, and how you checked that nothing else changed.

## Left out

What you noticed and didn't do, and why.

## Review

Rounds run, findings fixed, findings declined with the reason.
```

## Launch

Each agent starts from main, with this prompt:

> Work ticket `docs/issues/one-place-per-decision/issues/NN-<slug>.md`, following
> `docs/issues/one-place-per-decision/spec.md`. Work fully AFK.
