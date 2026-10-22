---
status: accepted
---

# A target is declared by the effect that uses it, and a played card carries what was targeted

A card's targets are declared inside the selectors of its play effects. A `Targeted` selector holds
the target's filter, count, bound and whether it must differ from every earlier target. The card's
targets are those declarations in effect order, and a target's id is its place among them, computed
and never written in card data. One target feeds one effect.

When the card is played, each declaration is replaced by the objects targeted for it, so the pending
effect carries what was targeted and resolution reads it there. A trigger's effects can't declare a
target, and pushing them maps them into the same type. The effect stack holds one type.

Playing a card with targets still takes several actions. `StartTargeting` opens a targeting.
`Target` adds one object to the first target in declaration order that isn't filled yet. `Cancel`
drops the targeting and leaves the game as it was before. `Play` pays the cost and plays the card
once every target is filled. A card with no targets is played with `Play` alone. `StartTargeting`,
and each `Target`, is offered only if every remaining target can still be filled, which a search
over the declarations decides. Every player's view shows an open targeting: the card's id and the
objects targeted so far. The card's face stays hidden from the opponent until it is played.

This supersedes ADR 0010, which declared targets as a list on the card that effects named by id.
The list and the ids pointing into it had to agree, so card data counted positions by hand and
loading checked every id. Resolution read targeted objects from player input keyed by the card, so
two resolutions from one source would share them, while MTG keeps an ability on the stack
independent of its source (113.7a). Play effects and trigger effects sat on the stack as two types.

## Consequences

- One target can't feed two effects, as in "Choose a minion. Give it +1/+1. If it's a Pirate, give
  it Haste.", the case that moved targets onto the card in ADR 0009. The planned way back is one
  effect applying several verbs to one selection, not built yet.
- Targets are asked in the order their effects resolve. A card can't ask for a target first and act
  on it last.
- A pending effect holds objects, not the declaration. A re-check of targets at resolution, as in
  MTG 608.2b, would bind both.

## Considered options

- A list on the card that effects name by id, ADR 0010.
- Names or builder handles instead of positions. Declaration and use stay apart, so loading still
  checks every reference.
- A back-reference to an earlier effect's target, as Forge's `Defined$ Targeted`. It keeps reuse and
  brings back a reference to check.
- Targeted objects in player input keyed by the pending effect. Nothing collides, but resolution
  still looks them up, and the stack still holds two types.
- An address into the definition plus the answers instead of bound effects. It is smaller and
  serializable, but resolution finds a target's answers by position again.
- Separate types for written effects and stack effects. They differ only in what `Targeted` holds,
  so every verb would be written in both.
