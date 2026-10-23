---
status: accepted
---

# A step returns its events, and the game reports them after it

An action runs in steps, and each effect of a resolving effect is one. A step writes only the parts
of the game that `Game` lends it, such as the zones or the effect stack, and returns the events it
caused, in order. It holds no observer and no history. Once the step ends, `Game` reports each
event: it matches the event against the triggers in play, records it in History, tells the observer
and pushes the matches. Then the observer reads the views, so a checkpoint follows every step, and
nothing else takes one. A state check pass that finds no dead, and a decision that doesn't end the
game, are no step: they write nothing, report nothing and take no checkpoint.

Matching therefore sees the state the step leaves, not the state at the moment of each event.
Wherever a later write in a step would change what an earlier event matches, the step ends and
another begins. Playing a minion is two steps, so the minion is not on the board when its own play
is matched. A state check pass is one step, since it reports its deaths only after all of its dead
have left (ADR 0012).

## Consequences

- A new trigger pattern has to be checked against every step that reports its event. No pattern
  matches a draw, a turn change, a reveal, a zone move or a board entry, and damage changes nothing
  a match reads, so the steps that report several of these are safe. A pattern on a board entry
  would let the second minion one summon effect makes hear the first one's entry.
- No step takes the observer. ADR 0006's decision stands, but not its consequence that every
  function that can report takes the observer.

## Considered options

- Commands that hold the whole game, as before. Any of them could write any part of it, and one
  that reports or checkpoints holds everything the matcher and the views read.
- A reporter the command lends its own borrows at each report. Each command needs a reporter shaped
  for what it holds, and still holds what the reporter reads.
- Borrows checked at run time, such as `RefCell`, which turn a build error into a panic.
- A hand-written state machine per command, paused at each report. Each command is rewritten as
  states, and its progress between reports is stored by hand.
- Matching every event once the action ends. A played minion would hear its own play.
