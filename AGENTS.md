# Card game systems course

Courses taught one node at a time, listed in `docs/course/README.md`. Each course's `COURSE.md`
holds its goal, session flow, knowledge map and plan.

## Tech stack

Rust, edition 2024, toolchain pinned in `rust-toolchain.toml`. The root `Cargo.toml` is a virtual
workspace manifest, with members under `crates/`.

## Common commands

- `just` lists all recipes. The root justfile imports the `.just/` modules.
- `just wk::new_ui <branch>` creates a worktree off origin/main and opens a zellij dev tab. It
  wraps worktrunk. `wk::new` skips the tab; `wk::list` and `wk::rm` inspect or remove.
- `just check::all` runs `cargo fmt --all --check`, then `check::paths`, a text search for the
  import path rules in `docs/conventions/file-structure.md`, then
  `cargo clippy --workspace --all-targets -- -D warnings`, then `cargo test --workspace`. "The
  checks" below means this recipe.

## Worktrees

When a task needs a branch with no worktree, create one with the `wk` recipes. Never use a
harness's or agent tool's own worktree option. It puts the worktree outside the `main.<branch>`
siblings that `wk` expects.

- `just wk::new <branch>` branches off a fresh origin/main. For another base, run
  `mise exec -- wt switch --create <branch> --base <base> --no-cd`.
- `just wk::rm <branch>` removes one. Only when I ask.

The shell stays in `main/`. In T3 Code, launch a thread bound to the new worktree as an existing
worktree, give it the full task and hand me its link. Elsewhere, work in it by absolute path. In
zellij, a task that needs its own worktree and my input goes to another pi session through the
`delegate` skill.

## Required reading

These docs are not background. When one applies, read the whole thing before writing any code.
They set the conventions you follow, not material for you to summarize.

- Every session: the course's `COURSE.md`, picked by `docs/course/README.md`. It says who writes
  what in an exercise, and where the learner's edge is.
- Before creating or changing a doc, a skill, the glossary, an ADR or a file in `docs/course/`, or
  deciding where knowledge belongs: `docs/conventions/skills-and-docs.md`.
- Before writing, moving or deleting any test, or designing code that has to be testable:
  `docs/conventions/testing.md`.
- Before recording a bug or chore found outside the current task: `docs/agents/issue-tracker.md`.
- Before adding, upgrading or removing a dependency, or changing the features a crate enables:
  `docs/conventions/dependencies.md`.
- Before creating, splitting, moving or naming a module, a file or a crate, or deciding where new
  code goes: `docs/conventions/file-structure.md`.

## Writing

Covers every text you write: code comments, docstrings, assertion messages, TOML `#` comments,
`clippy.toml` reasons, docs, skills, commit messages, PR bodies and replies to me.

- Unslop every text. Beyond the `unslop` skill, no arrows.
- The longer a text must stay true while the code changes, the stricter its wording. From loosest
  to strictest: a reply to me, a frozen record (a session log or an ADR) that may cite anything
  current when written, a PR body that describes the branch as it stands, docs and skills, then
  code text, which says only what stays true.
- Unneeded examples are the main way text goes stale. Add one only when the rule cannot be read
  without it. An example of behaviour belongs in a test.
- Professional tone. No chatty asides, no editorializing, no narration of the change being made.
- Keep replies to me focused, brief and concise.
- Concise. A rewrite makes the text shorter, not longer.
- Text moved from another file counts as new. Existing text gets restyled when next rewritten.

### Code comments

Default is no comment. Docstrings come before inline comments, and a durable explanation belongs
in a docstring.

- Only what reading the code cannot tell you. This decides whether an item gets a docstring,
  whatever its siblings carry.
- Do not couple a comment to code that is not directly below it: no caller, sibling module or
  future edit.
- Never reference agent tooling, docs or untracked files: anything under `.pi/`, `.agents/` or
  `docs/`, handoff docs, plans, course node labels. A plan claiming an exception is not license.
- Let the comments already in the file and the module guide what is worth saying. They don't
  decide it.

## Always check your work

After writing or changing any code, run the checks and confirm they pass. That is how you verify
the code matches your intent. The work is not done until they are green.

## Git rules

- Never commit your own code before I have reviewed it, unless I, or a skill or doc you're
  following, tell you to commit or to open a pull request. Otherwise present the work for review
  first. The rule exists only so I can review the diff locally. The per-node commit from the
  course's `COURSE.md` follows it too.
- Beyond that, how and when to commit is your call. GitHub squashes a pull request's commits, so
  don't ask me about messages or how to split them. A commit may hold red tests, clippy errors or
  warnings.
- When you push to a branch that has a pull request, or open one, keep its body up to date. After
  each push, edit it so it describes the whole branch as it stands.

## Plan before implementing

Implementation is the last step, never the first. Settle the shape of the work and every open
decision with me before writing code.

Never rush into implementation carrying uncertainty. If a choice has more than one defensible
answer and picking wrong means rework, stop and ask me. Do not pick silently and report the
assumption afterwards. Unknowns that touch one call site only are yours to decide. Anything
shaping an interface, a schema, a stored format, or a policy is mine.

Once the plan is settled, work test-first. In an exercise, the roles in the course's `COURSE.md`
decide who writes what. For code you own outright (scaffolding, tooling), use the `tdd` skill: red,
green, refactor. No permission checkpoints inside it.

## Agent skills

Most skills are installed outside this repo and read their per-repo configuration from
`docs/agents/`. Some ship here instead, in `.agents/skills/`.

### Teach

Runs the course sessions. It lives in `.agents/skills/teach/`. The tools and subagents it calls are
the pi extensions and agents under `.pi/`.

### Review loop

Reviews a branch in parallel lanes, fixes the findings its judges pass, commits each round,
then pushes and opens a pull request. AFK work always ends with it, in place of the `/code-review`
step that `implement` names. It lives in `.agents/skills/review-loop/`, and its subagents are the
agents under `.pi/`.

### Delegate

Starts a separate pi session in a new worktree and zellij tab for a task that needs its own
worktree and my input. It lives in `.agents/skills/delegate/`.

### Issue tracker

Local markdown in `docs/issues/`. See `docs/agents/issue-tracker.md`.

### Triage labels

The five default roles, written on each issue's `Status:` line. See `docs/agents/triage-labels.md`.

### Domain docs

Single context: `docs/CONTEXT.md` for the glossary, `docs/adr/` for decisions. The course material
lives in `docs/course/`. The skills look for a glossary at the repo root by default. Never put one
there, and never split the glossary per directory. See `docs/agents/domain.md`.
