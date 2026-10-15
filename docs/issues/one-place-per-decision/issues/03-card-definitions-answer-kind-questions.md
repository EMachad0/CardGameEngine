# Card definitions answer their kind questions

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 02
Branch: `card-def-kinds`, rebuilt from `grammar-walk` after ticket 02. PR #15.

## Why

`Binder` has 17 accessors over one map. Most return a field, or match on the kind to answer a
question about one definition.

`is_spell`, `is_minion` and `is_hero` are filters built with `matches!`. On main, these places
combine them, and each one decides something for every kind:

- the kind filters in `game/condition.rs`, which chain `||` over the three
- the in-play test in `Game::health`, in `game/lookup.rs`
- who receives auras: `Game::modifiers` in `game/lookup.rs`, under `if is_minion`
- whether a dead object leaves its zone: `Game::kill` in `game/zone_move.rs`, under
  `if !is_hero`
- history counting: `Game::history_query` in `game/resolve.rs`, which counts spells played and
  minions died.

A new `CardDefKind` variant compiles at every one of them and gets an answer nobody chose. A new
kind on the board would panic in `find_dead`. `CardKindFilter` and `CharacterKindFilter` copy the
kinds by hand. `destroy` asks `has_deck_presence` when it means "goes to the graveyard".

The doc's rule: "When several filters together decide what each variant means, give that
decision one exhaustive match."

## What to build

### Port the earlier #15

The earlier #15 (`17c73ac`, `11bec33`, `8965cc5`) was built on a `game/` layout that no longer
exists. Port it by intent, not by cherry-pick:

- `CardDef` answers `health`, `attack`, `has_board_presence`, the two aura lists and
  `choice(id)`. `Binder` keeps ticket 01's constructor and `get`, and nothing else.
- `game/` reaches a definition through one helper, `Game::card_def(object_id)`, next to
  `Game::def_id` in `game/lookup.rs` and no wider than `pub(super)`.
- `Trigger` holds its whole `TriggerDef` instead of copying the pattern and effects.
- Renames:
  - `Trigger.source` and `Modifier.source` become `granted_by`. For a granted trigger the
    grantor and the holder are different objects, and one name for both made `enqueue_triggers`
    read as if they matched.
  - `PendingEffect.object_id` and `ResolvingEffect.object_id` become `source`, the acting object,
    as in `Event::Damaged`.
- Locals named `def` that hold a trigger list or a number get names for what they hold.

### Kind questions

Remove `is_spell`, `is_minion` and `is_hero`. Each place in the list above asks one named
question instead, and each question is one exhaustive `match` on `CardDefKind`, with no `_` arm:

- Which `CardKindFilter` flag a definition has, and which `CharacterKindFilter` flag, if any. The
  kind filters and `history_query` ask these, as in `kind.contains(card_def.card_kind())`.
- Whether it receives auras.
- Whether it leaves its zone when it dies.
- Whether it goes to the graveyard. This replaces `has_deck_presence`, for the question `destroy`
  actually asks.
- The in-play test in `Game::health` asks the character kind flag. Ticket 04 replaces this test,
  so keep it simple.

Name each question for what its caller asks, with words from the glossary. Keep
`has_board_presence` as the question `play` asks, apart from "receives auras", even where the two
answers match today.

`Binder::get` stays, and keeps handing out `&CardDef` with public fields. Every interpreter reads
card text as data (ADR 0004), so `CardDef`'s layout is interface by design. Accessors would only
mirror the fields, and nothing can write through a shared reference.

## Out of scope

- Characters in play: ticket 04.
- Trigger matching: ticket 05.
- The `_ =>` filters over `ModifierEffect` in `game/lookup.rs`, and moving `Modifiers` or
  `Trigger` into modules of their own.

## Done when

- `Binder` is a constructor and `get`.
- Outside tests, no code matches on `CardDefKind` or combines kind checks, except `CardDef`'s own
  methods, ticket 02's walk and the in-play test that ticket 04 replaces.
- Adding a `CardDefKind` variant fails the build at every kind question and at the walk.
- The checks pass, with `tests/spec/` unchanged.
