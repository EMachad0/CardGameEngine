# Skills and docs

## When to use

Read this when you are:

- Creating or changing a doc, a skill, the glossary, an ADR, or a file in `docs/course/`
- Deciding where a piece of knowledge belongs

## Kinds of knowledge

Each kind has one home, and nothing is written in two places.

### Tests (`crates/<crate>/tests/` and each module's `mod tests`)

The source of truth for what the code does. There is no spec document. A rule no test checks is not
a promise. To keep one, write the test. Don't restate a rule in prose, in a doc or in a test
comment. The test code is the statement. `docs/testing.md` governs the tests.

### Docs (`docs/`)

Conventions to follow while working, such as `docs/testing.md` and this doc. CLAUDE.md says when to
read each one.

### Skills (`.agents/skills/` and `~/.agents/skills/`)

Workflows invoked as `/skill-name` or matched by their description. `.agents/skills/` holds the
ones this repo owns, `~/.agents/skills/` the ones every repo shares.

### Glossary (`docs/CONTEXT.md`)

One canonical name per domain concept, plus the aliases to avoid. Terms only. No rules, no
decisions, no implementation details. `docs/agents/domain.md` governs it, and the format lives in
the `domain-modeling` skill, in `CONTEXT-FORMAT.md`. This repo adds a `## Flagged ambiguities`
section at the end, for pairs of terms that get confused. They never go under `_Avoid_`.

A term resolves when the learner names it, in his code or in a session. When the code and the
course name one concept differently, the pair goes under Flagged ambiguities until he picks the
name. Write a term the moment it resolves.

### Decision records (`docs/adr/`)

Decisions that clear the bar in `docs/adr/README.md`, which also says who writes and commits them.

### Course (`docs/course/`)

The learner and the course plan. The game's design lives in the tests, the glossary and the ADRs.

- `COURSE.md`: the goal, how sessions work, the knowledge map, the course plan, the open threads,
  the dependency map and the sessions table. The knowledge map may cite the learner's code as
  evidence of what he holds and misses.
- `verified-facts.md`: facts checked against a source, each with that source.
- `sessions/`: one log per session, never edited after the session ends.
- `research/`: researcher reports, kept in full.

## Where a finding goes

| Finding | Home |
|---|---|
| What the code does | A test. Write one if none checks it yet |
| What the code must do but doesn't yet | A red test in the exercise that builds it. Until then, the `COURSE.md` open thread or session that plans it |
| A name for a concept, or a name to stop using | Glossary |
| A design choice that clears the ADR bar, built or not | ADR |
| A design choice below the bar, already built | Nowhere. The code and the tests show it |
| A design choice below the bar, not built yet | The `COURSE.md` open thread or session that builds it |
| What the learner holds and misses, with the evidence | `COURSE.md` knowledge map |
| What the course covers next, and why | `COURSE.md` |
| A fact checked against a source | `verified-facts.md` |
| A convention to follow while working | The doc for its topic, else CLAUDE.md. A teaching convention goes in the teach skill |

## Consistency rules

- Docs must not contradict each other. Each concept has one owner doc, and the others point to it in
  one line.
- When a skill and a doc disagree, the doc wins on conventions and the skill wins on its own
  workflow.
