# Summons: Knife Juggler, minions summoned together, and summon reactions after the play

## Goal

Build ADR 0022 and ADR 0025 in `crates/rules`, with Knife Juggler as the forcing card, and build
the rule that a card being played is never one of its own hand-target options.

Today no trigger pattern matches a summon. A played minion's summon is matched after its play,
and its matches are pushed on top, so they would resolve before the reactions to the play and
before the play effects. A character filter matches the object whose text it is whenever that
object passes it, and card data has no way to say "another".

When the effort finishes:

- A trigger can react to a summon, and minions summoned in one step hear each other's summons.
- A character filter can leave out the object whose text it is.
- A card filter can leave out the object whose text it is.
- A played minion's summon reactions resolve after its play effects and the reactions to its play.
- A card being played is never one of its own hand-target options.

## Tickets

| # | Ticket | Blocked by |
|---|---|---|
| 01 | `issues/01-another-on-a-character-filter.md` | |
| 02 | `issues/02-knife-juggler-and-the-summon-pattern.md` | 01 |
| 03 | `issues/03-summon-reactions-after-the-play.md` | 02 |
| 04 | `issues/04-a-card-is-never-its-own-hand-target.md` | |

## Before any code

Read in full: `AGENTS.md`, `docs/conventions/file-structure.md`, `docs/conventions/testing.md`,
`docs/CONTEXT.md`, ADRs 0014, 0016, 0021, 0022 and 0025, this spec and your ticket.

## Roles

An agent builds every ticket. It picks the interface and naming choices a ticket leaves open and
records them under "Choices" in its PR body. A rules call the ticket doesn't settle stays out,
under "Left out" in the PR body. Each ticket's tests come before its implementation.
