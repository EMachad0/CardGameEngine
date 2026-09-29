# Session 01 toy game: spec

The tests in `tests/contract.rs` check this spec. You write everything in `src/lib.rs` except `mod rng`.

## Rules

- Two players, `PlayerId(0)` and `PlayerId(1)`. Heroes start at 10 health.
- Setup:
  1. `Game::new` seeds the game's `Rng` with `seed`, shuffles player 0's deck, then player 1's deck.
     `Game::with_deck_order` seeds the `Rng` the same way but doesn't shuffle.
  2. Each player draws 3, player 0 first.
  3. Player 0's first turn starts.
- Turn start, for the active player: max mana +1 (capped at 10), mana refills to max, draw 1.
- Draw: take the top card of your deck and append it to the end of your hand.
  If the deck is empty, your hero takes 1 damage instead.
- Playing a card: pay its cost, remove it from your hand (other cards keep their order), resolve its effect.
  The card is then gone.
- Game over: as soon as a hero is at 0 health or less, the other player wins.
  Only one hero takes damage at a time, so there are no draws.

## Cards

| Card | Cost | Effect |
|---|---|---|
| `Bolt { damage }` | `damage` | Deal `damage` to the enemy hero. |
| `WildBolt` | 1 | Deal 3 damage to a random hero, possibly your own. Use the game's `Rng`. |
| `Forage` | 1 | Reveal the top 2 cards of your deck, or fewer if the deck is smaller. If nothing is revealed, nothing happens. Otherwise you must pick one: it goes to the end of your hand, and the rest go to the bottom of your deck in the order they were revealed. |

## Legal actions

- Game over: nobody has any.
- Active player, Forage pending: exactly one `Pick { index }` per revealed card. Nothing else.
- Active player, nothing pending: `Play { hand_index }` for each card whose cost is at most your current mana, plus `EndTurn`.
- The other player: none.
- No duplicates in a list. Order doesn't matter.

## Contract (nodes L and P)

- `apply(p, a)` is `Ok(())` if and only if `a` is in `legal_actions(p)`.
- If `apply` returns `Err`, it's `Err(Illegal { player: p, action: a })`, and the game is unchanged.
- Everything that affects the future lives in `Game` (node S). The core never reads a clock, OS randomness, or a std `HashMap`'s iteration order (node R1).

## API

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(pub u8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Card { Bolt { damage: u8 }, WildBolt, Forage }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action { Play { hand_index: usize }, Pick { index: usize }, EndTurn }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Illegal { pub player: PlayerId, pub action: Action }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game { /* yours */ }

impl Game {
    pub fn new(seed: u64, decks: [Vec<Card>; 2]) -> Game;             // shuffles
    pub fn with_deck_order(seed: u64, decks: [Vec<Card>; 2]) -> Game; // index 0 = top
    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action>;
    pub fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), Illegal>;
    pub fn applied(&self, player: PlayerId, action: Action) -> Result<Game, Illegal>;
    pub fn winner(&self) -> Option<PlayerId>;
    pub fn health(&self, p: PlayerId) -> i32;
    pub fn mana(&self, p: PlayerId) -> u8;
    pub fn hand(&self, p: PlayerId) -> &[Card];
    pub fn deck(&self, p: PlayerId) -> &[Card];     // top first
    pub fn revealed(&self, p: PlayerId) -> &[Card]; // pending Forage cards; empty if none
}
```

## Running

```sh
cargo test -p rules
```
