---
status: accepted
---

# Loading also rejects a summon that names a card that is not a minion

Card data rules out invalid shapes through its types wherever a type can. Where no type can rule a
shape out, the shape gets a defined meaning and is allowed, however pointless. Loading rejects only
data the engine can give no meaning to:

- a code that names no card,
- two cards with one code,
- a deck whose hero names a card that is not a hero,
- a summon that names a card that is not a minion.

Loading covers building a binder and starting a game from decks. A summon names its card by a code,
and no type can say that the code names a minion card. A spell or a hero on a board has no meaning:
a spell has no attack or health, and a hero belongs in its hero zone, which holds exactly one object
for the whole game (ADR 0008).

So a choice that no verb and no `others` use is still asked, and an empty filter matches nothing,
as ADR 0021 decided. Its considered options still hold.

## Considered options

- Giving the summon of a non-minion a meaning, such as summoning nothing. A card would then carry
  text that never does anything, and nothing would tell its author.
- No check, so the game puts a spell or a hero on the board and fails later, far from the card that
  caused it.
