# A character filter can leave out the object whose text it is

Status: ready-for-agent
Category: enhancement
Type: task

## Why

A character filter matches the object whose text it is whenever that object passes it: the
listener in a trigger's pattern, the source in a selector. Card text says "another" or "other" to
leave it out (Knife Juggler, Stormwind Champion, "deal 1 damage to all other minions"). Since a
minion is summoned before its play effects run, a play effect over "all other minions" would reach
the minion itself. ADR 0022 says "another" is card data on the filter.

## What

- `CharacterFilter` gets `include_itself: bool`. `CharacterFilter::all()` and every existing
  literal set it to `true`, so no behavior changes.
- One function answers whether a character passes a filter, and it takes the object whose text the
  filter is in. Every reader of a character filter asks it: trigger patterns, the `All`, `Random`
  and `Targeted` selectors, picks, and targeting.

## Forcing card

Tremor, a test minion: "Play: deal 1 damage to all other minions." Its play effect is `All` over
minions of every side, with `include_itself: false`.

## Tests

- Tremor's play effect leaves Tremor undamaged.
- Tremor's play effect damages every other minion, friendly and enemy.

## Leave alone

The friendly aura's own "other" in `modifiers()`. Auras have no filter.

## Done when

- The tests pass, and every existing test passes unchanged.
- The checks pass.
