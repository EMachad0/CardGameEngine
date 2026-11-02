# A played minion's summon reactions resolve after its play

Status: resolved
Category: enhancement
Type: task
Blocked by: 02

## Why

Playing a minion pushes its play effects, then the reactions to the play, then, in the summon
step, the reactions to its summon. The newest resolves first, so a Knife Juggler's knife resolves
before the minion's own play effects. ADR 0023 keeps the summon matched in its own step and pushes
its matches under everything the play pushed.

## What

- A played minion's summon matches go under its play effects and the reactions to its play.
- A summon by an effect is unchanged: its matches go on top.
- `Summoned(Itself)` gets its first card, so a minion's own summon reaction is pinned.

## Test cards

- Courier, a minion: "Play: deal 2 damage to the enemy hero. When this is summoned, deal 3 damage
  to the enemy hero." The second sentence is a `Summoned(Itself)` trigger.
- Herald, a minion: "Whenever you play a minion, draw a card."
- Ringmaster, a minion: "Play: summon a Knife Juggler."
- Dispatch, a spell: "Summon a Courier."
- Tremor, from ticket 01.

The amounts may change if the events need telling apart.

## Tests

- With a Knife Juggler on the board, Courier's play effect resolves before the knife.
- Courier's play effect resolves before its own summon reaction.
- With a Knife Juggler on the board, Courier's own summon reaction resolves before the knife.
- With Herald on the board, Herald's draw resolves before Courier's play effect.
- Dispatch runs Courier's summon reaction and not its play effect.
- Ringmaster played on an empty board throws no knife: the Juggler it summons was not there when
  Ringmaster's summon was matched.
- A Knife Juggler with 1 Health that Tremor's play effect kills still throws its knife.

## Done when

- The tests pass, and every existing test passes unchanged.
- The checks pass.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/89
