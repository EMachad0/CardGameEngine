---
status: accepted
---

# Card definitions have versioned codes, and each card in a game is an object

A card definition is identified by a stable string code that carries a version, such as
`base.bolt.v0`, in one append-only table. Changing a card adds a new version and leaves the old one
in place. A game recorded before the change replays with the definitions it was played with, and
the replay needs no data release number.

Each card in a game is an object with its own id, taken from a counter inside `Game`. Copies of one
card get different ids, and a replay hands out the same ids.

## Considered options

- The definition's position in the table. Reordering the table breaks every stored reference.
- One enum variant per card. A card loaded from a file can't name a variant.
- The card's name. Renaming a card breaks every stored reference.
- Unversioned codes, with each game pinned to one snapshot of the card data. A card that names
  another card, such as one that summons a Recruit, would not need a new version each time the named
  card changes. Every replay would have to record its snapshot instead.
- A global id counter. A game cloned for search would shift the ids of every game after it.
