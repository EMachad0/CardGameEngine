# The cards subject, with targets in a hand

Status: ready-for-human
Category: enhancement
Type: task
Blocked by: 02

## Why

Verbs act only on players and characters, so no card text can act on a card in a hand, a deck or
a graveyard. ADR 0019 adds a third subject, cards, whose targets reach the chooser's hand or a
graveyard. This ticket builds the hand target, since the engine already reads, moves and shows
hand cards.

## Forcing card

Recycle: "Put a minion card from your hand on the bottom of your deck. Draw a card." The minion
card is a target.

## What

From ADR 0019, limited to what Recycle reaches:

- `Effect::Cards { selector: CardSelector<S>, verbs: Vec<CardVerb> }`.
- `CardSelector<S>` with `Targeted(S::CardTarget)`. `TargetForm` gains `CardTarget`: `Declared`
  holds `Choice<CardTargetOptions>`, `Triggered` holds `NoTarget`, and `Bound` holds
  `BoundCards { zone, object_ids }`, so a move takes the card from the zone it was targeted in.
- `CardTargetOptions { zone: CardTargetZone, kind }`, with `CardTargetZone::Hand`.
- `CardVerb::Move(CardDestinationZone)` with `CardDestinationZone::DeckBottom`, into the card
  owner's zones. Moving a card there is a bury.
- Targeting offers hand cards: the targeting walk in `game/targeting_walk.rs`, the legal actions
  and the view's targeting.

## Not in this ticket

- The graveyard target. Its gaps are an open thread in `docs/course/card-game/COURSE.md`.
- The card being played as one of its own options. The open thread on hand targets in
  `docs/course/card-game/COURSE.md` holds the call.

## Tests

- Recycle puts the targeted minion card on the bottom of its owner's deck.
- Recycle draws the top card of its caster's deck.
- The bury is reported as an event, after the play and before the draw.
- With a minion card in its player's hand, Recycle is offered as a targeting.
- With no minion card in its player's hand, Recycle isn't playable.
- A spell card in the hand isn't offered.
- A minion card in an opponent's hand isn't offered.
- The view of an open targeting shows the targeted hand card to every player.
- The opponent sees no face on the targeted hand card.
- `Cancel` leaves the game as it was before the targeting started.

## Done when

The checks pass, and the agent's review against `docs/conventions/file-structure.md` is answered.
