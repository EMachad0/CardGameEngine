# Card text takes a target form, and a character target is a choice

Status: ready-for-agent
Category: enhancement
Type: task

## Why

`Effect<T>` and `CharacterSelector<T>` take as `T` what a `Targeted` selector holds. Play text
holds `CharacterTarget`, trigger text holds `NoTarget`, and the effect stack holds `Vec<ObjectId>`
(ADR 0017). Ticket 03 adds a second subject with a target of its own. Both targets change together
at the push, so ADR 0019 names them through one trait.

`CharacterTarget { filter, count, bound, different }` in `target.rs` is the options plus the
count, bound and difference rule that every choice shares.

## What

- A trait `TargetForm` with an associated type `CharacterTarget`, and three marker types:
  `Declared` holds `Choice<CharacterFilter>`, `Triggered` holds `NoTarget`, and `Bound` holds
  `Vec<ObjectId>`.
- `Effect<S: TargetForm>` and `CharacterSelector<S: TargetForm>`, with
  `Targeted(S::CharacterTarget)`.
- `CharacterTarget` becomes `Choice<O> { options: O, count, bound, different }`, used as
  `Choice<CharacterFilter>`.
- The bounds that derives need sit on the associated type, since `#[derive(Clone)]` on `Effect<S>`
  bounds only `S`.

## Leave alone

Behavior, and every test's assertions. The verbs are ticket 02's.

## Tests

None new. The existing tests pin the behavior.

## Done when

- No effect or selector takes a slot type as its parameter.
- The checks pass, and tests changed only in how they build card data.
