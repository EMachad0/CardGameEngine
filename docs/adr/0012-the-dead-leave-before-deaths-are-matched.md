---
status: accepted
---

# The dead leave before their deaths are matched, and the state check runs after each trigger

The state check finds every object at 0 Health or less, reads each one's own death triggers as it is
at that moment, removes them all together, then emits a death event for each in order of play. The
matcher therefore sees a board without the dead: a removed object hears no other death, and only its
own death triggers, read before removal, resolve for it. After each trigger resolves, the state
check runs again. This repeats until no trigger is waiting and nothing has died.

Order of play is the order objects entered the board, kept as a stamp set at each entry. It orders
the deaths of one pass and the triggers matched by one event.

This is Legends of Runeterra's model as its community documents it. Hearthstone, KeyForge and
Shadowverse also keep a dying watcher from hearing the others. MTG, Lorcana and Flesh and Blood also
check state after each trigger.

## Considered options

- Matching before removal, as MTG (603.10a) and Lorcana (7.4.3) do. A watcher that dies in the same
  pass hears every other death.
- Checking state only after the whole queue resolves, as Hearthstone does. A minion at 0 Health
  keeps triggering on later hits, as Acolyte of Pain draws three cards from three deathrattles.
- Removing one dead object per pass and running its death effects inside the pass. Whether a death
  effect saves another dying minion depends on board order.
- Ordering by `ObjectId`. Ids are allocated in deck-list order at setup, not in the order cards are
  played.
