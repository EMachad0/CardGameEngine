# Card text is validated, then assumed

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 11
Branch: `card-text`, new, from `main` after ticket 11 merges. New draft PR with base `main`.

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

- Which card texts are valid. Every validation result stays as it is.
- What every card does.
- The prelude's export of `CardDef`. See the spec's "Left for later sessions". A change to its
  shape changes the public API, so justify it under "Behavior".

## Done when

- Each fact a validator checks reaches the code that relies on it as a type, not as an `expect`.
- The checks pass, and the PR justifies every behavior change.
