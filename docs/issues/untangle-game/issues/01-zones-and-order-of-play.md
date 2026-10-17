# Zones and order of play have an owner

Status: ready-for-agent
Category: enhancement
Type: task
Branch: `zones-and-order-of-play`, new, from `main`. New draft PR with base `main`.

## Why

Which zone holds each object, and the order objects entered play, are stored data with rules, and
nothing guards them.

- **Zones.** `Player.zones` is a `Zones` whose fields are `pub` (`zones.rs`). Eight functions in
  `game/zone_move.rs` write it: `draw`, `reveal` and `bury` on the deck, `draw`, `add_to_hand` and
  `play` on the hand, `summon` and `destroy` on the board, `entomb` on the graveyard. `setup` in
  `game/mod.rs` fills the decks. Any child of `game` can write any zone of any player, so a rule
  such as "an object is in one zone at a time" holds only because each writer keeps it.
- **Order of play.** `Object.order` is a `pub` stamp. `ObjectBag::insert` sets it, and
  `update_order` refreshes it in five moves: `draw`, `entomb`, `add_to_hand`, `bury` and
  `character_entered`. Three places sort by it: `characters_in_order_of_play` in `game/mod.rs`
  (trigger listeners), `remove_dead` in `game/state_check.rs` (the order of deaths), and `triggers`
  in `game/lookup.rs` (by the stamp of each trigger's `granted_by`). Listener order and death order
  are two questions that share an answer today.

ADR 0012 says "Order of play is the order objects entered the board, kept as a stamp set at each
entry." The docstring on `Object.order` says the stamp updates "when object move between zones",
and the code refreshes it on moves that don't enter the board.

The doc's rules: "An invariant is a decision about stored data that holds whenever code outside its
module can see it." "Data tied by an invariant shares one owner." "Handing out write access to the
data makes the invariant breakable from outside." "Give each question its own function, even when
two share an answer today."

## Leave alone

- Every order the rules depend on: listeners, deaths, triggers, hand and deck order. Where the code
  disagrees with ADR 0012, keep the code and record the disagreement under "Left out". Which one is
  right is a rules question for the card game course.
- Two sources for an object's owner. `Object.player_id` is set at `insert` and never changes, and
  the zone holding an object is a second source. `mana_cost` and `attack` look in `player_id`'s
  hand and board, while `view`, `characters_in_play`, `find_dead` and the aura sources in
  `modifiers` read the zones. They disagree only when a card reveals an opponent's deck:
  `ResolvePick` moves the card into the source owner's zones, `mana_cost` returns `None`, and the
  `expect`s in `legal_actions` and `view.rs: hand_card` would panic. No built-in card does this.
  Picking one source changes that case, which is a rules question. Record it under "Left out".

## Behavior to watch

- When each move stamps relative to its event. `draw` refreshes the stamp after reporting `Drew`,
  while `add_to_hand`, `bury`, `entomb` and `character_entered` refresh it before reporting.

## Out of scope

- Revealed cards. `reveal` takes them off the deck into the pending pick, and after `pick` they sit
  only in the source's `pick_targets` until `ResolvePick` moves them, in no zone. Ticket 02 owns
  them, with the pending pick and its answer.
- Who may remove the dead from the board: ticket 06.
- `Object.choice_targets` and `Object.pick_targets`: ticket 02. `Object.damage`,
  `Object.modifiers` and `Object.friendly_aura`: ticket 08.

## Done when

- Only the boundary you build can write a zone or the order stamp.
- Each question about order of play has its own function.
- Every order the rules depend on comes out as before.
- The checks pass, and the PR justifies every behavior change.
