# Rust module file layout: mod.rs, self-named files, and Clippy

Researcher report, session 01e, kept in full. Local checks by the session agent follow it.

## Summary

Every item was checked against primary sources, with one important correction under 4d. The
Reference and the Book prefer `foo.rs` + `foo/`, and having both `foo.rs` and `foo/mod.rs` is error
E0761. `module_inception` (style, warn by default) also fires on a file-backed `mod foo;` inside
`foo/mod.rs`. Correction: `allow-private-module-inception` looks at whether the outer module is
`pub`, not the inner one. So with `pub mod foo;` in the parent, a private inner `mod foo;` plus
`pub use foo::Thing;` is still linted. Issue #13259 about this is open. The Clippy source also has
a new lint, `definition_in_module_root` (restriction, since 1.99), that enforces the "mod.rs is only
`mod` and `pub use`" pattern. Its docs say outright that it conflicts with `module_inception`.

## Findings

### 1. Rust Reference: Modules (https://doc.rust-lang.org/reference/items/modules.html). Verified

- Default layout: "Ancestor module path components are directories, and the module's contents are
  in a file with the name of the module plus the `.rs` extension." The example is `crate::util` to
  `util.rs` and `crate::util::config` to `util/config.rs`.
- mod.rs form and the conflict: "Module filenames may also be the name of the module as a
  directory with the contents in a file named `mod.rs` within that directory. The above example
  can alternately be expressed with `crate::util`'s contents in a file named `util/mod.rs`. It is
  not allowed to have both `util.rs` and `util/mod.rs`."
- The note preferring one form: "Prior to `rustc` 1.30, using `mod.rs` files was the way to load a
  module with nested children. It is encouraged to use the new naming convention as it is more
  consistent, and avoids having many files named `mod.rs` within a project."
- Defining mod-rs and non-mod-rs files, in the `#[path]` section: "'mod-rs' source files are root
  modules (such as `lib.rs` or `main.rs`) and modules with files named `mod.rs`. 'non-mod-rs'
  source files are all other module files."
- `#[path]` outside inline blocks: "For `path` attributes on modules not inside inline module
  blocks, the file path is relative to the directory the source file is located." So
  `#[path = "foo.rs"] mod c;` in `src/a/b.rs` loads `src/a/foo.rs` as `crate::a::b::c`. In
  `src/a/mod.rs` it loads the same file as `crate::a::c`.
- `#[path]` inside inline blocks: "Paths for `path` attributes inside inline module blocks in a
  mod-rs file are relative to the directory of the mod-rs file including the inline module
  components as directories. For non-mod-rs files, it is the same except the path starts with a
  directory with the name of the non-mod-rs module." The page's example: from `src/a/b.rs`,
  `mod inline { #[path = "other.rs"] mod inner; }` resolves to `src/a/b/inline/other.rs`.
- Error code E0761 (https://doc.rust-lang.org/error_codes/E0761.html): "Multiple candidate files
  were found for an out-of-line module." Its example error text is "file for module
  `ambiguous_module` found at both ambiguous_module.rs and ambiguous_module/mod.rs".
- Why the note is still there (context, not normative text): Reference PR #1703, which would have
  removed the preference, was closed on 2025-03-20 (https://github.com/rust-lang/reference/pull/1703).
  traviscross reported from lang triage that "the 2018 lang consensus was indeed to encourage one
  way (`foo.rs`) over the other way (`foo/mod.rs`), while at the same time not deprecating the old
  way, and we did not have any consensus for changing that."

### 2. Rust Book ch. 7.5 (https://doc.rust-lang.org/book/ch07-05-separating-modules-into-different-files.html). Verified

Under the heading "Alternate File Paths":

- "So far we've covered the most idiomatic file paths the Rust compiler uses, but Rust also
  supports an older style of file path."
- The two candidate paths are listed as "src/front_of_house.rs (what we covered)" and
  "src/front_of_house/mod.rs (older style, still supported path)". The `hosting` submodule gets the
  same pair.
- On mixing: "If you use both styles for the same module, you'll get a compiler error. Using a mix
  of both styles for different modules in the same project is allowed but might be confusing for
  people navigating your project."
- The downside it names: "The main downside to the style that uses files named mod.rs is that
  your project can end up with many files named mod.rs, which can get confusing when you have them
  open in your editor at the same time."

### 3. RFC 2126 path clarity (https://rust-lang.github.io/rfcs/2126-path-clarity.html). Verified

What it changed:

- "A `foo.rs` and `foo/` subdirectory may coexist; `mod.rs` is no longer needed when placing
  submodules in a subdirectory."
- "You can write `mod bar;` statements even when not in a `mod.rs` or equivalent; in this case,
  the submodules must appear within a subdirectory with the same name as the current module. Thus,
  `foo.rs` can contain `mod bar;` if there is also a `foo/bar.rs`."
- "It is not permitted to have both `foo.rs` and `foo/mod.rs` at the same point in the file system."
- "The use of `mod.rs` continues to be allowed without any deprecation. It is expected that tooling
  like Clippy will push for at least style consistency within a project, and perhaps eventually
  across the ecosystem."

Its rationale, in the section "The `mod.rs` file", which opens with "A final issue, though far
less important…":

- Learnability: "the paths in the module system aren't quite in direct correspondence with the
  file system… A simpler rule would be: the path to a module's file is the path to it within Rust
  code, with `.rs` appended."
- Ergonomics: "one often ends up with many `mod.rs` files open, and thus must depend on editor
  smarts to easily navigate between them."
- Refactoring: "having to use `mod.rs` means you often have to move existing files around."
- The benefit it grants mod.rs: "the code for a parent module and its children live more closely
  together (not necessarily desirable!) and that it provides a consistent story with `lib.rs`."

### 4. Clippy lints

All lint text below was read from the master source, which the lint list page is generated from.
Default levels come from the README table (https://github.com/rust-lang/rust-clippy): style is
warn, pedantic and restriction are allow.

4a. `mod_module_files`
(https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/module_style.rs). Verified.

- Group restriction, default allow, `#[clippy::version = "1.57.0"]`.
- What it does: "Checks that module layout uses only self named module files; bans `mod.rs`
  files."
- Why restrict this: "Having multiple module layout styles in a project can be confusing."
- Example: it flags `src/stuff/mod.rs` and suggests `src/stuff.rs` plus `src/stuff/stuff_files.rs`.
- It skips `mod.rs` under a `tests` directory, per the source comment "Using `mod.rs` in
  integration tests is a common pattern".

4b. `self_named_module_files` (same file). Verified.

- Group restriction, default allow, since 1.57.0.
- What it does: "Checks that module layout uses only `mod.rs` files."
- Its rationale is the same sentence as 4a. The example is the reverse of 4a.
- It only fires on a module file that itself declares out-of-line children (the
  `contains_external` check in the source).
- Related lint in that file: `inline_modules` (restriction, 1.97.0), which bans
  non-`#[cfg(test)]` inline modules.

4c. `module_name_repetitions`
(https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/item_name_repetitions.rs).
Verified, with a correction on the group.

- Group restriction (not pedantic), default allow, `#[clippy::version = "1.33.0"]`.
- Moved from pedantic in PR #13541 (https://github.com/rust-lang/rust-clippy/pull/13541), merged
  2024-10-13, in the merge window for Clippy 1.84.
- What it does: "Detects public item names that are prefixed or suffixed by the containing public
  module's name."
- Why it is bad: "It requires the user to type the module name twice in each usage… the style used
  in the Rust standard library; e.g. `io::Error` and `fmt::Error` rather than `io::IoError`…"
- Example: `mod cake { struct BlackForestCake; }` should become `struct BlackForest`.
- Since PR #13444 it only checks public items inside public modules.
- Config options: `allow-exact-repetitions` (default `true`), "Whether an item should be allowed to
  have the same name as its containing module", and `allowed-prefixes` (default
  `["to","as","into","from","try_into","try_from"]`).

4d. `module_inception` (same file). Lint text verified; the config's meaning corrected.

- Group style, default warn, version "pre 1.29.0".
- What it does: "Checks for modules that have the same name as their parent module."
- Why it is bad: "A typical beginner mistake is to have `mod foo;` and again `mod foo { .. }` in
  `foo.rs`. The expectation is that items inside the inner `mod foo { .. }` are then available
  through `foo::x`, but they are only available through `foo::foo::x`. If this is done on purpose,
  it would be better to choose a more representative module name."
- Example: `// lib.rs mod foo;` with `// foo.rs mod foo { ... }`.
- It fires on a file-backed `mod foo;` in `foo/mod.rs`. The check runs on HIR
  `ItemKind::Mod(ident, _)`, which doesn't distinguish inline from file modules. Issue #10842 shows
  it on exactly this layout (https://github.com/rust-lang/rust-clippy/issues/10842):
  "`// thing/mod.rs  mod thing; … pub use thing::Thing;` … Running `clippy` warns about module
  inception on the private `mod thing`."
- `allow-private-module-inception` exists. Docs
  (https://doc.rust-lang.org/clippy/lint_configuration.html#allow-private-module-inception):
  "Whether to allow module inception if it's not public." Default `false`. Added in PR #10917
  (merged 2023-06-10), listed under Rust 1.72 in the CHANGELOG.
- Correction: "not public" means the outer module. The source condition is
  `prev.name == ident.name && … && (!self.allow_private_module_inception || prev.is_public)`, where
  `prev` is the containing module. The ui-toml test agrees: under "// Don't lint" it has
  `mod foo { … pub mod foo { pub mod bar {} } }`, a private outer and a public inner
  (https://github.com/rust-lang/rust-clippy/blob/master/tests/ui-toml/module_inception/module_inception.rs).
  - Outer `pub mod foo;`, inner private `mod foo;` plus `pub use foo::Thing;`: still linted. Issue
    #13259, open, labelled C-bug and I-false-positive
    (https://github.com/rust-lang/rust-clippy/issues/13259): "(even with the config
    `allow-private-module-inception` enabled the warning occurs…)".
  - Outer plain `mod foo;`: allowed, whatever the inner module's visibility.
  - Outer `pub(crate) mod foo;`: probably allowed, inferred from `Visibility::is_public()`, not
    run.

### 5. Community practice

Who uses which style, verified by GitHub directory listings in October 2026:

| Project | Style | Evidence |
|---|---|---|
| tokio | mod.rs | `tokio/src/fs/mod.rs`. Alice Ryhl: "I disagree with the 'official' guidance… In Tokio, we mostly use `mod.rs` files, since it just makes more sense to have `fs`-related code inside the `tokio/src/fs` directory." Later: "Tokio is consistently using `mod.rs`." (https://users.rust-lang.org/t/module-mod-rs-or-module-rs/122653) |
| rustc | mod.rs, in places | `compiler/rustc_middle/src/ty/mod.rs`, about 96 KB of code. RalfJung: "Not even the compiler itself was ever systematically adjusted" (PR #1703). |
| bevy | mod.rs | `crates/bevy_ecs/src/system/mod.rs`, about 62 KB |
| serde | mod.rs | `serde_core/src/de/mod.rs`, about 84 KB |
| rust-analyzer | self-named | `crates/hir-ty/src/infer.rs` next to an `infer/` directory |

- rust-analyzer's style guide doesn't discuss `mod.rs`. It says: "By default, avoid re-exports.
  Rationale: for non-library code, re-exports introduce two ways to use something and allow for
  inconsistency."
  (https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/style.md)
- rustup issue #3883 is an open style debate (https://github.com/rust-lang/rustup/issues/3883).
  djc: "[`foo.rs`]… non-trivial cognitive overhead because it means each module appears twice in
  the directory tree".

The facade or aggregator pattern:

- For it, the strongest source is Clippy's `definition_in_module_root` (restriction,
  `#[clippy::version = "1.99.0"]`, PR #16965,
  https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/definition_in_module_root.rs).
  - What it does: "Checks for definitions (structs, functions, traits, etc.) in `mod.rs` files.
    `lib.rs` and `main.rs` are not checked."
  - Why restrict this: "`mod.rs` is well-suited to acting as a table of contents — listing
    submodules and re-exports while leaving definitions to named files. Naming each file after its
    primary definition keeps filenames descriptive and unique, makes editor tabs and search results
    easier to scan, and makes the filesystem tree mirror the module tree…"
  - Its notes on the conflict: "This lint is most useful alongside `self_named_module_files`… If a
    definition's name matches its parent module's name, moving it produces `foo/foo.rs`, which
    `module_inception` flags — projects in that situation typically also
    `allow(module_inception)`."
  - It allows `use` and `pub use`, `mod`, and `#[macro_export]` macros. It flags struct, enum, fn,
    impl, trait, const, static and similar items.
- For it, user reports: issue #10842 (`thing/mod.rs` with private submodules plus re-exports). An
  IRLO commenter, quoted in the users-forum thread: "for me I keep mod.rs very minimal, so the
  interesting functionality goes into a submodule anyway. I consistently use the mod.rs structure."
- Against it: the rust-analyzer "avoid re-exports" rule above, and the `module_inception`
  rationale, which says to "choose a more representative module name" rather than `foo/foo.rs`.

Editor labels, verified:

- VS Code 1.88 (March 2024, https://code.visualstudio.com/updates/v1_88#_custom-labels-for-open-editors)
  lets you customize "the display label for editor tabs and the Open editors view… under the
  `workbench.editor.customLabels.patterns` setting." Templates can use `${filename}`,
  `${extname}`, `${dirname}` and `${dirname(N)}`.
- kpreid's tip (https://users.rust-lang.org/t/tip-improving-vs-codes-display-of-mod-rs-files/111329)
  uses `"**/mod.rs": "${dirname}/mod.rs"`.

### 6. Privacy (https://doc.rust-lang.org/reference/visibility-and-privacy.html). Verified

- "If an item is private, it may be accessed by the current module and its descendants."
- "`pub(in path)` makes an item visible within the provided `path`."
- "`pub(super)` makes an item visible to the parent module. This is equivalent to
  `pub(in super)`."
- Note: "This syntax only adds another restriction… To access an item, all of its parent items up
  to the current scope must still be visible as well."
- So a private item in `foo::bar` is visible only to `foo::bar` and its descendants, and the
  sibling `foo::baz` can't see it. A `pub(super)` item in `foo::bar` equals `pub(in foo)`, so it is
  visible within `foo`, including `foo::baz`. The Reference's own example confirms this:
  `pub(super) fn super_mod_visible_fn()` in `outer_mod::inner_mod` is callable from
  `outer_mod::foo`, but not from the crate root.

## Gaps

- The rendered lint list page wasn't compared character by character with the source.
- `pub(crate)` outer module with `allow-private-module-inception`: inferred, not run.
- No `module_inception` test uses an out-of-line `foo/mod.rs` with `mod foo;`. That it fires comes
  from the HIR source plus issue #10842.
- Project surveys are spot checks, one directory per project.
- Apart from the `definition_in_module_root` docs, no authoritative written guide recommends
  mod.rs-only facades.

## Local checks (session agent)

Run in a scratch crate and on a copy of the `mod-rs-layout` branch (#24):

- Clippy 0.1.96 (the pinned toolchain): `self_named_module_files` flags `qux.rs` next to `qux/`,
  and leaves leaf files alone. `#[allow(clippy::self_named_module_files)]` on one `mod` item
  exempts that module.
- Clippy 0.1.96: `module_inception` fires on a file-backed `mod foo;` in `foo/mod.rs`. With
  `allow-private-module-inception = true`, it stops firing when the outer module is private or
  `pub(crate)`, and still fires when the outer module is `pub`. This settles the researcher's
  inferred `pub(crate)` case.
- `definition_in_module_root` is an unknown lint on Clippy 0.1.96 and 0.1.92. On Clippy 0.1.99
  (Rust 1.99.0, released) it exists and flags a `fn` in `foo/mod.rs`.
- The `mod-rs-layout` tree with the pin set to 1.99.0: `cargo clippy --workspace --all-targets --
  -D warnings` passes with no new warnings. `definition_in_module_root` flags only
  `game/mod.rs`: the `Game` struct and its `impl`. A
  `#![expect(clippy::definition_in_module_root, reason = "...")]` at the top of `game/mod.rs`
  silences both, and the checks pass with the lint denied.
