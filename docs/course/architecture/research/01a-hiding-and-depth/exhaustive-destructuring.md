# Exhaustive destructuring in Rust: community views and lints

Researcher report, session 01a. Kept in full.

## Summary

This idiom is real, and recent writing by Rust practitioners recommends it, mostly under the name **"exhaustive destructuring"**. It is not in the official Rust API Guidelines. Clippy has long had a lint that pushes the other way (`unneeded_field_pattern`). A lint that enforces the idiom (`rest_pattern_accessible_field`) was only merged in July 2026, so expect it on nightly Clippy only for now. `#[non_exhaustive]` forces `..` only outside the defining crate.

## Findings

### 1. Sources that recommend it: CONFIRMED

- **Antoine Vandecrème, "In praise of exhaustive destructuring"** [SECONDARY, practitioner blog]. His example is a validation walk, `is_dangerous`. After adding a field, *"We would now get `error[E0027]: pattern does not mention field 'wind_speed'`."* *"Note that you must be explicit about unused fields and resist the temptation of using the `..` pattern for this trick to work."* On `From` conversions between layers: *"If you add a field in one layer, the compiler will force you to decide whether that field should be propagated to the other layers or not."* He also notes that `self` cannot be destructured in the parameter list, so methods need `let Self { .. } = self;`. https://antoine.vandecreme.net/blog/exhaustive-destructuring-praise/
- **Matthias Endler (corrode), "Patterns for Defensive Programming in Rust"** (2025-11-08) [SECONDARY, consultancy blog].
  - For a manual `PartialEq`, the defensive approach writes `let Self { size, toppings, crust_type, ordered_at: _ } = self;`. The point is that with field access, the *"implementation still compiles, but is it correct? ... the compiler won't remind you to think about it."*
  - On defaults, he says to avoid `..Default::default()` and instead `let Foo { field1, field2, field3, field4 } = Foo::default();` so that *"The compiler will complain when new fields are added to the struct."*

  https://corrode.dev/blog/defensive-programming/
- **Samuel Selleck, "Tips for Defensive Programming in Rust"** [SECONDARY]: *"If you find yourself using all fields of a struct in a function, such as when implementing `std::fmt::Display`: use struct destructuring."* https://samuelselleck.com/blog/tips-for-defensive-programming-in-rust/
- **Rust API Guidelines: NOT FOUND.** No recommendation there, and no conference talk transcript either.
- **Use cases named across these sources:** manual `PartialEq` (corrode), `Display` (Selleck), `From` conversions between layers and validation walks (Vandecrème), building from defaults (corrode). `Hash`, `Debug`, `Clone` and serialization follow the same logic, but no source names them explicitly.

### 2. Costs and counterarguments: CONFIRMED (thin)

- **Verbosity.** Endler: *"Yes, it's slightly more verbose, but what you gain is that the compiler will force you to handle all fields explicitly"* (said about struct literals, same tradeoff). [SECONDARY]
- **Overuse.** A reply in the r/rust thread on Vandecrème's post: *"There's definitely a place for exhaustive destructuring -- I tend to use it when converting from non-validated to validated, for example -- but exhaustive destructuring everywhere leads to a lot of supplementary code compared to destructuring with wildcard or plain field access."* [SECONDARY, seen through a search snippet only; Reddit would not load.] https://www.reddit.com/r/rust/comments/1uua63n/in_praise_of_exhaustive_destructuring/
- **The tradeoff stated on the Clippy tracker.** In issue #10666, which proposed the opposite lint, the listed drawback is: *"This will prevent a compilation error if additional fields are added to the struct. Some programmers may want that error to remind them to do something with the newly added fields."* Maintainer Alexendoo replied: *"That drawback is a pretty big one, I would say this would have to be `restriction`."* [PRIMARY] https://github.com/rust-lang/rust-clippy/issues/10666
- **Cross-crate and visibility limits.** See section 3: `#[non_exhaustive]` makes the idiom impossible downstream. The new lint also skips fields that are not visible from the pattern's module.

### 3. Lints and `#[non_exhaustive]`

- **`clippy::rest_pat_in_fully_bound_structs`** is **restriction**, since Clippy 1.43. CORRECTED if expected to enforce the idiom. It only fires when `..` is redundant because every field is already bound: *"a match on a struct that binds all fields but still uses the wildcard pattern."* It does not catch `Foo { a, .. }` with unbound fields. [PRIMARY] https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/matches/mod.rs
- **`clippy::unneeded_field_pattern`** is **restriction** and works against the idiom. It flags `Foo { a: _, ... }` because *"Using `..` instead is shorter and leaves the focus on the fields that are actually bound."* Do not enable it alongside the idiom. [PRIMARY] https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/misc_early/mod.rs
- **`clippy::rest_pattern_accessible_field`** (new, **restriction**) enforces the idiom: *"Disallow the use of rest patterns for accessible fields ... It might lead to unhandled fields when the struct changes."* Its suggested fix is `let S { a, b, c: _ } = s;`. A sibling lint, **`unnecessary_rest_pattern`** (restriction), flags `..` that captures no fields. Both came in PR #15000, merged 2026-07-11. The source carries `#[clippy::version = "1.99.0"]`, likely a placeholder, so treat these lints as nightly-only until they appear in a stable Clippy. Per the code, the lint stays quiet when `..` is required (non-exhaustive or inaccessible fields). The PR's lintcheck reported 773 hits on popular crates, which shows how rarely the idiom is used by default. [PRIMARY] https://github.com/rust-lang/rust-clippy/pull/15000, https://github.com/rust-lang/rust-clippy/blob/master/clippy_lints/src/rest_when_destructuring_struct.rs
- **Requiring it only in chosen functions.** Restriction lints are off by default, and lint levels can be set on any item, so `#[deny(clippy::rest_pattern_accessible_field)]` on a single `fn` or `impl` should scope it. Not tested for this lint. On stable today, the only enforcement is the compiler itself: a pattern without `..` that misses a field fails with E0027. The Reference: *"If `..` is not used, a struct pattern used to match a struct is required to specify all fields"* [PRIMARY]. The same rule applies to enum struct variants. https://doc.rust-lang.org/reference/patterns.html
- **`#[non_exhaustive]`: CONFIRMED, outside the defining crate only.** The Reference: *"Within the defining crate, `non_exhaustive` has no effect."* Outside the crate: *"When pattern matching on a non-exhaustive variant (struct or enum variant), a StructPattern must be used which must include a `..`."* [PRIMARY] https://doc.rust-lang.org/reference/attributes/type_system.html

### 4. How common it is

A recognized but niche defensive idiom, most often called "exhaustive destructuring". Endler calls it *"rarely documented."* The Clippy discussion treats it as a deliberate opt-in style, never a default. Clippy's `unneeded_field_pattern` and Rust by Example's `Foo { y, .. }` show that `..` is the mainstream default.

## Sources

- Kept: Rust Reference, Patterns and Type system attributes [PRIMARY].
- Kept: Clippy lint source, PR #15000, issue #10666 [PRIMARY].
- Kept: Vandecrème, corrode, Selleck [SECONDARY].
- Dropped: a Reddit thread on destructuring syntax and an internals thread on wildcard destructuring, both off topic.

## Gaps

- Which stable Clippy release ships `rest_pattern_accessible_field`. Check the Clippy CHANGELOG first.
- No primary source from Rust project members or a conference talk recommends the idiom.
- Function-scoped `#[deny]` with the new lint is untested.
- The Reddit criticism comes from a search snippet only.
