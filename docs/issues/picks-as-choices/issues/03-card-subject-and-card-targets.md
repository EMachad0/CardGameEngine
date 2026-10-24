# The cards subject, with targets in a graveyard

Status: ready-for-human
Category: enhancement
Type: task
Blocked by: 02, and card-game session 04g

## Why

Verbs act only on players and characters, so no card text can act on a card in a hand, a deck or
a graveyard. ADR 0019 adds a third subject, cards, whose targets reach the chooser's hand or a
graveyard.

## Forcing card

Raise Dead: "Return a minion card from your graveyard to your hand." The minion card is a target.

## What

From ADR 0019, limited to what Raise Dead reaches:

- `Effect::Cards { selector: CardSelector<S>, verbs: Vec<CardVerb> }`.
- `CardSelector<S>` with `Targeted(S::CardTarget)`. `TargetForm` gains `CardTarget`: `Declared`
  holds `Choice<CardTargetOptions>`, `Triggered` holds `NoTarget`, and `Bound` holds
  `Vec<ObjectId>`.
- `CardTargetOptions { zone: CardTargetZone, kind }`, with `CardTargetZone::Graveyard(sides)`.
- `CardVerb::Move(CardDestination)` with `CardDestination::Hand`, into the card owner's zones.
- Targeting offers graveyard cards: the targeting walk in `game/targeting_walk.rs`, the legal
  actions and the view's targeting.

## Tests

- Raise Dead moves the targeted minion card from its owner's graveyard to the hand.
- With no minion card in its player's graveyard, Raise Dead isn't playable.
- A spell card in the graveyard isn't offered.
- A minion card in an opponent's graveyard isn't offered.
- The view of an open targeting shows the targeted graveyard card to every player.
- `Cancel` leaves the game as it was before the targeting started.
- The move is reported as an event.

## Done when

The checks pass, and the agent's review against `docs/conventions/file-structure.md` is answered.
