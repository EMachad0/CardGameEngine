# Trait impl placement: practitioners and large projects

Researcher report, outside a session, for PR #70's doc gaps, kept in full. Question: what Rust
practitioners and large projects recommend about where a trait `impl` goes, and why.

## Summary

No major Rust project I checked has a written rule on where trait `impl` blocks go. The official Rust docs only say location "matters little". What exists is informal opinion and the code layout of popular crates. Those point the same way:

- **Default:** put trait impls next to the type when the type is yours.
- **Trait side:** group impls of your own trait for foreign types next to the trait.
- **Own module:** put feature-gated impls of third-party traits (serde, rayon, arbitrary, borsh) in a dedicated module, such as `serde.rs` or `external_trait_impls/`.

One hard constraint shapes the choice: an impl in another module can only use the type's private fields if you widen their visibility, for example to `pub(crate)`. That pushes impls that touch private fields back toward the type.

## Findings

### Primary-source statements

1. **The std docs say location barely matters.** The `impl` keyword docs read: *"It matters little where a trait implementation is defined; its functionality can be brought into scope by importing the trait it implements."* — verified. This describes how the language works; it is not style advice. [std `impl` keyword](https://doc.rust-lang.org/std/keyword.impl.html)

2. **RFC 735 names the cost of scattering impls, and says trait impls already have it.** This RFC let inherent impls live outside the type's module. Its Drawbacks section says: *"Consistency and ease of finding method definitions by looking at the module the type is defined in, has been mentioned as an advantage of this limitation. However, trait `impl`s already have that problem…"* — verified. So the language designers accepted that trait impls can sit anywhere in the crate. The cost they named is how easy impls are to find. [RFC 0735](https://github.com/rust-lang/rfcs/blob/master/text/0735-allow-inherent-impls-anywhere.md)

### Documented project rules

3. **The Rust API Guidelines (C-SERDE) say to gate serde impls behind a feature, but not where to put them.**
   - Verified quote: *"If a crate does not already depend on Serde for other reasons, it may wish to gate Serde impls behind a Cargo cfg… the name of the Cargo cfg should be simply `"serde"`."*
   - The guideline's own example puts `#[cfg(feature = "serde")] impl Serialize for T` right after `pub struct T`, or uses `cfg_attr(feature = "serde", derive(...))` on the type.
   - So the official guidance on feature naming implicitly shows impls next to the type. It does not require a separate module.

   [API Guidelines – Interoperability](https://rust-lang.github.io/api-guidelines/interoperability.html)

4. **rust-analyzer's style guide covers order inside a file, not which file.** Verified: *"Put `struct`s and `enum`s first, functions and impls last."* Its stated reason is to *"get the sense of the API by visually scanning the file."* That assumes impls live in the same file as their types, but the guide never says so. [rust-analyzer style.md](https://raw.githubusercontent.com/rust-lang/rust-analyzer/master/docs/book/src/contributing/style.md)

5. **Guides I checked that say nothing on impl placement:**
   - Embark's Rust guidelines: only covers rustfmt, parking_lot and clippy.
   - Microsoft's Pragmatic Rust Guidelines: no match for "impl block" or "orphan" in the agent text dump; its module/feature mentions are unrelated.
   - Matklad's "Large Rust Workspaces": about crate layout.

   All three were fetched and checked. [Embark](https://github.com/EmbarkStudios/rust-ecosystem/blob/main/guidelines.md), [MS Pragmatic Rust](https://microsoft.github.io/rust-guidelines/agents/all.txt), [matklad](https://matklad.github.io/2021/08/22/large-rust-workspaces.html)

### Individual opinions (forum and Stack Overflow)

6. **H2CO3: put the impl next to whichever item is yours; if both are yours, next to the type.** H2CO3 is a very prolific forum answerer, not a Rust team member. Verified quotes:
   - *"my convention is to group by whatever item is 'mine'… and by the type if both the type and the trait are mine."*
   - Reason: *"the type is the 'principal' entity, and the various behaviors it is bestowed are secondary… (I am out of control as to what traits it implements anyway, because 3rd-parties are always free to add specific or blanket impls.)"*
   - For a local trait on foreign types, he groups by trait, because splitting them across modules per foreign type *"would be painful"*. His example puts `impl ExternTrait for MyType` (a foreign trait on a local type) in `mytype.rs`.

   [users.rust-lang.org/87455](https://users.rust-lang.org/t/clean-code-where-to-put-trait-implementations/87455)

7. **Trade-off raised by the asker in that thread (mankinskin).** Impls next to the type mean that changing the trait touches many files. Impls next to the trait mean that changing the type touches many files.
   - scottmcm (Scott McMurray, a Rust lang team member) did not pick a location. Verified: *"If you're changing the trait more than you're adding new types implementing the trait, then consider whether you want an `enum` instead."* He later accepted that traits were right for the asker's case.
   - Same thread. [users.rust-lang.org/87455](https://users.rust-lang.org/t/clean-code-where-to-put-trait-implementations/87455)

8. **For `From<A> for B`, H2CO3 says either module is fine.** Verified: *"Putting `impl From<A> for B` in either the module of `A` or the module of `B` should be fine. Which one you prefer is mostly a philosophical question."* He adds that scattering *inherent* impls *"just because you can is usually not wise."* [users.rust-lang.org/67584](https://users.rust-lang.org/t/what-is-better-practice-to-implement-traits/67584)

   One post in this thread was missing from my fetch, probably deleted.

9. **drewtato and paramagnetic: the norm is next to the type, but organize by domain.**
   - drewtato (verified): *"I think normally, you'd just implement the traits next to the structs."* He also notes that a trait impl in another file can only reach private fields if you make them `pub(crate)`.
   - paramagnetic (verified): *"These sort of rules are extremely superficial… organize your code following the semantics of the domain… if that means a trait, a UDT, and an impl in the same file, then go for it."*

   [users.rust-lang.org/114569](https://users.rust-lang.org/t/idiomatic-way-for-structs-and-traits/114569)

10. **Letheed (2016): keep method and trait impls with the struct unless there is a reason not to.** Verified: *"I don't think there's an officially sanctioned pattern… Unless there's a need for it (like an impl defined inside a macro) I tend to keep the method and trait impls in the same file as their struct."* Letheed also warns against splitting files too much. [users.rust-lang.org/7785](https://users.rust-lang.org/t/code-structure-for-big-impl-s-distributed-over-several-files/7785)

11. **Shepmaster (Stack Overflow, `Default` in a separate file): possible, but it costs privacy and he would not do it.** Shepmaster is a top Rust answerer on Stack Overflow. His accepted answer:
   - Shows that it works if the fields are made `pub(crate)`.
   - Verified: *"If it's not an autogenerated file, I don't see how separate files make anything easier."*
   - His own suggestion puts `impl Default for Config` back in the same file as `impl Config`, and says the fields *"no longer need to be public."*

   I read this through the StackExchange API, because the site itself returned 403. [SO 68200668](https://stackoverflow.com/questions/68200668/how-can-i-separate-the-implementation-of-the-default-trait-for-a-structure-in-an)

### What popular crates do (observed layout; none of them states a rationale)

12. **indexmap.** Verified in `lib.rs`:
   - `#[cfg(feature = "borsh")] mod borsh; #[cfg(feature = "serde")] mod serde; #[cfg(feature = "sval")] mod sval;` plus a `mod rayon;` and a `mod arbitrary;`, where `arbitrary.rs` gates internally on both arbitrary and quickcheck.
   - std trait impls stay in `map`/`set`, next to the types.
   - The only rationale in the source is about doc ordering: *"Placed after `map` and `set` so new `rayon` methods on the types are documented after the 'normal' methods."*

   [indexmap lib.rs](https://raw.githubusercontent.com/indexmap-rs/indexmap/main/src/lib.rs)

13. **uuid.**
   - std trait impls (`Default`, `Hash`, `AsRef`, `From<Uuid> for Vec<u8>`) sit in `lib.rs` next to `Uuid`.
   - Third-party impls sit in `external.rs`: `#[cfg(feature = "arbitrary")] mod arbitrary_support; … borsh_support; … serde_support; … slog_support`. Verified.

   [uuid lib.rs](https://raw.githubusercontent.com/uuid-rs/uuid/main/src/lib.rs), [external.rs](https://raw.githubusercontent.com/uuid-rs/uuid/main/src/external.rs)

14. **hashbrown (rust-lang org).** `src/external_trait_impls/mod.rs` contains `#[cfg(feature = "rayon")] pub(crate) mod rayon; #[cfg(feature = "serde")] mod serde;`. Verified. [hashbrown](https://raw.githubusercontent.com/rust-lang/hashbrown/master/src/external_trait_impls/mod.rs)

15. **chrono uses a mixed layout.** serde code lives in a per-type submodule (`datetime::serde`), re-exported through a crate-level `pub mod serde`. Verified via `pub use super::datetime::serde::*`. So chrono keeps feature-gated impls close to each type, not in one central file. [chrono lib.rs](https://raw.githubusercontent.com/chronotope/chrono/main/src/lib.rs)

16. **Reasons for a dedicated module that I inferred myself (not stated by these projects):**
   - One `#[cfg]` on the `mod` line instead of one on every impl.
   - The optional dependency's `use` lines stay in one place.
   - Changes to the optional dependency's version are contained.

   These are my inference; none of the crates above says so in source or docs. Note also that `impl Serialize for T` written by hand needs either access to T's fields or public accessors. That is one more reason the guidelines show `cfg_attr(derive)` on the type itself.

### How this maps to your five cases

| Case | Weight of opinion |
|---|---|
| (a) Local trait, local type, different modules | Mostly next to the type (H2CO3, drewtato, Letheed). Grouping by trait is defensible when the trait changes more often than the types (asker in thread 87455). Not unanimous. |
| (b) `From<A> for B`, A private | "Either is fine" (H2CO3). Since A is private to its module, A's module is the only place that can see it without widening its visibility. That follows from the language rules, not from any quoted source. |
| (c) Local trait, foreign type | Next to the trait (H2CO3). No one disagreed. |
| (d) `Default` for a local type placed in a third module | Discouraged. Shepmaster moved it back to the type, and it forces `pub(crate)` fields. No source supported "put it where it's used". |
| (e) Feature-gated third-party impls | A dedicated module is common practice (indexmap, uuid, hashbrown). The API Guidelines example instead puts them inline next to the type. Both are accepted. |

## Sources
- **Kept:**
  - users.rust-lang.org 87455, 67584, 114569, 7785, 58794 (verified): direct practitioner positions.
  - SO 68200668 via the StackExchange API (verified): `Default` in a third module.
  - RFC 0735 (verified): language-design view on impl location.
  - std `impl` keyword docs (verified).
  - API Guidelines C-SERDE (verified).
  - rust-analyzer style.md (verified).
  - indexmap, uuid, hashbrown and chrono source (verified): observed conventions.
- **Dropped:**
  - Embark guidelines, MS Pragmatic Rust, matklad's "Large Rust Workspaces" and ARCHITECTURE.md: checked, nothing on impl placement.
  - Google Comprehensive Rust: only notes that impls *"can even be spread across multiple modules/files"*, with no advice.
  - Search-result SEO pages, DeepWiki and "rust-skills" summaries: secondary or AI-generated.
  - users.rust-lang.org 58794: about trait scoping, not placement.

## Gaps
- **Reddit threads were not read.** r/rust 12b805c and zzumcy were blocked (403 and JS-rendered), so community positions there are missing.
- **Not checked:** Bevy CONTRIBUTING, Tokio, rustc dev guide, Servo, Google Android/Chromium/Fuchsia guidance, Ferrous Systems, the Rust Design Patterns book, and smallvec (its `v2` path returned 404). Bevy and Tokio are unlikely to have such a rule, but I have not confirmed that.
- **No stated reason found for dedicated serde/rayon modules.** Next step: search the git history or PRs of indexmap, uuid and hashbrown.
- **The forum sample is small and opinion-based.** The "next to the type" consensus is a lean, not a documented standard.