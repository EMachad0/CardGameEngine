---
status: accepted
---

# Minions summoned in one step hear each other's summons

A summon is matched against the state its step leaves (ADR 0018), so every minion one step summons
is on the board when the first summon is matched, and each hears the others. MTG checks the
newcomers too (603.6a), and in Yu-Gi-Oh monsters Special Summoned at the same time see each other.
Each effect is one step, so card data chooses: the summons of one effect happen together, and in a
list of summon effects a minion hears only the summons after its own.

A filter matches the listener's own object whenever the listener passes it, for a summon as for
damage. Card data says "another" with `include_itself: false` on a character filter, which leaves
out the object whose text it is, in a trigger's pattern and in a selector alike. No rule removes
the listener from its own summon.

## Considered options

- One step per summon, as Hearthstone gives each summon its own sequence: an earlier minion hears a
  later one, never the reverse. Card data could not say "together", and the step would stop
  partway through an effect.
- A rule that a filter never matches the listener's own summon, so Knife Juggler's "a friendly
  minion" is written as printed. A filter would include the listener for damage (Armorsmith) and
  leave it out for a summon, and "Whenever a creature enters", which hears itself, would need two
  triggers.
- "Another" as a third trigger subject beside `Itself` and a filter. Selectors such as "your other
  minions" (Stormwind Champion) would need a second way to say it.
- "Another" as a set axis on the filter, like kind and side. Its "itself only" value would say what
  `Itself` already says.
