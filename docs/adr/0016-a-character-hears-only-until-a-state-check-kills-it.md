---
status: accepted
---

# A character hears events only until a state check kills it

A character hears an event only while no state check has killed it: a minion while it is on a board,
a hero while its player is in the standings. A hero never leaves its zone, so a dead hero is still in
play, and hearing is a separate question from being in play. The state check removes its dead
minions and eliminates the players of its dead heroes before it reports any death of the pass, so no
character that dies in a pass hears a death from it. Its own death rattles are read before, as ADR
0012 says.

This adds to ADR 0012, which states the same rule for minions only. Hearthstone also removes a dead
hero from play with the minions, and MTG removes a departed player's objects and drops their triggers
(800.4a, 800.4d).

## Consequences

No two-player game reaches the hero half. A hero's death ends the game in the same state check, and
nothing resolves after the outcome, so a test needs more than two players.

## Considered options

- Hearing every character in play. A dead hero hears every later event, and its own death matches its
  death rattles a second time.
- Hearing every character with Health above 0. A minion at 0 Health stops hearing before the state
  check finds it, unlike Hearthstone's mortally wounded minions: Acolyte of Pain draws at 0.
- Skipping the dead of a pass only while that pass reports its deaths. A dead hero still hears every
  other event.
- Moving a dead hero out of its zone, as Hearthstone and Fireplace send it to the graveyard. The hero
  would stop answering its Health.
