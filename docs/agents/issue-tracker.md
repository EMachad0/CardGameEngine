# Issue tracker: local markdown

Issues live as markdown files in `docs/issues/`, committed with the code. They work offline, and
`rg` finds them.

## What goes here

A defect or chore found outside the current task, with a known fix and no design call. A finding
that needs a design call or belongs in a course session is a `COURSE.md` open thread instead.
`docs/conventions/skills-and-docs.md` owns that split.

## Layout

```
docs/issues/
├── <slug>.md                  one standalone issue
└── <effort-slug>/             one to-spec, to-tickets or wayfinder effort
    ├── spec.md or map.md
    └── issues/<NN>-<slug>.md
```

- Name a standalone issue by slug only, never by number. Parallel branches would collide on numbers.
- Number the tickets inside an effort folder from `01`. One session writes them.
- Where a skill names `.scratch/`, use `docs/issues/` instead.

## Standalone issue format

```markdown
# <Title, in the glossary's vocabulary>

Status: needs-triage
Category: bug

<What happens, how to reproduce it, and what should happen instead.>
```

- `Status:` holds one state from `docs/agents/triage-labels.md`. File a new issue as
  `needs-triage`, or as `ready-for-human` when the fix belongs in the learner's design-bearing
  code. The roles in the course's `COURSE.md` say which code that is.
- `Category:` is `bug` or `enhancement`.
- Comments go at the bottom, under `## Comments`.

## Filing

1. Search every local branch first, since another worktree may have filed it:
   `git grep -il <term> $(git for-each-ref --format='%(refname:short)' refs/heads) -- docs/issues/`.
2. Write the file on the current branch and give it a commit of its own, apart from the task's
   changes.

## Resolving

- The PR that fixes or rejects a standalone issue deletes its file, and the commit message says
  why. `git log --diff-filter=D -- docs/issues/` lists closed issues.
- Effort tickets resolve in place, with `Status: resolved` and an `## Answer`. The PR that finishes
  the effort deletes the folder, after moving any lasting decision to its home.
- `triage` does not write `.out-of-scope/` here. A rejection's reason goes in the deleting commit,
  and in an ADR if it clears the bar.

## When a skill says "publish to the issue tracker"

Create a file under `docs/issues/`, following the layout above.

## When a skill says "fetch the relevant ticket"

Read the file at the referenced path. The user normally passes the path or the slug.

## Wayfinding operations

`/wayfinder` uses these operations. The map is a file with one child file per ticket.

- Map: `docs/issues/<effort>/map.md`, holding the Notes, Decisions-so-far and Fog body.
- Child ticket: `docs/issues/<effort>/issues/NN-<slug>.md`, with the question in the body. A
  `Type:` line records the ticket type: `research`, `prototype`, `grilling` or `task`. A `Status:`
  line records `claimed` or `resolved`.
- Blocking: a `Blocked by: NN, NN` line near the top. A ticket is unblocked when every file it
  lists is `resolved`.
- Frontier: the open, unblocked, unclaimed files in `docs/issues/<effort>/issues/`, lowest number
  first.
- Claim: set `Status: claimed` and save before any work.
- Resolve: append the answer under `## Answer`, set `Status: resolved`, then append a pointer to
  the map's Decisions-so-far: a gist and a link.
