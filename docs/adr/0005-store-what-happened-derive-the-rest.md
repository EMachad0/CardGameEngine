---
status: accepted
---

# Store what happened and derive the rest

`Game` stores only what can't be recomputed: the damage marked on each minion, buffs given by
effects, and a history of what happened, such as each card played and each minion that died, with
its turn. Attack, health and cost are computed from that and the printed data on every read, with
no cache. A buff ends the moment its source leaves, a card drawn late counts everything that
happened before it, and no stored copy of a computed value can go stale.

Deaths and the end of the game are committed instead, because they have consequences. The state
check at the end of every action removes every minion at 0 health or less at once, repeats until a
pass removes none, then decides the outcome.

## Considered options

- A stored number per value, such as attack or a spells-cast counter, kept up to date by hooks. It
  reads fast, but every hook is one more place to forget, and a value that depends on another card
  goes stale when that card leaves.
- A modifier pushed onto cards when something happens. A card that enters the game afterwards
  misses it.
- Deriving deaths as well. Whether one minion is dead would depend on whether others are removed, a
  cycle with no single answer.
