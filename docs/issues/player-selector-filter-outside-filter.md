# The player selector's filter sits outside the module that holds the other filters

Status: needs-triage
Category: enhancement

`PlayerSelectorFilter` lives in `crates/rules/src/cards/player_selector.rs`. `cards::filter` holds
the other filters (`Side`, `CardFilter`, `CharacterFilter`) and `fulfill_side_filter`, which
`select_players` uses to apply it. A reader looking for a filter opens `filter` and doesn't find
it, so `filter` doesn't rule out what its sibling holds.

Move `PlayerSelectorFilter` and its `impl` into `cards/filter.rs`, then import it from there in
`player_selector.rs`, `game/selector.rs` and the prelude.
