---
status: accepted
---

# Each player's hero is an object

A player makes the decisions and owns the mana, the zones and any pending pick. The hero is the piece
that represents the player and takes damage. Each player's hero is an object with a hero card
definition, held in a hero zone that always holds exactly one object. Damage to a hero is marked on it
and its health is computed on read, as for a minion. So a damage effect takes one path for every
character, and events name a hero by its `ObjectId`.

This follows Hearthstone, where damage marks a hero the way it marks a minion. A game where damage to
a player works differently, such as MTG's life loss, keeps its players out of the objects.

## Considered options

- Health on the player, and targets that are either a player or an object. Every verb that hits
  characters needs a branch for each side, and buffs on a hero need a second home.
- A shared supertype for players and cards, with damage as a method each side implements, as in
  Forge's `GameEntity`. It suits rules where the two sides take damage differently.
- One id space for players and objects, with each verb looking an id up on both sides, as in XMage's
  UUIDs.
