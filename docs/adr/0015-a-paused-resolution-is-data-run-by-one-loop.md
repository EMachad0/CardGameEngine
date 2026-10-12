---
status: accepted
---

# A paused resolution is data in the game, run by one loop

When an effect needs a player's answer, `apply` returns, and the work left to do must survive until
the answer arrives. It is stored in `Game` as data: the unit being resolved, one effect sequence for
one source (a card's play effects or one trigger), with its position and its context, held apart
from the stack of waiting units. Pauses happen only between steps, and a step runs to its end
without input: an effect that needs an answer posts the question in one step and uses the answer in
a later one.

One loop resolves everything. It steps the resolving unit until it finishes, then runs the state
check, then takes the next waiting unit, and it stops when the game is over, a question is pending
or nothing is left. The state check and waiting triggers run only between units, so a card that
pauses still resolves as one unit. Who may act is derived from the pending question and never stored
as a fact of its own. Playing a card queues its play effects as a unit before the `Played` event, so
the triggers that react to the play resolve first.

## Considered options

- Blocking a thread on the answer, as Forge, XMage and Metastone do. The paused work lives on the
  call stack, so a copy of the game can't finish it, and their search AIs either replay the card
  from a fresh copy or don't simulate choices.
- Suspending an `async` function or a coroutine, as Spellsource does on fibers. Futures from `async`
  blocks are not `Clone`, so `Game` would no longer hold the whole state.
- Closures stored in the state and keyed by an id, as Jinteki.net does. The state can't be cloned,
  compared or saved, and its maintainers name misplaced async calls as the main source of dropped
  effects.
- Saving only the work tied to the answer, as SabberStone's `AfterChooseTask` and Fireplace's
  `.then()` do. The card's later effects run before the answer arrives.
- Each effect of a card pushed as its own entry on the waiting stack. The state check and other
  triggers then run between a card's effects, unless every entry carries its unit, which rebuilds
  the unit.
