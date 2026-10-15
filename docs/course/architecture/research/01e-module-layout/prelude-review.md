# Critical review: a prelude-only public API with no internal re-exports

Researcher report, session 01e, kept in full.

## Summary

The proposal breaks a weak convention: it is unusual, but nothing in it is harmful. Across the
ecosystem, a `prelude` is a convenience subset meant for glob import, and every item in it also
exists at a real path. I found no crate whose whole API is reachable only through its prelude. The
concrete costs are elsewhere:

- Glob collisions in the Bevy front end. Bevy 0.20's prelude exports `Event` and `Observer`.
- The name says "glob me". Clippy's `wildcard_imports` exempts any module named `prelude`.
- The CI grep has gaps (detailed in Q6).

The proposal's real technical benefit is also verified: since April 2026, rust-analyzer demotes any
path segment named exactly `prelude` when `imports.preferPrelude` is false (the default). That makes
the IDE agree with the "import from the defining module" rule, which (A) and (B) cannot do.

Recommendation: keep the proposal and the name `prelude`, with a few changes. Treat the prelude as
an explicit-import API in the front end, harden the grep, add the `unreachable_pub` lint, and decide
whether to rename `Event` and `Observer`. Choose (D) only if you want the in-crate rule enforced by
the compiler.

## Findings

### Q1. What a prelude is for, and how real crates use it

1. std describes preludes as a convenience layer over the real paths (verified).
   https://doc.rust-lang.org/std/prelude/index.html:
   - "Preludes can be seen as a pattern to make using multiple types more convenient."
   - Other preludes "are not automatically `use`'d, and must be imported manually. This is still
     easier than importing all of their constituent components."

   That second sentence assumes the components can be imported somewhere else. The Rust API
   Guidelines checklist has no entry on preludes or re-exports; I searched for "prelude" and
   "re-export" and found neither (verified, https://rust-lang.github.io/api-guidelines/checklist.html).
   The rustdoc book has no prelude guidance either.

2. Style guides disagree, but none describes a prelude-only API (verified).
   - PingCAP (https://pingcap.github.io/style-guide/rust/modules.html): "If a crate … has a large
     API, but has a relatively small subset which is commonly used, then consider adding a prelude
     module", "It should not include items with common names (to avoid name clashes with other
     modules)", and "In small crates, it is often best to re-export the entire API and make all
     submodules private."
   - Microsoft Pragmatic Rust, M-NO-PRELUDE (https://microsoft.github.io/rust-guidelines/guidelines/libs/ux/):
     "Crates must not define a `prelude` … once multiple preludes are used from different crates
     there is potential for conflicts", with an E0659 example.
   - epage's Rust style (https://epage.github.io/dev/rust-style/), P-PRELUDE-MOD: "they are
     intended as an import convenience." Note that epage's P-DIR-MOD says `mod.rs` "should only
     contain re-exports", which is the opposite of your "mod declarations only" rule.
   - corrode (https://corrode.dev/blog/dont-use-preludes-and-globs/): "why not re-export common
     types in root?" and "Tokio removed their prelude."
   - Effective Rust, Item 23 (https://effective-rust.com/wildcard.html): preludes are an accepted
     exception to "avoid wildcard imports", because they are "carefully curated."

3. Survey of real crates (verified on docs.rs unless marked). In every crate that has a prelude,
   the prelude is a subset of items that also live at their real paths:
   - std::io::prelude: `Read`, `Write`, `BufRead`, `Seek` link to `std::io::Read` and so on.
   - rayon::prelude: `pub use crate::iter::ParallelIterator`, and similar traits from
     `rayon::slice` and `rayon::str`.
   - chrono::prelude: "A convenience module appropriate for glob imports"; it contains
     `pub use crate::DateTime` and other root items.
   - diesel::prelude: "Re-exports important traits and types. Meant to be glob imported."
   - bevy::prelude (0.20.0): `Observer` links to `bevy::ecs::observer::Observer`.
   - tokio: has no prelude; `docs.rs/tokio/latest/tokio/prelude` returns 404.
   - nalgebra and itertools: the word "prelude" does not appear on their root docs pages.
     Itertools tells users to write `use itertools::Itertools;`.
   - A crate whose API exists only behind a prelude or an `api` module: I found none. This is a
     negative result, not proof that none exists.

### Q2. Glob collisions with Bevy 0.20

4. Which of your names collide (verified against the docs.rs bevy 0.20.0 prelude HTML; crates.io
   lists 0.20.0 as the newest version).
   - Collide: `Event` and `Observer`. Bevy's prelude has the trait `bevy::prelude::Event` and also
     a derive macro `Event`, and the struct `bevy::prelude::Observer`.
   - No collision: `Action`, `View`, `Outcome`, `Rng`, `Face`, `Game`, `Views`, `PlayerId`,
     `ObjectId`, `DefId`, `CardDef`, `ApplyError`, `HandCard`, `DraftView`, `PlayerView`,
     `IllegalAction`.
   - Caveat on `Rng`: it is free in Bevy, but it would collide with `rand::prelude::*` if the
     front end ever imports that (not checked against rand).

5. The Reference confirms the error happens only when the ambiguous name is used, and that
   explicit imports win (verified, https://doc.rust-lang.org/reference/names/name-resolution.html):
   - "Glob imports are allowed to import conflicting names in the same namespace as long as the
     name is not used. Names with conflicting candidates from ambiguous glob imports may still be
     shadowed by non-glob imports and used without producing an error. The errors occur at time of
     use, not time of import."
   - Use declarations (https://doc.rust-lang.org/reference/items/use-declarations.html): "Items and
     named imports are allowed to shadow names from glob imports in the same namespace."
   - The error code is E0659, "An item usage is ambiguous".

6. How painful it is in practice (my assessment, partly inferred).
   - Your `Event` type and Bevy's `Event` trait share the type namespace. Bevy's `#[derive(Event)]`
     lives in the macro namespace, so the derive should keep working. Inferred, not compiled.
   - Adding `use rules::prelude::Event;` fixes your name, but then Bevy's `Event` trait can only be
     named by its full path (for example in `T: Event` bounds).
   - `Observer` is central to Bevy 0.17 and later, so that clash will show up in many files.
   - Future risk: any later Bevy prelude addition named `Game`, `View`, `Action` and so on breaks
     every file that globs both preludes and uses that name. That is corrode's "minor update breaks
     you" scenario.

### Q3. How rustdoc renders it

7. Items are inlined into the prelude page (rule verified; `pub(crate)` case inferred). rustdoc
   book, Re-exports (https://doc.rust-lang.org/rustdoc/write-documentation/re-exports.html): "If a
   public item comes from a private module, it will be inlined." The example is
   `mod private_module { pub struct Public; } pub mod public_mod { pub use super::private_module::Public; }`,
   where `Public` is "inlined here since `private_module` is private."
   - A `pub(crate)` module does not appear in public docs, so I expect the same behaviour; I did
     not render it.
   - Result: each item gets a full page at `rules/prelude/struct.Game.html`. The crate root page
     lists only the `prelude` module plus the crate docs, so it is almost empty.
   - `#[doc(inline)]` is not needed. Doc comments and most attributes are carried along when an
     item is inlined.
   - The re-exported `static_card_definition` module should also be inlined as a module (not
     verified).

### Q4. rust-analyzer and the name `prelude`

8. The demotion applies to any module named exactly `prelude`, in any crate, including your own
   (verified in https://raw.githubusercontent.com/rust-lang/rust-analyzer/master/crates/hir-def/src/find_path.rs).
   - The path choice records "Whether any segment of this path is named `prelude`" as
     `has_prelude_segment`, set with `has_prelude_segment |= name == sym::prelude;`.
   - Paths are ranked by stability first, then by the prelude flag, then by length:
     ```rust
     .then_with(|| {
         if prefer_prelude {
             other.has_prelude_segment.cmp(&current.has_prelude_segment)
         } else {
             current.has_prelude_segment.cmp(&other.has_prelude_segment)
         }
     })
     .then_with(|| (current.path.len()).cmp(&(other.path.len() + 1)))
     ```
   - This ranking (`Choice::try_select`) is used for paths inside the local crate
     (`calculate_best_path_local`), not only for paths into dependencies.
   - The test `avoids_prelude_when_prefer_prelude_false` shows `krate::module::sub::Foo`
     (4 segments) chosen over `krate::prelude::Foo` (3 segments).
   - It is not tied to `#[prelude_import]`. That is a separate early return, `find_in_prelude`,
     which handles only the language prelude.
   - It does not apply to `api` or `public`: only the exact symbol `prelude` counts. So option (B)
     loses this benefit.

9. Correction: this only became true in April 2026. Before PR #22098
   (https://github.com/rust-lang/rust-analyzer/pull/22098, merged 2026-04-26, closing #21914), the
   flag was only computed when the setting was on. As that issue put it, "with
   `prefer_prelude: false` (the default) the flag was always `false` … A shorter
   `krate::prelude::Foo` won." The config docs say: `imports.preferPrelude`, "Default: `false`.
   Prefer import paths containing a `prelude` module." Older rust-analyzer builds, or a user who
   sets `preferPrelude = true`, will suggest `crate::prelude::X`. The CI grep is the real guarantee.

### Q5. Which path rustc prints in errors

10. Inside the crate, errors never show the prelude path (verified,
    https://raw.githubusercontent.com/rust-lang/rust/master/compiler/rustc_middle/src/ty/print/pretty.rs).
    `try_print_visible_def_path_recur` has `if def_id.is_local() { return Ok(false); }`, so local
    items are printed by their definition path, or trimmed to the bare name.

11. Downstream, errors print `rules::prelude::X` whenever the name is not trimmed (verified from
    source; not compiled).
    - `visible_parent_map` (`rustc_metadata/src/rmeta/decoder/cstore_impl.rs`) does a breadth-first
      search over public children only (`if !child.vis.is_public() { return; }`) to find
      "minimal-length paths". With a prelude-only API, the only visible path is `rules::prelude::X`.
    - Trimming to the bare name happens only "if a symbol name can only be imported from one place
      for a type, and as long as it was not glob-imported anywhere in the current crate".
    - So `Event` and `Observer`, which exist in Bevy too, will print in full, for example
      `rules::prelude::Event`. Noisy but accurate.
    - How the glob-import condition affects names brought in by `use rules::prelude::*` was not
      tested.
    - Under option (A), the same errors would read `rules::Event`.

### Q6. The alternatives

12. Proposal (prelude-only).
    - For: downstream crates see exactly one path per item; the CI regex is simple; rust-analyzer's
      demotion matches the in-crate rule (Finding 8); with `unreachable_pub` on, `prelude.rs` is
      provably the whole API. That lint "triggers for `pub` items not reachable from other crates -
      that means neither directly accessible, nor reexported (with `pub use`)"
      (https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html).
    - Against: the prelude's usual meaning is inverted; the name signals "glob this", and Clippy
      agrees: "Wildcard imports are allowed from modules that their name contains `prelude`"; and
      `rules::Game` fails to resolve.
    - Grep failure modes (my analysis): matching on `crate::prelude::` misses
      `use crate::{prelude::Game}`, `super::prelude`, and `use crate::prelude as p; p::Game`. Use
      `\bprelude\b` across `src/`, allowing only the declaration in `lib.rs` and `prelude.rs`
      itself. Any `#[macro_export]` macro must use `$crate::prelude::…`, because the internal
      modules are `pub(crate)`; those need an allowlist entry. Intra-doc links that mention the
      prelude will also match.

13. (A) Root re-exports.
    - For: the strongest convention (PingCAP, corrode); `rules::Game` is shortest; downstream errors
      read `rules::Game`.
    - Against: rust-analyzer will suggest `crate::Game` inside the crate, because it is shortest and
      has no `prelude` segment, so the IDE fights the CI rule on every auto-import. The grep is
      harder: the case convention (`crate::[A-Z]`) does not cover the lowercase module
      `static_card_definition` or grouped imports such as `crate::{cards, Game}`. Forcing one
      import per line needs rustfmt's `imports_granularity`, which is nightly-only.

14. (B) Same as the proposal, but named `api` or `public`.
    - For: honest naming, and no Clippy wildcard exemption.
    - Against: it loses the rust-analyzer demotion. `crate::api::Game` will often be shortest, so
      the IDE fights the rule just as in (A). Strictly worse than the proposal for "tooling over
      review".

15. (C) Public module tree, no re-exports.
    - For: one path everywhere, enforced by the compiler, with no grep at all.
    - Against: the file layout becomes the API, which undercuts information hiding: moving a file
      breaks every consumer. It cancels `pub(crate)` on modules. Paths get long, such as
      `rules::cards::object::ObjectId`. Because every consumer is inside the workspace, the
      breakage only costs a refactor, but it is still churn.

16. (D) Facade crate over `rules_core`.
    - For: the compiler enforces the rule. `rules_core` cannot name `rules`, because that would be
      a dependency cycle.
    - Against: `rules_core` must expose the API through public paths, so it has a second public
      surface. Stopping the front end from depending on `rules_core` directly needs another tool;
      cargo-deny's `wrappers` feature looks like a fit, not verified. The integration tests and the
      bench would move to the facade crate. More machinery than a 4k-line crate needs.

17. API and semver tooling (verified).
    - cargo-public-api's item processor inlines `pub use` re-exports and lists an item once per
      reachable path: "the same item will be reachable by different paths." Under the proposal it
      would print `pub struct rules::prelude::Game`. Keeping both a root and a prelude path would
      double every entry.
    - cargo-semver-checks has handled moved and re-exported items since v0.17
      (https://predr.ag/blog/moving-and-reexporting-rust-type-can-be-major-breaking-change/).
    - The Cargo semver reference lists "Major: renaming/moving/removing any public items".
    - Since `rules` is unpublished, the most useful of these is a cargo-public-api snapshot used as
      an API review gate.

### Q7. Verdict

18. Weak convention. Nothing in rustc, rustdoc, or Clippy treats the design as wrong. Downstream,
    the compiler's import suggestions should point users at the right path (expected, not
    verified). The surprise is semantic: "prelude" means an optional glob convenience. The one
    strong-ish harm is that this meaning invites `use rules::prelude::*` next to
    `use bevy::prelude::*`, which runs into `Event` and `Observer`.

## Final recommendation

Keep the proposal and keep the name `prelude`. It is the only option without a second crate where
the IDE (rust-analyzer since April 2026), the compiler's downstream paths and a simple grep all
agree. If you keep it, change these:

1. Ban globbing the prelude in the front end. No Clippy lint can ban a glob of one specific path,
   and `wildcard_imports` exempts preludes, so use a grep for `use rules::prelude::\*` in the
   front-end crate. Explicit imports win over `bevy::prelude::*` (per the Reference). Say this in
   the prelude's module doc.
2. Decide on renaming `Event` and `Observer` (for example `GameEvent`, `Viewer`). This is a domain
   naming call. It removes both Bevy collisions, which otherwise appear in many front-end files.
3. Harden the grep to `\bprelude\b` across `crates/rules/src`, allowing only the declaration and
   `prelude.rs`, plus `$crate::prelude` inside exported macros.
4. Enable `unreachable_pub` as a warning so every `pub` item must appear in `prelude.rs`.
   Optionally add a cargo-public-api snapshot.
5. Do not add root re-exports. That would create a second downstream path and make rust-analyzer
   suggest `crate::Game` inside the crate.
6. Pin a rust-analyzer version from after 2026-04-26 in the team setup. Treat the IDE ranking as a
   convenience; the grep is the guarantee.

Choose (D) instead if you want the compiler, not a grep, to enforce the in-crate rule.

## What I could not verify

- I compiled nothing. Rendering, error output, and Bevy's `#[derive(Event)]` working next to your
  `Event` type are inferred from source and docs.
- rustdoc treating `pub(crate)` modules exactly like private ones for inlining.
- The exact rust-analyzer release (and rustup `rust-analyzer` component for Rust 1.99) that
  includes PR #22098.
- Any crate whose API exists only behind a prelude or a single `api` module, and how such a crate
  was received.
- cargo-deny `wrappers` for fencing off `rules_core`, and whether rustc skips transitive
  dependencies when computing visible paths under (D).
- Whether the "not glob-imported" condition for name trimming applies to names brought in by
  `use rules::prelude::*` in the front end.
