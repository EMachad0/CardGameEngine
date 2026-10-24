---
status: accepted
---

# Card data is checked by its types, and loading rejects only data with no meaning

Card data rules out invalid shapes through its types wherever a type can, as the subjects of ADR
0019 keep a stat buff off a spell card. Where no type can rule a shape out, the shape gets a defined
meaning and is allowed, however pointless. Loading rejects only data the engine can give no meaning
to: a code that names no card, or two cards with one code.

So a choice that no verb and no `others` use is asked like any other. A target with no verbs still
decides whether the card can be played and shows in every player's view of the targeting, and a pick
with no verbs still opens. An empty filter matches nothing.

## Considered options

- A load check that rejects a choice nothing reads. Each new kind of reader widens what it has to
  look at: a named result, an amount that reads a target, a target remembered by a trigger. With
  named results, one way to write "Draw cards equal to the power of target creature you control" is
  an effect that targets with no verbs, and Forge holds that target on a pump effect that pumps
  nothing.
- Skipping a choice nothing reads. The card plays as if it had no choice: offered with no
  candidate, nothing shown, while its data says otherwise. A forgotten verb then shows in no game,
  only in a test of what the verb should do.
- A verb list that can't be empty. A pick whose `others` act needs none ("Choose a minion. Give the
  other minions +1 Attack."), and neither does a selection that a later effect reads.
- Checks only while card files are parsed, none on parsed data. Cards written in Rust have no
  parse, so nothing would catch a code that names no card or a code used twice.

## Consequences

- A mistake with a meaning, such as a forgotten verb, shows in play or in a test, never at load.
- The check that pairs each `Reveal` with its `ResolvePick` falls outside this line, and ADR 0020
  removes it.
