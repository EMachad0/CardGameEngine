# An effect is a subject with one selector and a list of verbs

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 01

## Why

Each `Effect` variant in `effect.rs` carries its own selector. Two verbs on one random minion draw
twice, and one target can't feed two verbs, the reuse ADR 0017 gave up. ADR 0019: the selector is
read once, and every verb runs in order on the whole selection.

## What

- `Effect<S>` becomes `Players { selector: PlayerSelector, verbs: Vec<PlayerVerb> }` and
  `Characters { selector: CharacterSelector<S>, verbs: Vec<CharacterVerb> }`.
- `PlayerVerb { Draw(EffectAmount), Summon(DefId) }` and
  `CharacterVerb { Damage(EffectAmount), Modify(ModifierEffect), AddFriendlyAura(ModifierEffect) }`.
  `Buff` becomes `Modify`, as the glossary names it.
- `Reveal` and `ResolvePick` stay as variants of their own. Ticket 04 removes them.
- Every card in `cards/binder/sets/base.rs` becomes one subject with one verb.
- The interpreter in `game/resolve/effect.rs` resolves the selector once and runs each verb on the
  selection. The effect stays one step with one checkpoint (ADR 0018).

## Leave alone

Every existing card resolves as before: the same events, in the same order, with the same random
draws.

## Tests

- Two verbs on one `Random` selection act on the same character.
- Two verbs on one `Targeted` selection act on the targeted character: "Choose a minion. Deal 2
  damage to it, then give it +2 Attack."
- Verbs run in list order, as the order of their events shows.
- A verb after one that brings a minion to 0 health still acts on it, since it dies only at the
  state check.
- An effect with several verbs gives one checkpoint.
- An effect with no verbs changes nothing.

## Done when

- No effect variant carries a selector of its own, except `Reveal` and `ResolvePick`.
- The checks pass, and existing tests changed only in how they build card data.
