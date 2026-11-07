---
name: review-loop
description: Review the changes on a branch in parallel fresh-context lanes (behaviour, spec, locality, names and placement, tests, text), verify each finding, let three judges decide which confirmed findings are worth fixing, fix those, commit each round, then push and open a pull request. Use as the last step of every AFK task, when the user asks for a review loop, or when a change needs one.
---

# Review loop

Fresh subagents review the branch in lanes, a fresh verifier checks each finding, three fresh
judges label each confirmed finding, and you fix the ones they pass. Rounds repeat until one leaves
nothing to fix. This skill authorizes the subagents it names.

## When to run

Always run it as the last step of AFK work. It replaces the `/code-review` step that `implement`
names. Otherwise run it when the user asks, or when you judge a change needs it.

## Roles

You, the session that wrote the code, run the loop. You launch the lanes, verifiers and judges,
triage, fix, commit and open the pull request. Lanes, verifiers and judges are fresh subagents and
never see your reasoning. Each runs as one of two agents under `.pi/agents/`:

- `review-lane` reads files and runs read-only commands. Judges run as it.
- `review-probe` writes and runs probe tests. Launch it only with `worktree: true`, so nothing it
  writes reaches the branch.

## Start

1. Check that the branch is not `main`. On `main`, stop and say so.
2. Run the checks and commit any uncommitted work once they pass. That commit ends round 0.
3. The base is `git merge-base origin/main HEAD`.
4. The spec is the ticket you worked from, or the user's request, quoted.

## Lanes

A lane runs when the round's review range touches its trigger. Brief paths are relative to this
skill's directory.

| Lane | Brief | Agent | Trigger |
|---|---|---|---|
| Behaviour | `lanes/behaviour.md` | `review-probe` | Rust code, a `Cargo.toml` or `Cargo.lock` |
| Spec | `lanes/spec.md` | `review-lane` | anything |
| Locality | `lanes/locality.md` | `review-lane` | a file under `crates/*/src/` |
| Names and placement | `lanes/names.md` | `review-lane` | a file under `crates/*/src/` |
| Tests | `lanes/tests.md` | `review-lane` | a test, a test helper or a bench |
| Text | `lanes/text.md` | `review-lane` | a comment, docstring, assertion message, TOML comment or Markdown file |

Every lane also follows `reviewing.md`. When the range changes a `Cargo.toml`, check it yourself
against `docs/conventions/dependencies.md`.

## A round

1. **Range.** Round 1 reviews `<base>..HEAD`. Each later round reviews only the previous round's
   fix commit.
2. **Lanes.** Launch the triggered lanes in parallel in one workflow, with `context: "fresh"`.
   Each task gives:
   - the paths of `reviewing.md` and the lane's brief
   - the review range, and the branch range `<base>..HEAD` as context
   - the spec
   - the ledger of every earlier round
   - for `review-probe`, this checkout's path, so it can check it isn't running there
3. **Verify.** Once every lane has reported, launch one fresh verifier per finding, all in
   parallel. Each follows `verifying.md`. Behaviour findings go to `review-probe`, the rest to
   `review-lane`. Each task gives the finding as its lane reported it, the two ranges, the spec,
   and for `review-probe` this checkout's path.
4. **Judge.** If any finding is confirmed, launch three judges in parallel, with
   `context: "fresh"`. Each follows `judging.md`. Each task gives every confirmed finding of the
   round, as its lane reported it, cut to the places its verifier upheld, in an order of its own.
   It also gives the two ranges and the spec. No judge sees another judge's report.
5. **Triage.** Sort each verified finding:
   - Refuted: drop it, with the verifier's reason.
   - Pre-existing: file it in `docs/issues/`, in a commit of its own, as
     `docs/agents/issue-tracker.md` says. If its fix needs a design call, it waits for the user
     instead.
   - Confirmed, and it contradicts what the user asked for: drop it.
   - Confirmed, and fewer than two judges label it blocker or should: it is a nit. Don't fix it.
   - Confirmed, and the fix needs a design call about an interface, a schema, a stored format, a
     policy, a dependency `docs/conventions/dependencies.md` leaves to the user, or code the
     course's `COURSE.md` roles give the learner. Don't fix it. It waits for the user.
   - Confirmed otherwise: fix it.

   The labels decide. Don't fix a nit, and don't skip a finding the judges pass for a reason this
   list doesn't give.
6. **Fix.** Fix every place on the branch where the finding's flaw appears, not only the cited
   line. A Behaviour fix keeps the verifier's failing test as its regression test.
7. **Commit.** Run the checks. Once they pass, commit the round's fixes, titled
   `Review round <n>: <what the fixes change>`, with the round's ledger as the body. If a fix
   can't get the checks to pass, stash its files with
   `git stash push -m "review round <n>: <finding>" -- <paths>`. The finding then waits for the
   user.

The ledger has one line per finding of the round: its lane, `path:line`, what it found, the three
labels, and the outcome. The outcome is fixed, a nit, filed as `<issue path>`, waiting for the
user, dropped because the user asked otherwise, or refuted with the reason. A last line gives the
three judges' scores.

## Stop

- A round has nothing to fix. The loop is done.
- A round's fixes change only comments, docstrings or Markdown. The loop is done, and nobody
  reviews those fixes.
- A finding fixed in an earlier round comes back, or a fix undoes an earlier one. Stop, and name
  both in the report.
- Round 10 ends. Nobody reviews its fixes, and the report says so.

## Pull request

Push with `git push -u origin HEAD`. If the branch has no pull request, open one with
`gh pr create`. Title it in the style of `git log`, and use the report as its body. If the branch
has one, post the report with `gh pr comment`.

The report, in this order:

1. Waiting for the user: each finding left for a design call, each fix the checks rejected, and
   why the loop stopped early, with the evidence.
2. The rounds run, and why the loop stopped.
3. What each round fixed, one line per commit.
4. The judges' scores, as a table with a row per judge and a column per round. A round without
   judges reads `not judged`.
5. The nits, one line each, with `path:line` and the three labels.
6. The issues filed.
7. The refuted and dropped findings, one line each, with the reason.
