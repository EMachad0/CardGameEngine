# Verifying a finding

You check one finding from a review lane. Try to refute it. The task gives the finding, the review
range, the branch range and the spec.

## Confirm only when it holds

- A behaviour finding: write a test that fails on the current code for the reason the finding
  gives, and run it. Put it where `docs/conventions/testing.md` would, so it can stay as the
  regression test. A finding you can't reproduce is refuted.
- A spec finding: the spec says what the finding claims, and the code doesn't do it.
- A rule finding: the quoted sentence exists word for word in the doc it names, and it applies to
  these lines on its own terms. No other sentence in the repo's docs allows what the code does,
  and no check already enforces it. `.just/check.just`, `[workspace.lints]` in the root
  `Cargo.toml`, and each `clippy.toml` say what the checks cover.

For a finding that lists several places, check each place and drop the ones that don't hold.

## Pre-existing

A flaw the review range neither adds nor makes reachable is pre-existing. For a behaviour finding,
run your test at the start of the review range too. If it fails there for the same reason, the flaw
is pre-existing.

## Report

The first line is `Verdict: confirmed`, `Verdict: refuted` or `Verdict: pre-existing`. Then the
evidence:

- confirmed: the places that hold, and for a behaviour finding the test's full code and its failing
  output
- refuted: what contradicts the finding, with `path:line` or the quoted sentence
- pre-existing: what shows the flaw was there before the range

Write nothing else.
