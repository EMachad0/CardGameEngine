---
status: accepted
---

# Loading also rejects a hero that names a card that is not a hero

Card data rules out invalid shapes through its types wherever a type can. Where no type can rule a
shape out, the shape gets a defined meaning and is allowed, however pointless. Loading rejects only
data the engine can give no meaning to:

- a code that names no card,
- two cards with one code,
- a deck whose hero names a card that is not a hero.

Loading covers building a binder and starting a game from decks. A deck names its hero by a code,
and no type can say that the code names a hero card. A minion or a spell in the hero zone has no
meaning: the zone holds exactly one object for the whole game (ADR 0008), and a dead minion leaves
its zone.

So a choice that no verb and no `others` use is still asked, and an empty filter matches nothing,
as ADR 0021 decided. Its considered options still hold.

## Considered options

- Giving a non-hero in the hero zone a meaning, such as a minion that never leaves it. That is rules
  work no card asks for.
- No check, so a non-hero hero breaks the zone's invariant later, far from the deck that caused it.
