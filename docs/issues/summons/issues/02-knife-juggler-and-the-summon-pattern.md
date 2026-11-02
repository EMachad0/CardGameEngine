# Knife Juggler reacts to summons, and minions summoned together hear each other

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 01

## Why

No trigger pattern matches a summon. Knife Juggler needs one, and ADR 0022 decides who hears it:
minions summoned in one step hear each other's summons, and a list of summon effects summons one
after another.

## What

- `TriggerPattern::Summoned(TriggerSubject)`, matching a summon.
- `Event::BoardEntered` becomes `Event::Summoned`, so the pattern and its event share a name. Its
  history kind, `HistoryKind::CharacterEntered`, gets a name in the glossary's words (Summon).
- The summon step stays whole. Matching reads the state the step leaves, as ADR 0018 says.
- When the summon reactions of a played minion resolve doesn't change here. Ticket 03 moves them.

## Cards

- Knife Juggler, base set, a 2 mana 2/1 minion: "After a friendly minion is summoned, deal 1 damage
  to a random enemy." Pattern `Summoned(Character(..))` over friendly minions with
  `include_itself: false`. Effect: 1 damage to a `Random` enemy character.
- Mirror Image, base set, a 1 mana spell: "Summon two 0/2 minions." One effect whose selector is
  the owner and whose verbs are two `Summon`s of a new 0/2 Image definition.
- Juggling Act, a test spell: one effect that summons two Knife Jugglers.
- Juggling Line, a test spell: two effects, each summoning one Knife Juggler.

## Tests

With no enemy minions, a knife can only hit the enemy hero.

- A Knife Juggler damages the enemy hero when Barracks summons a Squire.
- A Knife Juggler doesn't hear its own summon.
- A Knife Juggler doesn't hear an enemy minion's summon.
- Across many seeds, a Knife Juggler's knife hits only enemy characters.
- With a Knife Juggler on the board, Mirror Image throws two knives.
- Juggling Act throws two knives: each Juggler hears the other's summon.
- Juggling Line throws one knife: the first Juggler hears the second, and the second doesn't hear
  the first.

## Done when

- The tests pass, and every existing test passes, changed at most in event and history names.
- The checks pass.
