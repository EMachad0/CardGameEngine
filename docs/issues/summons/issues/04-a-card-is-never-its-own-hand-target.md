# A card being played is never one of its own hand-target options

Status: ready-for-agent
Category: enhancement
Type: task

## Why

While a card is targeted it is still in its player's hand, so a hand target can offer the card
being played. The rule is that it never does: the card is left out of its own options, and it is
not moved out of the hand while targeting. MTG reaches the same place by moving the card to the
stack first (601.2a) and by 115.5, "A spell or ability on the stack is an illegal target for
itself." Recycle's options are minion cards only, so no card reaches the rule yet.

## What

The card being played is never a candidate for its own hand targets, in the targeting walk, in
whether its targeting can finish, and in the view of an open targeting.

## Forcing card

Tinker, a test spell: "Bury a card from your hand. Draw a card." Like Recycle, with a hand target
over every card kind.

## Tests

- Tinker is not offered as one of its own options.
- Another Tinker in the same hand is offered.
- With no other card in hand, Tinker can't be played, since its targeting can't finish.

## Done when

- The tests pass, and every existing test passes unchanged.
- The checks pass.
