# Testing

## When to use

Read this before writing, moving or deleting any test, and before designing code that has to be
testable. Agents write and maintain the tests in this repo. In an exercise, the course's `COURSE.md`
decides who writes what.

## Two layers, kept apart

Unit tests live in the module they cover, in a `#[cfg(test)] mod tests` at the bottom of the file.
Their subject is that module's own type, and their setup builds only that type. They may use the
module's internals, such as `pub(crate)` items and private helpers. `rng.rs` tests `Rng` this way,
and `turn.rs` tests `TurnOrder`.

Crate tests live in `crates/<crate>/tests/` and use only the crate's public interface. A test
whose setup builds a `Game` is a crate test, even when it checks one rule that one `src/` file
implements. It never goes in that file, so moving code inside `src/` never moves a test.

Don't keep a `src/` file that holds only tests.

## One test crate per crate

Each crate has at most one integration test crate: `tests/<name>/main.rs`, one module per topic,
and a `support.rs`. Don't add another `tests/*.rs` or `tests/*/main.rs`. Cargo builds each one as
a separate binary with its own copy of the support module. Each binary then flags the helpers that
only the other one calls as dead code.

In `rules` it is `tests/spec/`, with one module per topic, such as `setup`, `cards` and
`state_check`. A test goes in the module for the topic it checks. A new topic gets a new module.

## Helpers

Helpers shared by crate tests live in `tests/<name>/support.rs`. A unit test's helper lives in its
own `mod tests`.

Never put a test helper module in `src/`, with or without `#[cfg(test)]`. A helper in `src/` can
reach internals, so the tests built on it stop testing the public interface. Without
`#[cfg(test)]` it compiles into the library, where every helper is dead code.

## Never widen visibility for a test

Don't make an item `pub` or `pub(crate)` so a test can reach it. If a crate test needs something
the public interface lacks, raise it with the learner, because the interface is theirs. If only a
unit test needs it, the test belongs in that module.

## Tests follow the code's interface

Before the learner writes an API, tests target the one agreed in the session and don't compile
until it exists. Once the code exists, it is the reference. When it differs from the tests in a
name, a signature or a derived trait, update the tests to match the code. When it differs in
behavior, ask the learner which one is right.

## Red tests

In an exercise, tests come before the implementation, so some fail on purpose. A red test compiles
and fails on a behavior the code doesn't have yet. Don't delete, weaken or `#[ignore]` a red test
without the learner's OK. A commit that leaves red tests lists them in its message.

No `#[test]` with an empty body, and no commented-out tests. Delete a test that can't be written
against the current code. Don't disable it.

## Reviewing an exercise

Green shows only what the tests pin. Before trusting it, probe each behavior agreed in the session
that no test pins yet, such as where checkpoints fall, with a throwaway recorder or test. A gap the
probe finds gets a test.

## Conventions

- Start every colocated `mod tests` with `use super::*;`.
- Take `assert_eq!` and `assert_ne!` from `pretty_assertions`, which prints a diff on a mismatch
  instead of both values whole. Import them in each test module that calls them, after
  `use super::*;` in a colocated `mod tests`. `crates/rules/clippy.toml` bans the std versions,
  so a missing import fails the checks.
- The diff covers only the two compared values. Don't assert more than the behavior needs to get
  one. Put the context in the message instead, and print an event stream there with `listing` from
  `tests/spec/support.rs`, one event per line.
- Name a test after its behavior, as a `snake_case` sentence:
  `forage_on_an_empty_deck_does_nothing`, not `test_forage`.
- One behavior per test. Several assertions about one behavior are fine. A name joined by "and"
  or "then" is usually two tests.
- Cover the happy and the unhappy path of each behavior, one test each:
  `giant_costs_one_less_per_spell_its_holder_has_cast` pairs with
  `opponents_spells_do_not_lower_a_giants_cost`.
- When the values alone don't show an assertion's intent, add a message that states the expected
  behavior:

  ```rust
  assert_eq!(
      game.health(recruit),
      Some(3),
      "the Captain's buff must hold the Recruit above Blast's damage",
  );
  ```

- Assert on what the interface returns, not on private state. A test should survive an internal
  refactor that keeps the behavior.
- Take inputs instead of creating them inside, the way `Game` takes its seed and its deck order,
  so a test controls every input. Return results instead of hiding side effects.

## Expected values and randomness

- Write expected values out by hand, in the test or in an independent model such as
  `tests/spec/model.rs`. Never read them from the crate's card data. A value the test reads from
  the code can't catch a wrong value in the code.
- Find a card by its identity (`def_id`), not by a property under test. A test that looks for the
  Bolt as "the card that costs 2" never finds a Bolt that costs 1.
- A seed fixes a stream of numbers, and one unrelated extra draw shifts every later outcome. Assert
  a property across many seeds instead of pinning one seed to one outcome.

## Running

The checks in `AGENTS.md` run every test. Clippy has to run with `--all-targets`. Without it,
clippy checks only the library and binaries, so no `#[cfg(test)]` module or `tests/` file gets
linted. An editor that runs clippy without the flag hides every test lint.

`cargo test -p rules --test spec state_check` runs the crate tests whose path contains
`state_check`.

## Benchmarks

Benchmarks live in `crates/<crate>/benches/`. Like crate tests, they use only the crate's public
interface. They can't import `tests/`, so the benches of a crate share their fixtures through
`benches/support.rs`, which each bench declares with `mod support;`. The crate sets
`autobenches = false` and lists each bench, so `support.rs` is not a bench of its own. Every bench
uses every fixture, or the other bench flags it as dead code.

Each measurement comes in two harnesses, under the same names:

- `benches/game.rs` uses criterion and measures wall-clock time. Shared CI runners swing it by
  10% and more, so it runs locally only.
- `benches/game_counts.rs` uses gungraun and counts instructions under Valgrind's Callgrind, in
  one run. The counts repeated exactly across CI's runners, so CI compares them.

A bench may pin a seed, because it measures and asserts nothing. A rules change that shifts the
RNG stream also changes the pinned game, so numbers from before and after such a change don't
compare directly.

`cargo bench -p rules --bench game` runs the criterion benches and reports the change since the
last run. `-- clone` after it runs the benches whose name contains `clone`, and `-- --test` runs
each one once as a smoke test. A plain `cargo bench` also runs `game_counts`, which fails without
gungraun-runner. The checks don't run benches, but clippy with `--all-targets` lints them.

Callgrind counts only inside the benchmark function, which runs once, so a gungraun bench needs
two things that criterion doesn't:

- Its setup makes the measured call once first. Otherwise the count includes the allocator's cold
  start.
- Its benchmark function defines no closure. A generic function instantiated with such a closure
  matches gungraun's collection toggle, and the count silently skips everything it runs. Pass a
  helper from `support.rs` instead.

`just bench::compare <ref>` counts `<ref>` and the working tree and prints a markdown table of
both. It needs Linux with Valgrind, gungraun-runner and jq. When the two sides lock different
gungraun versions, it counts only the working tree, since gungraun-runner must match the library.

`just bench::docker_compare <ref>` runs `compare` in a Linux x86_64 container. Valgrind on arm64
misses part of a count, so an Apple-silicon Mac counts under emulation. The first run builds the
image, which takes minutes. Under QEMU, the emulator Colima uses by default, a build now and then
crashes the linker or deadlocks cargo, and a rerun is the only fix. The container's glibc and
Valgrind differ from CI's, and under emulation two builds of the same code can differ slightly, so
compare container counts only with each other.

CI runs `just bench::compare` on every pull request that touches the code, against the base
branch, and posts the table as a PR comment, updated on each push. A regression doesn't fail CI.

## References

- `docs/course/README.md`: the courses, each with its exercise roles in its `COURSE.md`.
