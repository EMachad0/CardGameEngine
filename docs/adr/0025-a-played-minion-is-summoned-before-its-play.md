---
status: accepted
---

# A played minion is summoned before its play

An action runs in steps, and each effect of a resolving effect is one. A step writes only the parts
of the game that `Game` lends it, such as the zones or the effect stack, and returns the events it
caused, in order. It can't write history or reach the observer. Once the step ends, `Game` reports
each event: it matches the event against the triggers in play, records it in history, tells the
observer and pushes the matches. Then the observer reads the views, so a checkpoint follows every
step, and nothing else takes one. A state check pass that finds no dead, and a decision that doesn't
end the game, are no step: they write nothing, report nothing and take no checkpoint.

Matching therefore sees the state the step leaves, not the state at the moment of each event.
Wherever a later write in a step would change what an earlier event matches, the step ends and
another begins. A state check pass is one step, since it reports its deaths only after all of its
dead have left (ADR 0012).

Playing a minion is two steps: the summon, then the play. The summon step moves the minion from the
hand to the board and reports its summon. The play step pays its cost, closes its targeting, adds
its play effects and reports the play. Each step pushes its own pending effects before its
checkpoint, so the newest resolves first with no exception (ADR 0014): the reactions to the play,
then the play effects, then the reactions to the summon. Hearthstone resolves them in this order,
with the minion on the board from the start. Playing a spell is one step.

The minion is on the board when its own play is matched, so a trigger on it hears its own play
whenever its card filter passes it. Card data says "another" with `include_itself: false` on a card
filter, as on a character filter (ADR 0022). In Hearthstone a minion's own "whenever you play"
trigger never fires for its own play.

This supersedes ADR 0023, which pushed the summon's matches under the play's, and ADR 0024, which
pushed every step's pending effects after its checkpoint so the play's could wait for the summon.

## Consequences

- A minion the play effects summon was not there when the played minion's summon was matched, so it
  doesn't hear it, as in Hearthstone since patch 9.2. Its own summon is matched on top, so its
  reactions resolve before the played minion's.
- A matched summon reaction resolves even if its own object or the summoned minion died in the play
  effects, as every matched trigger does. Hearthstone fires no knife in either case.
- History and the observer see a played minion's summon before its play.
- The reactions to the play find the minion on the board and can change it, as Hearthstone's
  Hobgoblin does.
- A new trigger pattern has to be checked against every step that reports its event. No pattern
  matches a draw, a turn change, a reveal, a zone move or a board entry, and damage changes nothing
  a match reads, so the steps that report several of these are safe. A pattern on a board entry
  would let the second minion one summon effect makes hear the first one's entry.
- No step takes the observer. ADR 0006's decision stands, but not its consequence that every
  function that can report takes the observer.

## Considered options

- The play step first, then the summon step, with the summon's matches pushed under the play's
  (ADR 0023). It is the one exception to the stack, and the play's pending effects wait past the
  play step's checkpoint (ADR 0024).
- The summon as an effect pushed under the play's pending effects, as Magic casts a creature spell
  before it enters. The minion is in no zone while the reactions to the play resolve, so none can
  change it. Only Magic counters a minion before it lands. Legends of Runeterra's Deny negates a
  unit's Play skill, and the unit still enters.
- Reporting the summon once the play effects finish, as Legends of Runeterra summons a unit after
  its Play effect. The minion is off the board during its own play effects, and a minion the play
  effects summon hears the played minion's summon, as in Hearthstone before patch 9.2.
- A rule that a card never hears its own play. A filter would include the listener for a summon and
  for damage and leave it out for a play.
- Commands that hold the whole game. Any of them could write any part of it, and one that reports
  or checkpoints holds everything the matcher and the views read.
- A reporter the command lends its own borrows at each report. Each command needs a reporter shaped
  for what it holds, and still holds what the reporter reads.
- Borrows checked at run time, such as `RefCell`, which turn a build error into a panic.
- A hand-written state machine per command, paused at each report. Each command is rewritten as
  states, and its progress between reports is stored by hand.
- Matching every event once the action ends. A minion summoned late in the action would hear every
  event before it.
