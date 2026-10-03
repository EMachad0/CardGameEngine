# Decision records

One file per decision, numbered in sequence: `0001-slug.md`, `0002-slug.md`, and so on. Format
rules live in the `domain-modeling` skill, in `ADR-FORMAT.md`.

A decision earns a record only when it is hard to reverse, surprising without context, and the
result of a real trade-off. A choice that misses one of the three gets no record.

A record states the decision as it stands, written for a reader who never saw the session behind
it. To change a decision, write a new record that supersedes the old one, and set the old one's
status to `superseded by ADR-NNNN`. Never rewrite an old record to match the present.

## Writing and committing

When a decision clears the bar, the agent writes the record and commits it on its own, without
asking first or waiting for review. This overrides the git rules in `CLAUDE.md` for that commit.
It stages the new record and, when the record supersedes one, the old record's status change.
Nothing else.
