---
name: delegate
description: Start a separate pi session in its own git worktree and zellij tab, with a prompt you write, for a task that needs its own worktree and the user's input. Use it unprompted whenever a task meets that rule, and when the user asks to delegate work to another agent. For work whose result comes back to this session, use pi-subagents instead.
---

# Delegate

The child is an ordinary interactive pi session. It reads AGENTS.md and your prompt, nothing else
from this conversation, and the user works with it in its tab. No result comes back to you.

## When

Delegate a task that needs both:

- its own worktree, because it changes code apart from the work in this one;
- the user's input along the way: decisions, questions or review.

Start it without asking. A task that needs neither, or a worktree but no input from the user, goes
to pi-subagents and reports back here.

## Before

1. Check that `ZELLIJ_SESSION_NAME` is set. If not, tell the user the task and stop.
2. Pick a new branch named after the task. Never reuse the current branch or one that
   `just wk::list` shows, since two sessions in one worktree overwrite each other.
3. Pick the base. Independent work starts from origin/main. Work that needs commits not on
   origin/main starts from the local branch that has them. Uncommitted edits never reach the child.

## Start

Write the prompt to `/tmp/<branch>.prompt.md` with the write tool. A heredoc inside `$(...)`
breaks on an apostrophe in macOS bash 3.2. Then, from origin/main:

```bash
just wk::new_ui <branch> --no-focus --prompt "$(cat /tmp/<branch>.prompt.md)"
```

From another base, create the worktree with
`mise exec -- wt switch --create <branch> --base <base> --no-cd`, then run `just wk::zellij` with
the same arguments. Drop `--no-focus` only when the user asks to switch to the tab.

If a start fails, run `just wk::list` before retrying. The worktree may exist without its tab, and
then `just wk::zellij` opens it.

## The prompt

Write it for a reader who starts cold. It holds:

- The goal, and what done looks like.
- Every decision already settled with the user, with the reason when the child might reopen it.
  The child asks the user about the rest in its tab.
- The files, issues or plans to read first, by path.
- Whether the task is AFK. AFK work ends with the review loop; otherwise the child presents its
  work for review.

Leave out what AGENTS.md already says.

## After

Tell the user the branch name, which is also the tab name, and what the child is doing. The tab is
theirs from then on. Don't read its screen or type into it unless asked.
