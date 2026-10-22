# Card text is validated, then assumed

Status: resolved
Category: enhancement
Type: task
Blocked by: 11
Branch: `card-text`, with draft PR #62 against `main`.

## Rework

PR #62 is this ticket's first pass. Rework it on its branch, rebased on `main`, instead of opening
a new PR. The learner's review:

- He likes that card text is generic over what a `Targeted` selector holds.
- The API got confusing:
  - Seven card text types gained defaulted type parameters, and `Effect` has two, with different
    meanings.
  - `ValidCardDef` repeats `CardDef`'s fields, and the parse rebuilds every effect, modifier and
    trigger to change one type.
  - The kind questions moved to `CardKindDef`, so every reader reaches them through `.kind()`.
- Several of the PR's choices were made only to keep `CardDef`'s public shape and every validation
  result as they are on `main`. Neither restriction holds any more. See "Leave alone".

Aim for the best code, not the smallest diff against `main`. Nothing outside "Leave alone" is off
limits, the public API included. Settle the shape yourself, within the doc, and state each choice in
the PR body as the spec asks. PR #62's Left out lists what the old restrictions kept it from doing,
so revisit each item. Rewrite the PR body for the new design, and start its Review section over,
with this review as its first entry.

PR #55's behavior probe and PR #62's validation probe are in comments on those PRs.

## Why

`Binder::built_in` in `cards/binder/store.rs` runs `validity::validate(&defs)`, which returns
`Result<(), CardDefError>`, and then stores the same `CardDef`s. `CardDef`'s fields are `pub`
(`cards/definition.rs`), so code anywhere in the crate can build one the validator would reject.
The code that reads card text relies on facts only the validator checked:

- `Binder::get` `expect`s "card def not found". `validate_not_found` checks that every built-in
  ID has a definition.
- `resolve_character_selector` in `game/resolve.rs` `expect`s "Target not fulfilled" for
  `CharacterSelector::Targeted`. It relies on `validate_target_on_trigger` and
  `validate_target_not_found`: a `Targeted` sits only in play effects and names a target the card
  declares. It also relies on targeting having stored an answer for every target.
- `CardDef::health` and `CardDef::attack` return `Option` by card kind. `Game::health` and
  `Game::attack` in `game/lookup.rs` `expect` them ("in play objects always have health", "board
  objects always have attack"), relying on only characters being in play and only minions on the
  board.

The validators also copy resolution's rules. `validate_target_on_trigger` knows a trigger
resolves with no targeting, and `validate_picks` knows `ResolvePick` reads what an earlier `Reveal`
stored.

The doc's rules: "A check validates when it returns nothing, or hands back the type it was given."
"When the type's shape rules out every wrong value, no code can build a wrong one, and nothing
needs trusting. Prefer shape wherever the guarantee fits a layout." "A step that checks what a
later step will do has to know that step's rule."

## Leave alone

- What every card does.

## Done when

- Each fact a validator checks reaches the code that relies on it as a type, not as an `expect`.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/62
