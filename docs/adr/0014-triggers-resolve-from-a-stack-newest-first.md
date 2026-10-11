---
status: accepted
---

# Triggers resolve from a stack, newest first

Matched triggers wait on a stack in `Game`, and the newest resolves first. Whatever a resolving
trigger causes is pushed on top of everything already waiting, so it resolves first: if A happened
because of B, A resolves before anything that was waiting when B resolved.

A dying object's own death triggers are pushed before its death is reported. The triggers that react
to the death are pushed after, so they resolve first: a Mourner draws before the dead Bomber's
deathrattle. Triggers matched by one event are pushed in order of play, oldest first, so the newest
resolves first. Any other order follows the order in which the triggers were matched, and nothing
promises it.

This adds to ADR 0012, which decides what the matcher sees and when the state check runs. It
supersedes ADR 0013, which resolved the triggers matched by one event oldest first.

## Considered options

- The triggers matched by one event resolved oldest first, as Hearthstone's order of play does. On a
  stack, that means pushing them newest first, against the order the stack gives everything else.
  Legends of Runeterra puts skills on its stack in board order, so the rightmost resolves first.
- A queue, oldest first. What a trigger causes waits behind triggers that were already waiting.
  Hearthstone, MTG and Legends of Runeterra all resolve consequences first.
- Sorting every waiting trigger by order of play, as Hearthstone does. Combined with consequences
  first, it needs inserts at both ends of the queue.
- The dying object's own death triggers before the triggers that react to its death, as Shadowverse
  and KeyForge order them.
