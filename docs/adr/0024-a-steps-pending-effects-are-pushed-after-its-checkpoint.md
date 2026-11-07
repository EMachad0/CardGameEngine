---
status: superseded by ADR-0025
---

# A step's pending effects are pushed after its checkpoint

An action runs in steps, and each effect of a resolving effect is one. A step writes only the parts
of the game that `Game` lends it, such as the zones or the effect stack, and returns the events it
caused, in order. It can't write history or reach the observer. Once the step ends, `Game` reports
each event: it matches the event against the triggers in play, records it in history and tells the
observer. Then the observer reads the views, so a checkpoint follows every step, and nothing else
takes one. A state check pass that finds no dead, and a decision that doesn't end the game, are no
step: they write nothing, report nothing and take no checkpoint.

Nothing a step adds goes on the effect stack before its checkpoint. Its pending effects, such as a
played card's play effects, a dying object's own death triggers and the matches of its events, wait
in a list in the order they were added. The step hands the list back after its checkpoint, and the
code that took the step pushes it before the next step. A played minion's play is the one
exception: its list waits through the summon step and goes on top of the summon's (ADR 0023).

Matching therefore sees the state the step leaves, not the state at the moment of each event.
Wherever a later write in a step would change what an earlier event matches, the step ends and
another begins. Playing a minion is two steps, so the minion is not on the board when its own play
is matched. A state check pass is one step, since it reports its deaths only after all of its dead
have left (ADR 0012).

This supersedes ADR 0018, which pushed the matches before the checkpoint.

## Consequences

- A new trigger pattern has to be checked against every step that reports its event. No pattern
  matches a draw, a turn change, a reveal or a zone move, and damage changes nothing a match reads,
  so the steps that report several of these are safe. The minions one step summons hear each
  other's summons, as ADR 0022 decides.
- No step takes the observer. ADR 0006's decision stands, but not its consequence that every
  function that can report takes the observer.
- A checkpoint's views don't hold the pending effects of the step before it. No view shows pending
  effects. One that did would show them a checkpoint late.

## Considered options

- Pushing the matches before the checkpoint, as ADR 0018 did. A played minion's play effects still
  have to wait past the play step's checkpoint, so the play and the summon would end apart from
  every other step.
- Commands that hold the whole game, as before ADR 0018. Any of them could write any part of it,
  and one that reports or checkpoints holds everything the matcher and the views read.
- A reporter the command lends its own borrows at each report. Each command needs a reporter shaped
  for what it holds, and still holds what the reporter reads.
- Borrows checked at run time, such as `RefCell`, which turn a build error into a panic.
- A hand-written state machine per command, paused at each report. Each command is rewritten as
  states, and its progress between reports is stored by hand.
- Matching every event once the action ends. A played minion would hear its own play.
