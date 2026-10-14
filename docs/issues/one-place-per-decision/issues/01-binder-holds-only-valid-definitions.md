# A binder holds only valid card definitions

Status: resolved
Category: enhancement
Type: task
Branch: `loader-split`, rebuilt from `main`. PR #10.

## Why

`cards/loader.rs` holds three things that change for different reasons: the card codes
(`def_ids!`, which generates `static_card_definition` and `ALL_DEF_ID`), the base card table
(`CardDefLoader::load_all`), and the validation rules with `CardDefError`. And `Binder::new`
accepts any `Vec<CardDef>`. A binder holds valid definitions only because `Game::setup` calls
`load_and_validate` first, which is informal knowledge in the caller.

The placeholder rule is split too. `definition.rs` builds the `Default` with `PLACEHOLDER`, the
loader rejects it, and the two files import each other.

The learner's verdict on the earlier #10, a move-only split into `def_ids.rs`, `loader.rs` and
`validate.rs`: it split by step. The codes and the table are only used together and always
change together, so they belong in one module. Loading and validation should be hidden, and only
loading and the codes exposed.

## What to build

- `cards/binder.rs` becomes the module `cards/binder/`. Its interface:
  - one constructor that builds the base set's definitions, validates them and returns
    `Result<Binder, CardDefError>`. `Binder::built_in` is a candidate name.
  - `get`, and the other accessors as they are. Ticket 03 replaces them.
  - `Binder::new` becomes private to the module, so holding a `Binder` proves validation ran.
- A child module holds the base set: the `def_ids!` invocation and the table, in one file. The
  codes read `base.<card>.v0`, so `base.rs` is the proposed name. `PLACEHOLDER` and
  `impl Default for CardDef` move into it, because only the table's literals use them. Then
  `definition.rs` no longer refers to the placeholder, and the import cycle is gone.
- A private child holds validation: `CardDefError`, `validate`, every validator, and their 13
  unit tests, unchanged apart from paths. Name it for what it decides. If you keep a step name,
  say in "Names" why it passes.
- `CardDefLoader` goes away, and `Game::setup` calls the new constructor.
- The `TODO` about loading from disk describes a future edit. Drop it.
- Public API: remove `CardDefLoader` and `CardDefError` from `lib.rs`. Confirm first that no
  test or bench names them. `CardDef`, `DefId` and `rules::static_card_definition::*` keep their
  public paths. Making `CardDef` private would force most card-text types to `pub(crate)` under
  `unreachable_pub`, and that decision waits.

## Out of scope

- The validators' hand-written walks over card text. Ticket 02 replaces them, so move them as
  they are.
- Kind questions and the accessors on `Binder`: ticket 03.
- The name `static_card_definition`.
- Whether `Game::new` should take a binder from its caller.

## Done when

- Only the binder module can build a `Binder`, and its one constructor validates.
- The codes and the table sit in one file, and validation is private to the binder module.
- `lib.rs` exports neither `CardDefLoader` nor `CardDefError`.
- The checks pass, with `tests/spec/` unchanged.

## Intentions to cover

Besides the spec's format, say what each new module hides and what crosses its boundary, and
which changes now fail the build when someone tries to store unvalidated definitions.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/10
