---
status: superseded by ADR-0025
---

# A played minion's summon reactions resolve after its play

Playing a minion takes two steps (ADR 0018): the play, which pushes the card's play effects and
reports the play, then the summon. The summon is matched in its own step, as every event is, but its
matches are pushed under everything the play pushed: the play effects and the reactions to the
play. So they resolve in Hearthstone's order: "whenever you play" reactions, then the play effects,
then the reactions to the summon, such as Knife Juggler's. Legends of Runeterra also resolves a
unit's Play effect before the reactions to its summon.

This adds to ADR 0014. Every other match is pushed on top when it is matched.

## Consequences

- A minion the play effects summon was not there when the played minion's summon was matched, so it
  doesn't hear it. Its own summon is matched on top, so its reactions resolve before the played
  minion's.
- A matched summon reaction resolves even if its own object or the summoned minion died in the play
  effects, as every matched trigger does. Hearthstone fires no knife in either case.

## Considered options

- Pushing on top, as before. The summon reactions resolve first, before the reactions to the play
  and before the play effects: the reverse of Hearthstone.
- Reporting the summon once the play effects finish. The step that summons would not report its own
  event (ADR 0018), a summon whose minion died would need a matching rule, and a minion the play
  effects summon would hear it, as in Hearthstone before patch 9.2.
- Matching at the summon and checking again on resolving that the summoned minion and the reacting
  object are still in play, like MTG's intervening "if" (603.4). It follows Hearthstone in every
  case, but no other trigger checks again.
- Reactions to the play after the play effects, as Hearthstone's "after you play". Drillmaster's
  Squire would come after the spell it reacts to.
