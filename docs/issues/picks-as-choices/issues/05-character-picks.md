# Character picks

Status: ready-for-human
Category: enhancement
Type: task
Blocked by: 04

## Why

After ticket 04 only cards can be picked. A choice of a character made while the card resolves,
such as one that sees the damage an earlier effect dealt, still can't be written.

## Forcing card

"Deal 1 damage to all minions. Then choose a minion and give it +2 Health."

## What

From ADR 0019 and ADR 0020:

- `CharacterSelector::Picked { choice: Choice<CharacterFilter>, others: Vec<CharacterVerb> }`.
- `Picking::Characters`. Its options are the characters that match the filter when resolution
  reaches the effect. A character pick has no window, so every character it covers is an option.

## Tests

- A 1-health minion hit by the first effect is offered. Picked, it gets +2 Health and survives the
  state check at the card's end.
- A minion summoned by an earlier effect of the same card is offered, since options are read when
  resolution reaches the effect.
- With no minion in play, the pick is skipped and the card's other effects still run.
- The verbs in `others` act on every option not picked: "Choose a minion. Give the other minions +1
  Attack."
- A death rattle can pick a character.
- Every player other than the chooser sees how many options there are and no object ids.

## Done when

The checks pass, and the agent's review against `docs/conventions/file-structure.md` is answered.
