---
status: accepted
---

# A card's reactions are triggers, derived on read and queued in the game

A trigger is an event pattern plus effects. Everything a card does in reaction to an event is a
trigger: entering the board, leaving it, its own death, and events about other objects. A card's
play effects, what playing it from the hand does, are not a trigger.

Which triggers an object has is computed on read, the way its modifiers are: its definition's
triggers plus those granted by auras that reach it. Nothing records them as they come and go. One
emit function records the event in History, matches it against the triggers of the objects in play,
and reports it to the observer. Each match is queued in `Game` with its effects copied at match
time, and resolves later.

## Considered options

- A registry that adds an object's triggers when it enters the board and removes them when it
  leaves, as SabberStone and Metastone do. Every other input of "which triggers does this object
  have" needs its own patch: a silence, an aura that grants a trigger to minions played after it, a
  change of control. XMage never prunes its registry and re-checks each ability at every event.
- Play effects as a trigger on the card's own `Played` event. They would share the queue with
  "whenever you cast a spell" triggers and need a rule to resolve last, and a draft's choices belong
  to playing the card, not to a trigger.
- Resolving each match at the event. A trigger's effects would run in the middle of another effect.
- The queue outside `Game`, in a local or in the observer. A trigger whose effect reveals cards
  pauses `apply` with other matches still waiting, and a bot's clone has no observer.

## Consequences

Every event scans the objects in play. A cache is added only after a measurement shows the scan
matters.
