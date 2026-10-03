# Card game systems course

A course on card game systems, taught one node at a time. `docs/course/COURSE.md` holds the goal,
the session flow, the knowledge map and the decisions so far.

## Tech stack

Rust, edition 2024. The root `Cargo.toml` is a virtual workspace manifest, with members under
`crates/`.

## Common commands

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`

"The checks" below means all three, in that order.

## Required reading

These docs are not background. When one applies, read the whole thing before writing any code.
They set the conventions you follow, not material for you to summarize.

- Every session: `docs/course/COURSE.md`. It says who writes what in an exercise, and where the
  learner's edge is.
- Before writing or changing anything in `crates/rules`, tests included: `crates/rules/SPEC.md`.
- Before writing, moving or deleting any test, or designing code that has to be testable:
  `docs/testing.md`.

## Code comments

Default is no comment. Docstrings come before inline comments. If the explanation is durable, it
belongs in a docstring, not above a line.

When a comment is warranted:

- Professional tone. No chatty asides ("and friends", "gotcha"), no editorializing, no narration
  of the change being made.
- Follow the pattern already in the file and the module. If sibling functions carry no docstring,
  the new one gets none either.
- Concise. When rewriting an existing comment, make it shorter, not longer.
- Only what reading the code cannot tell you. A comment restating the next line is noise.
- Do not couple a comment to code that is not directly below it. No comment that describes a
  caller, a sibling module, or a future edit.
- The comment must stand on its own and cost nothing to keep accurate. A comment that goes stale
  the next time nearby code moves is a bad comment. Drop it instead.
- Never reference agent tooling or untracked files: anything under `.pi/` or `.agents/`, handoff
  docs, plans. A handoff or plan doc claiming an exception is not license. Two references are
  allowed because they tie a test to what it checks: the crate's `SPEC.md`, and course node labels
  (R1, L, P) from `docs/course/COURSE.md`. Never reference session logs.
- No em dashes, en dashes, or arrows.

These rules cover all new text, not only inline comments: docstrings, assertion messages, test
comments, TOML `#` comments and `clippy.toml` reasons. Text moved from another file counts as new
text and gets restyled during the move.

## Always check your work

After writing or changing any code, run the checks and confirm they pass. That is how you verify
the code matches your intent. The work is not done until they are green.

## Git rules

- Never commit your own code before I have reviewed it. Present the work for review first. The
  per-node commit from `docs/course/COURSE.md` follows this rule too: my review is the gate.
- Run the checks and confirm they pass before you commit.

## Plan before implementing

Implementation is the last step, never the first. Settle the shape of the work and every open
decision with me before writing code.

Never rush into implementation carrying uncertainty. If a choice has more than one defensible
answer and picking wrong means rework, stop and ask me. Do not pick silently and report the
assumption afterwards. Unknowns that touch one call site only are yours to decide. Anything
shaping an interface, a schema, a stored format, or a policy is mine.

Once the plan is settled, work test-first. In an exercise, the roles in `docs/course/COURSE.md`
decide who writes what: I write the design-bearing code, and you write the tests against my API
before I implement it. For code you own outright (scaffolding, tooling), use the `tdd` skill: red,
green, refactor. No permission checkpoints inside it.

## Agent skills

Most skills are installed outside this repo and read their per-repo configuration from
`docs/agents/`. One ships here instead.

### Teach

Runs the course sessions. It lives in `.agents/skills/teach/`. The tools and subagents it calls are
the pi extensions and agents under `.pi/`.

### Domain docs

Single context: `docs/CONTEXT.md` for the glossary, `docs/adr/` for decisions. The course material
lives in `docs/course/`. The skills look for a glossary at the repo root by default. Never put one
there, and never split the glossary per directory. See `docs/agents/domain.md`.
