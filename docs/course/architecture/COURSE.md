# Codebase architecture course

## Goal

Know where code goes in a project that doesn't fit a framework layout such as Rails' MVC. Design
patterns are out of scope, since he knows them. Topics he named: domain-driven design, data-driven
design (which meaning is still open), ECS for the Bevy front end, wrapping dependencies, ports and
adapters, and test-driven development.

The deliverable is a set of convention docs that say where each kind of code goes, lint rules
wherever a rule can be checked by a tool, and the `rules` crate refactored to follow them. The docs
keep three layers apart: general guidance, Rust guidance, and this project's rules. The general and
Rust layers should hold for his work too: critical finance services in Rust, Python and TypeScript,
after years of Rails. Reuse outside this repo is a nice-to-have. He copies the layers out by hand
if they hold up.

## How sessions work

- Teaching follows the `teach` skill, `.agents/skills/teach/SKILL.md`.
- One pi session per course session. Start with `/new`, `/name NN-topic`,
  `/md-log docs/course/architecture/sessions/NN-topic.md` (the file must exist first), then
  "continue the architecture course".
- Every session opens with a short retrieval quiz on the previous session's nodes.
- Roles. He learns the concepts and makes every decision about the rules. The agent drafts the docs
  and the lint rules, and does the refactor. He reviews all of it before it's committed.
- No new card game features for this course. A concept with no problem in today's code becomes an
  open thread and waits until the card game course creates the problem.
- Project-specific work waits until the refactor PRs in flight land. General and Rust concepts come
  first.
- End at a node boundary, not mid-node. Commit once per node.

## Sessions

| # | Nodes | Topic | Status |
|---|-------|-------|--------|
| 00 | probe | Knowledge probe and plan | in progress |
