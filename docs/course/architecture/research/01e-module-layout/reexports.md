# Re-exports and one path per definition

Researcher report, session 01e, kept in full. Local checks and counts by the session agent follow
it.

## Summary

No current tool enforces "never two ways to reach the same definition" inside a crate. Clippy has
no lint that flags importing an item through a re-export when the defining path is also visible,
or the reverse. Two Clippy feature requests for this are open, and neither has a linked PR. The
nearest primary-source endorsement of the rule is the rust-analyzer style guide: "By default, avoid
re-exports. Rationale: for non-library code, re-exports introduce two ways to use something and
allow for inconsistency."

The privacy rules make a `mod.rs` facade a hard boundary only from outside the directory. Inside
`foo/`, a sibling can still write `super::bar::Thing` next to the facade path, so two paths stay
reachable there and only convention or review can stop it. Since 1.75, rustc's `unused_imports`
decides by effective visibility: it reports an unused `pub(crate) use`, an unused
`pub(super) use`, and an unused `pub use` that is not exported from the crate. rust-analyzer's
auto-import picks the shortest path, which is usually the re-export.

## 1. Clippy lints

a. `pub_use`, verified.

- Group `restriction` (allow by default), since 1.62.0.
- What it does: "Restricts the usage of `pub use ...`"
- Rationale: "A project may wish to limit `pub use` instances to prevent unintentional exports, or
  to encourage placing exported items directly in public modules." The help text says "move the
  exported item to a public module instead".
- Scope: only `pub use`. The check is `VisibilityKind::Public`, so `pub(crate) use` and
  `pub(super) use` are never flagged. A `pub use` inside a private module is flagged, because the
  check is on syntax only.
- Source: https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/pub_use.rs

b. `wildcard_imports`, corrected (the docs understate the exemption).

- Group `pedantic` (allow by default), since 1.43.0.
- What it does: "Checks for wildcard imports `use _::*`."
- Rationale: "wildcard imports can pollute the namespace. This is especially bad if you try to
  import something through a wildcard, that already has been imported by name from a different
  source"
- Documented exceptions: modules whose name contains `prelude`; "Wildcard imports reexported
  through `pub use` are also allowed."; "`use super::*` is allowed in test modules."
- Correction: the code exempts every glob that is not private, not only `pub use`:
  `if cx.tcx.local_visibility(item.owner_id.def_id) != ty::Visibility::Restricted(module) && !self.warn_on_all { return; }`.
  So `pub(crate) use foo::*` and `pub(super) use foo::*` are exempt as well.
- `allowed-wildcard-imports`: "List of path segments allowed to have wildcard imports". It "has no
  effects if used with `warn_on_all_wildcard_imports = true`."
- `warn-on-all-wildcard-imports`: "Whether to emit warnings on all wildcard imports, including
  those from `prelude`, from `super` in tests, or for `pub use` reexports." Default `false`. Lint
  on `pub use` was added in PR #14182.
- Sources: https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/wildcard_imports.rs,
  https://github.com/rust-lang/rust-clippy/blob/master/book/src/lint_configuration.md

c. `redundant_pub_crate`, verified.

- Group `nursery` (allow by default), since 1.44.0.
- What it does: "Checks for items declared `pub(crate)` that are not crate visible because they
  are inside a private module."
- Rationale: "Writing `pub(crate)` is misleading when it's redundant due to the parent module's
  visibility." It suggests `pub` instead.
- It covers `pub(crate) use` too, except macro re-exports. It doesn't cover `pub(super)`. "Private
  module" means the parent isn't exported from the crate, which includes a `pub(crate) mod`. So
  `pub(crate) use bar::Thing;` in a non-exported `foo/mod.rs` gets flagged.
- Conflict with `unreachable_pub`: issue #5369 is open. Clippy suggests `pub`, then rustc's
  `unreachable_pub` suggests `pub(crate)`, in a loop, re-exports included. flip1995: "We should
  move this lint to nursery until this is fixed."
- Source: https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/redundant_pub_crate.rs

d. A lint for "same item reached by two paths": verified that none exists in Clippy.

- rust-clippy#12878 "Lint against inconsistent import paths" (open, 2024-06-03) asks to "warn if
  the import paths to the same object in the project is inconsistent", suggesting "use the
  shortest absolute path" or "use the path which has the least restrictive visibility". No linked
  PR.
- Users forum, 2024-06-03 (https://users.rust-lang.org/t/how-to-make-the-import-paths-consistent/112382),
  jhpratt: "As far as I know, not at this point in time."
- rust-clippy#16392 "disallow reexporting non-owned imports" (open, 2026-01-13): "every entity not
  owned by the current module has an unique 'canonical' import path." A commenter points to a
  third-party tool, `redundant_reexport`, "which warns about `use`ing items from a more visible
  module". Its repo returns 404, unverified.
- Dylint's `inconsistent_qualification` "Checks that a module's items are either imported or
  qualified with the module's path, but not both", and lists "Re-exports may not be handled
  correctly." among its known problems. It doesn't do this.
- rustc's `redundant_imports` (allow by default) "detects imports that are redundant due to being
  imported already; either through a previous import, or being present in the prelude".
  `unqualified_local_imports` (unstable) checks that local imports start with `self::`, `super::`
  or `crate::`. Neither solves this.

e. Path-choice lints, verified.

- `absolute_paths` (`restriction`, 1.73.0): "Many codebases have their own style when it comes to
  importing, but one that is seldom used is using absolute paths everywhere." It ignores `use`
  items.
- `std_instead_of_core`, `std_instead_of_alloc` and `alloc_instead_of_core` (`restriction`,
  1.64.0) choose between two paths to one item, for `no_std` portability.
- Clippy's stance: path choice is project style, handled by opt-in restriction lints.

## 2. rustc lints

`unused_imports`, verified from source.

- `check_unused.rs`: an import is exempt only if `self.r.effective_visibilities.is_exported(def_id)`.
  So an unused `pub(crate) use` in a lib warns, an unused `pub(super) use` warns, and an unused
  `pub use` inside a private module that nothing re-exports outward warns.
- Changed in rust-lang/rust PR #116033, "report `unused_import` for empty reexports even it is
  pub", merged 2023-10-23, milestone 1.75.0. Before, any syntactically `pub` import was skipped.
- The lint doc text predates the change: "If you intended to re-export the item to make it
  available outside of the module, add a visibility modifier like `pub`."
- Caveat: glob `pub(crate) use …::*` had false positives, e.g. rust#152004 (2026 nightly
  regression). Status not checked.

`ambiguous_glob_reexports` (warn) "detects cases where names re-exported via globs collide."
`hidden_glob_reexports` (warn) "detects cases where glob re-export items are shadowed by private
items." `unreachable_pub` (allow) fires on "`pub` items not reachable from other crates - that
means neither directly accessible, nor reexported (with `pub use`)…", `pub use` items included.

## 3. Privacy mechanics, verified

Rust Reference, Visibility and privacy:

1. "If an item is public, then it can be accessed externally from some module `m` if you can
   access all the item's ancestor modules from `m`. You can also potentially be able to name the
   item through re-exports."
2. "If an item is private, it may be accessed by the current module and its descendants."

- A private `mod bar;` in `foo` is accessible to `foo` and every descendant, so a sibling can
  write `super::bar::Thing` next to the facade path.
- Outside `foo`, rule 1 fails at `bar`. The Reference's own example: "any external crate
  referencing `implementation::api::f` would receive a privacy violation, while the path `api::f`
  would be allowed", the "privacy chain" being "short-circuited through the reexport". E0603: "A
  private item was used outside its scope."
- From memory, unverified: re-exporting a `pub(super)` item with `pub(crate) use` should fail with
  E0364.

## 4. Guidance for and against re-exports

a. rust-analyzer style guide (https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/style.md),
verified:

- "Prefer `use crate::foo::bar` to `use super::bar` or `use self::bar::baz`. Rationale:
  consistency, this is the style which works in all cases."
- "By default, avoid re-exports. Rationale: for non-library code, re-exports introduce two ways to
  use something and allow for inconsistency."
- "Re-exports are treated as item definitions rather than imports, so they go after imports and
  modules. Use them sparingly."

b. Rust API Guidelines: no item on re-exports or module facades. Closest is C-HIDDEN: "Rustdoc is
supposed to include everything users need to use the crate fully and nothing more."

c. rustdoc book, Re-exports (https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html):
"But what if you want the types to be available directly at the crate root or if we don't want the
modules to be visible for users? That's where re-exports come in", and "since both `sub_module1`
and `sub_module2` are private, users won't be able to import them." It notes `#[doc(inline)]` shows
a type "both at the crate root and in the `public_mod` module", with no normative statement about
several paths.

d. Standard library: no official statement on why `HashMap` has two paths. In rust#72642
(2020-05-27) dtolnay says the `std::collections::hash_map::HashMap` suggestion "is just noise,
it's (almost?) never what we actually want the person to import." Fixed by PR #73023.

e. Practitioners:

- Aaron Turon, "Revisiting Rust's modules" (2017-07-26): "This kind of setup is known generally as
  the facade pattern, and it's pretty ubiquitous in Rust code." "`pub use` abuse… the 'apparent'
  module hierarchy… often tells you very little about the actual module hierarchy." "it was
  impossible to predict anything about the public API surface just by looking at the file system
  organization; you have to trace re-exports."
- RFC 2126: "in general you have to walk up the module tree looking for re-exports to know exactly
  how public an item is."
- matklad, "Notes On Module System" (2021-11-27): "the fact that `use` is an item (ie, `a::b` can
  `use` items imported in `a`) is an unnecessary cuteness. Imports should only introduce the name
  into module's namespace, and should be separate from intentional re-exports." "It might make
  sense to ban glob re-export."
- users.rust-lang.org "Avoid `pub use`" (2024-05-29), community posters: farnz, "Avoid having
  multiple paths to the same item in your exported API… each item should have one exported path
  only."; mankinskin, "within the same crate it can easily get overused… It can even break auto
  import features in IDEs."

## 5. rust-analyzer auto-import, verified from docs and source

- `imports.prefix`, default `"crate"`: "force paths to be always absolute, starting with the
  `crate` prefix, unless the item is defined outside of the current crate". `imports.preferPrelude`
  and `imports.preferNoStd` default to `false`. `imports.granularity.group` only merges imports.
- `find_path.rs`: an item already in scope keeps its name; else a prelude path; else
  `Choice::try_select` ranks by stability, then prelude preference, then fewest segments. The test
  `shortest_path` expects `baz::S` over `foo::bar::S` when `baz` re-exports it.
- So auto-import picks the re-export wherever it is shorter, and nothing stops a hand-written
  longer path.

## Could not verify

The exact E0603 message (no compiler run), E0364 for widening a `pub(super)` item, whether the 1.75
`unused_imports` change is in the release notes, the `redundant_reexport` tool, an official std
statement on `HashMap`'s two paths, rust-analyzer's tie-break between equal segment counts, and the
status of rust#152004.

## Local checks and counts (session agent)

On Clippy 0.1.96, in a scratch crate with `mod bar;` and `pub(crate) use bar::Thing;` in
`foo/mod.rs`:

- A sibling in `foo/` compiles with `super::bar::Thing`, `super::Thing` and `crate::foo::Thing`,
  with no warning.
- Outside `foo`, `foo::bar::Thing` fails with E0603, "module `bar` is private".
- An unused `pub(crate) use` warns as an unused import.
- `clippy::pub_use` doesn't fire on `pub(crate) use`. `clippy::redundant_pub_crate` fires on
  `pub(crate)` items and re-exports inside the private module.

On the `loader-split` branch (#10), counting `use crate::…` imports:

- 63 imports in 23 files go through the crate root's public re-exports, covering 11 items:
  `DefId`, `Event`, `Game`, `IllegalAction`, `ObjectId`, `Observer`, `Outcome`, `PickId`,
  `PlayerId`, `View`, `Views`.
- 14 items leave `cards/` by their module path, and 2 leave `game/`.
