---
status: superseded by ADR-0010
---

# Choices are declared on the card and made one at a time in a draft

A card that needs targets declares its choices as a list on its definition. Each choice has an id,
a filter, a count, and whether its targets must differ from those of every earlier choice. An effect
refers to a choice by its id, so several effects can act on one choice.

Playing such a card takes several actions. `Draft` opens a draft held in the player's state.
`Choose` adds one target to the first choice in declaration order that isn't filled yet. `Cancel`
drops the draft and leaves the game as it was before `Draft`. `Play` pays the cost and resolves the
card once every choice is filled. A card with no choices is played with `Play` alone. `Draft`, and
each `Choose`, is offered only if every remaining choice can still be filled, which a search over
the choices decides. The draft is hidden from the opponent.

This supersedes ADR 0007, which put the rule for each target in the effect that uses it.

## Considered options

- The rule for each target in the effect that uses it. Two effects can't act on one choice, as in
  "Choose a minion. Give it +1/+1. If it's a Pirate, give it Haste."
- A target rule on the card with one target slot in the play action. It can't express two choices.
- One play action per combination of card and targets. The list multiplies with every choice.
- Choices filled in any order, with each `Choose` naming its choice. When two filters overlap, the
  shell has to decide which choice a click fills. MTG, Forge, XMage and Hearthstone's Barbed Nets
  all ask in order.
- Committing on the last choice. A shell can send `Play` automatically later, while splitting one
  action into two later would change the interface.
- Checking only when the draft opens. A target that matches its filter can still leave a later
  choice with no candidate, as in "a minion, then a different friendly minion" with one friendly
  minion on the board.
- Offering every draft and letting `Cancel` recover, as MTG reverses a cast that can't be completed.
  `legal_actions` would list actions that lead nowhere.
