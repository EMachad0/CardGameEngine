# Reviewing a lane

You review one lane of a change. The task names your lane's brief, the review range, the branch
range, the spec, and the ledger of findings earlier rounds decided.

## Scope

- Report what the review range changes or makes reachable. The rest of the branch range is
  context. A defect you find outside the review range is pre-existing. Report it as such.
- Check only what your brief names. It also says which lane covers the rest.
- Skip what the checks enforce. They pass on every commit you review. `.just/check.just`,
  `[workspace.lints]` in the root `Cargo.toml`, and each `clippy.toml` say what they cover.
- Don't report a finding the ledger already decided, unless you have new evidence against its
  outcome.

## Evidence

Look for defects, not for a verdict on the change. Report every finding you can prove. A verifier
checks each one before anyone acts on it, so a doubtful finding with proof costs less than a missed
one.

Drop a finding you can't prove, and a preference no rule backs. The proof is one of:

- for behaviour, a test or a sequence of calls that shows it
- for the spec, the spec's line, quoted
- for a rule, the doc's sentence, quoted word for word, and why it applies to these lines

When the same flaw appears in several places, report it once and list every place.

## Report

One entry per finding:

```markdown
### <short title>

- Lane: <lane>
- Where: <path>:<lines>, one per place
- Pre-existing: yes | no
- What: <one or two sentences>
- Proof: <as above>
- Fix: <the smallest change that removes it>
```

If nothing qualifies, reply `No findings.` Write nothing else, no summary of the change and no
praise.
