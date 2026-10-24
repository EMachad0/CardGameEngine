# `pub` items the prelude doesn't list pass the checks when the public API reaches them

Status: needs-triage
Category: bug

`file-structure.md`, Rust "Paths", says: "With `unreachable_pub` on, a `pub` item the prelude
doesn't list fails the checks." The lint doesn't do that for an item the public API reaches. The
rustc lint listing says `unreachable_pub` flags items "neither directly accessible, nor reexported
(with `pub use`), nor leaked through things like return types (which the `unnameable_types` lint
can detect if desired)".

`lib.rs`'s docstring makes the same claim: "Every other module is private, so no other crate can
reach an item it leaves out."

## What happens

`CardDef` is in the prelude, and its `pub` fields reach the card text types. On main at `95dea0c`,
`cargo rustc -p rules --lib -- -W unnameable_types` reports 26 items that other crates can reach
but not name: `AmountBound`, `CardFilter`, `CardKind`, `CardKindDef`, `CharacterFilter`,
`CharacterKind`, `CharacterSelector`, `CharacterTarget`, `Effect`, `EffectAmount`, `HeroCardDef`,
`HistoryQuery`, `HistoryQueryKind`, `MinionCardDef`, `ModifierEffect`, `NoTarget`, `PlayerFilter`,
`PlayerSelector`, `PlayerSelectorFilter`, `Side`, `SpellCardDef`, `TriggerDef`, `TriggerPattern`,
`TriggerSubject`, `TurnFilter` and `ZoneMove`. PR #70 adds `Choice`, `TargetForm`, `Declared` and
`Triggered`, and drops `CharacterTarget`.

They can't be `pub(crate)` either. In a scratch crate, a `pub(crate)` type in a `pub` field of a
type the prelude lists raises `private_interfaces`, which `-D warnings` turns into a failure.

No crate outside `rules` names `CardDef`, and no public function takes or returns one. Finding 22
in the architecture course's open threads covers that.

## What should happen

The doc and the docstring say what the checks enforce. Two ways were found:

- Reword both. The sentence in "Paths" becomes: "With `unreachable_pub` on, a `pub` item fails
  the checks unless the prelude lists it or a public field, signature or bound reaches it. Other
  crates can use a reached item but can't name it." The docstring says "name" where it says
  "reach".
- Turn on `unnameable_types` in the workspace lints, so the sentence holds as written. The checks
  then fail on the 26 items until each is listed in the prelude or `CardDef` stops reaching them,
  which decides finding 22.
