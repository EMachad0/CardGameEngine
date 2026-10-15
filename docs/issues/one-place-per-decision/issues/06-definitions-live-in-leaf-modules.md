# Definitions live in leaf modules

Status: resolved
Category: enhancement
Type: task
Blocked by: #24 merged
Branch: `leaf-definitions`, new, from `main` after #24 merges. New draft PR with base `main`.

## Why

`docs/conventions/file-structure.md` says: "Keep definitions in leaf modules, so that every module
can hide its decision. A module with children is a directory whose `mod.rs` holds only `mod`
declarations." A definition in a parent is open to every child, so the parent can't hide its
decision from them. Re-exports in `mod.rs` are ticket 07's, and the lint allows them, so leave
them. #24 enforces the first half with `self_named_module_files`. Clippy's
`definition_in_module_root` enforces the second half, and it first ships in Rust 1.99.

## What to build

- Bump `rust-toolchain.toml` from 1.96.0 to 1.99.0. The learner approved this bump.
- Add `definition_in_module_root = "deny"` under `[workspace.lints.clippy]` in `Cargo.toml`, next
  to `self_named_module_files`.
- `crates/rules/src/game/mod.rs` holds `Game` and its `impl`, and every child file reads `Game`'s
  private fields. Splitting that is separate work. Exempt this one file with
  `#![expect(clippy::definition_in_module_root, reason = "...")]` at its top. The reason states
  why the file is exempt today. `expect`, not `allow`: once the file holds no definitions, the
  unfulfilled expectation warns, so the exemption can't outlive its reason.
- Fix any other `mod.rs` that holds a definition by moving it to a leaf named for its decision.
  A leaf named like its directory fails the doc's name test. A local run on 1.99 found only
  `game/mod.rs`, so expect none.
- If 1.99 raises new warnings elsewhere, fix them in their own commit, and list them in the PR
  body.

## Out of scope

Splitting `game`. Any rename beyond what a moved definition needs.

## Done when

- The toolchain is 1.99.0, and `definition_in_module_root` is denied workspace-wide.
- `game/mod.rs` is the only exemption, through `expect`.
- The checks pass, with `tests/spec/` unchanged.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/26
