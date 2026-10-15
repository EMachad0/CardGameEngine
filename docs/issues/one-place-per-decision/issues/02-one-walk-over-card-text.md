# One walk over a card's effect sequences

Status: resolved
Category: bug
Type: task
Blocked by: 01, 06, 07
Branch: `grammar-walk`, rebuilt from `main` once #10 and tickets 06 and 07 are merged. PR #14,
retargeted from `game-by-question` to `main`.

## Why

Card text nests. An effect can `Buff` or `AddFriendlyAura` with a `ModifierEffect`, which can
`GrantTrigger` a `TriggerDef`, whose effects nest again. Three validators walk this by hand, and
each one misses branches:

- a trigger granted through `CardDef::modifier_effects`
- a trigger carried by a `Buff` or `AddFriendlyAura` payload, at any depth.

A `Chosen` selector there passes validation and then panics at resolution with "Choice not
fulfilled", or reads another card's targets. An unresolved pick there passes too. No built-in
card hits this today.

The earlier #14 fixed it with one walk, `effect_sequences`, inside the validation file. The
learner's verdict was to keep the idea, with two changes:

- The walk moves next to `CardDef`, the type whose shape it knows, in a module of its own that
  hides its `collect_*` helpers.
- It destructures exhaustively. Today a new `CardDef` field that holds effects compiles and gets
  skipped, which is the original bug again, now in one place.

## What to build

- The walk goes in a new leaf module in `cards/`, a sibling of `definition.rs`, named for what
  it answers. `definition.rs` stays a leaf. The interface returns every effect sequence of a card
  at any depth, each labeled with its origin: the play effects or a trigger's effects. The old PR
  used `Sequence { origin, effects }` and `SequenceOrigin::{Play, Trigger}`. The `collect_*`
  helpers stay private to the new leaf.
- The walk uses no `..` in its patterns: `CardDef`, the payload of every `CardDefKind` variant,
  `TriggerDef`, every `Effect` variant and every `ModifierEffect` variant. Bind fields it ignores
  as `field: _`. Every `match` in it is exhaustive.
- The validators ask the walk:
  - Choice not found: play sequences only, as today.
  - Choice in a trigger: every trigger sequence, at any depth.
  - Pick order: every sequence.
- Pin the walk's own contract with unit tests in its module, one for each place a sequence can
  sit: play effects, a printed trigger, a grant in `modifier_effects`, a grant in each aura list,
  a grant inside a `Buff` payload, inside an `AddFriendlyAura` payload, and a grant nested inside
  a granted trigger.

## Tests first

This ticket changes behavior on purpose: validation rejects definitions it used to accept. Bring
over the old PR's validator tests (`git show 7b0bb2f:crates/rules/src/cards/validate.rs`):

- a choice in a trigger granted through `modifier_effects`, through a `Buff`, through
  `AddFriendlyAura`, and two levels deep
- an unresolved pick in a trigger granted by a `Buff`
- a `Buff` in play effects may still choose whom it buffs.

Commit them first and run them against main, which holds ticket 01's code. All but the last should fail with
`Ok(())`. Record that run in "Behavior". Then build the walk and make them pass. No built-in card
changes, so `tests/spec/` stays unchanged.

## Out of scope

- `game/lookup.rs`'s `triggers()` reads live triggers from runtime modifiers, not from card text.
  Leave it.
- Kind questions on `CardDef`: ticket 03. The walk can match on `CardDefKind` itself.

## Done when

- One function knows where effect sequences sit in a card, and every validator asks it.
- A new `CardDef` field, a new field on a kind payload or on `TriggerDef`, a new `Effect` or
  `ModifierEffect` variant, or a new field on an existing variant fails the build at the walk.
- The new tests fail on 01's code and pass here, and the checks pass.

## Answer

PR #14: https://github.com/EMachad0/CardGameEngine/pull/14
