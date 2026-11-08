---
status: accepted
---

# The rules crate holds no cards, and a game is built from a binder of prefixed sets

The rules crate defines what card data can say and holds no card definitions. A game is started
with a binder, an input like its seed and its decks, so a replay records it too. Which sets a game
loads, and which of them it calls its base, is the game's call. The rules crate's own tests share a
test set of their own.

A binder is built from sets. Each set holds a prefix, and each of its cards holds only a name and a
version. The binder composes the code, `prefix.name.version`, so no set can give a card a code under
another set's prefix. A code still carries its version, as in ADR 0003.

## Considered options

- The base set inside the rules crate, with test cards added on top. Cards that exist only to pin a
  rule, such as one whose effect has no verbs, would ship in every game's binder.
- The cards in a separate crate that the rules crate takes as a dev-dependency. Its crate tests
  compile, but its unit tests link a second copy of the rules crate, and their types don't match.
- Full codes on each card, with a load check that each code starts with its set's prefix. It adds a
  rejection to the two ADR 0021 allows, for data a type can rule out.
- A binder built from any list of cards. Nothing would stop two binders from giving one code two
  definitions, and a code would no longer pin a definition across games.

## Consequences

- A unit test inside the rules crate builds the cards it needs, and the bench builds its own set,
  since neither can reach the crate tests' set.
- Setup can't name a hero definition, so the hero comes from outside the rules crate.
