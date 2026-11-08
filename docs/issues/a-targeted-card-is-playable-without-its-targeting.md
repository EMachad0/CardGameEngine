# A targeted card is playable without its targeting

Status: needs-triage
Category: enhancement

`WasPlayable::new` in `crates/rules/src/game/play/playability.rs` builds a `WasPlayable` with no
targeting for any card its player can pay for, including a card that declares targets.
`steps::prepare` expects such a card to come with its targeting and panics otherwise. Only the
`targets().next().is_none()` branch in `legal_actions.rs` keeps that value away from `prepare`.

So "a card that declares targets is played only from its targeting" sits in three modules as
informal knowledge. A second caller of `WasPlayable::new` compiles, then panics.
`docs/conventions/file-structure.md`, "Parse, don't validate": "To make forgetting that check fail
the build, the code that runs the steps for that input takes a value only the check can build."

Fix: `playability` holds the rule. `WasPlayable::new` refuses a card that declares targets,
`with_targeting` is the only way to build one for it, and `legal_actions` drops its own branch.
`new`'s shape for a targeted card is an interface call.
