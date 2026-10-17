# A character's stats are decided in several places

Status: ready-for-agent
Category: enhancement
Type: task
Blocked by: 07
Branch: `stats-and-modifiers`, new, from `main` after ticket 07 merges. New draft PR with base
`main`.

## Why

How a modifier changes a stat is decided in four places, each with its own filter, and the data
the stats come from is open to every child of `game`.

- `modifiers` in `game/lookup.rs` gathers the printed `modifier_effects`, `Object.modifiers`, and,
  for an object that receives auras, the `friendly_aura` or `hostile_aura` of every other object on
  the board.
- `mana_cost`, `health`, `attack` and `triggers` in the same file each `filter_map` that list for
  their variant (`ReduceManaCost`, `BuffHealth`, `BuffAtk`, `GrantTrigger`) and drop the rest with
  `_ => None`. A new `ModifierEffect` variant passes all four silently.
- `Object.modifiers` and `Object.friendly_aura` are `pub` fields. `apply_effect` writes them, in
  its `Buff` and `AddFriendlyAura` arms.
- `Object.damage` is a `pub` field. `damage` in `game/zone_move.rs` writes it, and so does the
  fatigue branch of `draw`. `health` reads it.

From the doc's Rust section: "When several filters together decide what each variant means, give
that decision one exhaustive match." From the general section: "Decisions that change together
count as one." "Data tied by an invariant shares one owner."

## Facts to work with

- An earlier draft, PR #18 on `origin/modifiers-stats`, was closed before the ownership rules were
  in the doc. Reuse only what fits the doc as it stands.
- The glossary has terms for Buff, Aura, Cost, Health and Damage, but none for modifiers as a
  whole. Flag any name you need under "Names".

## Leave alone

- Every stat value, including `hostile_aura` holding only printed modifiers, and `mana_cost`
  returning `None` outside the owner's hand.
- Fatigue adding damage outside `Game::damage`. See the spec's known bugs.
- The `Option` returns of `health`, `attack` and `mana_cost`. See the spec's "Left for later
  sessions".

## Done when

- One exhaustive match decides what each modifier variant does to the stats.
- Only the boundary you build can add a modifier or an aura to an object, or damage it.
- The checks pass, and the PR justifies every behavior change.
