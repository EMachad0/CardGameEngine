---
status: accepted
---

# Targets are chosen in a draft held in the game

A card whose effects need chosen targets is played over several actions. Playing it opens a draft.
The player then makes one pick per target the effects ask for, in effect order, and an explicit
commit pays the cost and resolves the card. The player can cancel during the draft, and nothing is
paid and nothing leaves the hand before the commit. A card is offered for play only when its draft
can be finished. The rule for each target lives in the effect that uses it, and the draft is hidden
from the opponent.

## Considered options

- A target rule on the card and one target slot in the play action. It can't express two picks,
  such as two minions, or a minion and a hero.
- One play action per combination of card and targets. The list multiplies with every pick.
- Committing on the last pick. Auto-play can skip an explicit commit later, while splitting one
  action into two later would change the interface.
