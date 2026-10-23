# Card picks read their options in place and replace `Reveal`

Status: ready-for-human
Category: enhancement
Type: task
Blocked by: 03

## Why

A resolution-time pick is two effects joined by a `PickId`. `Reveal` takes cards off the deck into
no zone, so the deck looks smaller while the player picks. `ResolvePick` moves them later, and a
load check makes sure every `Reveal` has one. `StoredPicks`, in `game/player_input/picks.rs`,
keeps every answer by source and never clears it, so a second resolution from one source would
move the first one's cards. ADR 0020 replaces all of it.

## Forcing cards

- Forage, Sweep and Hoarder, ported.
- A new card whose filter is narrower than its window: "Look at the top 3 cards of your deck. Put
  a minion among them into your hand and bury the rest."

## What

From ADR 0019 and ADR 0020, limited to what the forcing cards reach:

- `CardSelector::Picked { choice: Choice<CardPickOptions>, others: Vec<CardVerb> }`, with
  `CardPickOptions { zone: CardPickZone, kind }`, `CardPickZone::YourDeck(DeckWindow)` and
  `DeckWindow::Top(n)`.
- `CardDestination::DeckBottom`. Moving a card there is a bury.
- The resolving effect holds `Option<Picking>`, an enum by subject. `Picking::Cards` holds the
  effect's choice, its verbs and its `others`, plus `options`, `filtered_out` and `picked`. Opening
  a pick moves the cursor past the effect. Opening a pick and finishing it are separate steps
  (ADRs 0015 and 0018), each followed by a checkpoint.
- One function answers what the resolving effect does next: run its next effect, finish a complete
  picking, wait on an open one, or end. The loop asks it.
- `Pick { object_id }` adds one object. The pick is complete once it holds the choice's count, or
  every option if there are fewer. Then the effect's verbs run on the objects picked, the verbs in
  `others` run on the options left and on `filtered_out`, in the order they were shown, and the
  picking is dropped.
- A pick with no options is skipped. The source's owner answers.
- The view: the chooser sees the options and `filtered_out` face up, and every other player sees
  how many options there are.
- Remove `Reveal`, `ResolvePick`, `PickId`, `ZoneMove`, `game/player_input/picks.rs` with the
  old `Picking`, `Picks` and `StoredPicks`, `PlayerInput`'s `pickings` and `stored_picks`, the
  pick check in `cards/binder/validity.rs` and `reveal` in `zones.rs`. Keep, rename or remove the
  `Revealed` and `Picked` events, and say why in the PR.
- Port Forage, Sweep and Hoarder to `Picked` over the top 2 with a count of 1, verbs
  `[Move(Hand)]` and `others` `[Move(DeckBottom)]`.
- In `docs/CONTEXT.md`, reword Picking, Player input and Resolving effect, and remove Reveal.

## Tests

- Forage offers its player the top 2 cards, puts the one picked into the hand and buries the
  other.
- A pick that is the last effect of its card stays open until it is answered.
- Sweep's draw runs after its pick is answered, not before.
- The deck keeps its size and its order while a pick is open.
- The new card offers only the minions among the top 3, and buries the non-minion cards it looked
  at along with the minion not taken.
- The cards in `others` reach the bottom of the deck in the order they were shown.
- A deck with fewer cards than the window offers the cards it has.
- With an empty deck the pick is skipped, and the card's later effects still run.
- While a pick is open, the chooser's only legal actions are `Pick` on the options, and every other
  player has none.
- The chooser's view shows the options and `filtered_out`. Every other player's view shows how
  many options there are and no object ids.
- Hoarder's death rattle picks, so trigger text can pick.
- If the trigger patterns can fire one source's trigger twice, two resolutions from that source
  each move their own picks. Otherwise, say so in the PR.

## Done when

The checks pass, nothing named `Reveal`, `ResolvePick` or `PickId` is left, and the agent's review
against `docs/conventions/file-structure.md` is answered.
