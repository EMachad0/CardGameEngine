# A character's stats are decided in several places

Status: resolved
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
- `mana_cost`, `health` and `attack` return `Option`, with `None` when the object isn't in its
  owner's hand, in play, or on the board. Every caller in `src` already knows the zone and
  `expect`s the answer: `legal_actions` and `hand_card` in `game/view.rs` walk the hand,
  `find_dead` in `game/state_check.rs` and `board_card` walk the board, and `hero_health` asks
  about a hero. The three are also public, where an ID from outside the crate can name any
  object, and `tests/spec/` asserts their `None`.

From the doc's Rust section: "When several filters together decide what each variant means, give
that decision one exhaustive match." From the general section: "Decisions that change together
count as one." "Data tied by an invariant shares one owner." "A question that answers "maybe" to
callers who already know the answer repeats the check on every call. Take the parsed type instead,
and answer plainly."

## Facts to work with

- An earlier draft, PR #18 on `origin/modifiers-stats`, was closed before the ownership rules were
  in the doc. Reuse only what fits the doc as it stands.
- The glossary has terms for Buff, Aura, Cost, Health and Damage, but none for modifiers as a
  whole. Flag any name you need under "Names".
- Ticket 01's boundary decides which zone holds an object. It is on main to build on.

## Leave alone

- Every stat value, including `hostile_aura` holding only printed modifiers, and `mana_cost`
  having no answer for a card outside its owner's hand.
- Fatigue adding damage outside `Game::damage`. See the spec's known bugs.

## Out of scope

- `CardDef::health` and `CardDef::attack` returning `Option` by card kind: ticket 12.

## Done when

- One exhaustive match decides what each modifier variant does to the stats.
- Only the boundary you build can add a modifier or an aura to an object, or damage it.
- A caller that found an object in a zone reads its stats without the zone being checked again.
- The checks pass, and the PR justifies every behavior change.

## Answer

https://github.com/EMachad0/CardGameEngine/pull/48
