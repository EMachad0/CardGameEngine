# One path to each definition

Status: resolved
Category: enhancement
Type: task
Blocked by: 01, 06
Branch: `one-path`, new, from `main` once #10 and ticket 06 are merged. New draft PR with base
`main`.

## Why

`docs/conventions/file-structure.md`, Rust, "Paths": from any place in the code, one path reaches
each definition. Today most items have two or more:

- `lib.rs` re-exports the public API at the crate root, and crate code imports through it. On the
  `loader-split` branch, 63 imports in 23 files do so, as in `use crate::{Event, ObjectId};`. The
  items are `DefId`, `Event`, `Game`, `IllegalAction`, `ObjectId`, `Observer`, `Outcome`,
  `PickId`, `PlayerId`, `View` and `Views`. The same items are also reachable by their defining
  module.
- `mod.rs` files re-export inside the crate: `cards/binder/mod.rs`, `cards/binder/sets/mod.rs`,
  and `pub use resolve::ApplyError` in `game/mod.rs`.

The doc's rules: import from the defining module, spelled from `crate::`; no re-exports inside the
crate; the public API is one leaf, `prelude.rs`; nothing inside the crate imports through it; other
crates import from it by name, never with a glob; a text search in the checks enforces it.

## What to build

1. `crates/rules/src/prelude.rs`, a leaf holding the crate's whole `pub use` list: every item
   `lib.rs` exports today, each from its defining module. Its docstring says it is the crate's
   public API, imported by name.
2. `lib.rs` holds only `mod` declarations, `pub mod prelude;` and its docstring. Rewrite the
   docstring to match: the `Layout:` list goes stale, so drop it.
3. Remove every other re-export in `crates/rules/src`. Each `mod.rs` holds only `mod`
   declarations, each at the narrowest visibility that lets its users reach the defining module.
4. Rewrite every import in `crates/rules/src` to the defining module's path, from `crate::`. The
   one exception is `use super::*;` in test modules. Today `game/resolve.rs` has `use super::Game;`
   and `base.rs` has `use self::static_card_definition::*;`.
5. Rewrite the imports in `crates/rules/tests/` and `crates/rules/benches/` to name items from
   `rules::prelude`, as in `use rules::prelude::{Game, ObjectId};`. No glob import of the prelude.
6. Add a recipe to `.just/check.just` and include it in `check::all`. It fails when:
   - a file in `crates/rules/src` other than `prelude.rs` contains the word `prelude`, apart from
     the `pub mod prelude;` line in `lib.rs`
   - any `.rs` file in the workspace glob-imports `rules::prelude`
   - a file in `crates/*/src` contains `super::` or `self::`, apart from `use super::*;`.

   Match whole words, so `use crate::{prelude::Game}` and `super::prelude` are caught. Use only
   tools the CI image already has: `grep` runs everywhere, `rg` may not. Update the description of
   `check::all` in `AGENTS.md` to name the new step.
7. Keep `unreachable_pub` as it is. It is what makes the prelude the whole public API.

## Out of scope

- Renaming `Event` or `Observer`, which collide with names in Bevy's prelude. That is a domain
  naming call for later.
- Moving definitions between leaves, or renaming modules.

## Done when

- `prelude.rs` is the only file in `crates/rules/src` with `pub use`, and no file there has a
  restricted `use` re-export such as `pub(crate) use`.
- Every import inside the crate starts with `crate::` and names the defining module.
- The new recipe runs in `check::all`, fails on each of the three patterns when you plant one,
  and passes on the branch. Say in the PR body how you checked each pattern.
- The checks pass, with `tests/spec/` changed only in its imports.

## Answer

PR #27: https://github.com/EMachad0/CardGameEngine/pull/27
