# Session 02 game: spec

Two kinds of tests check this spec:

- Rule tests cover setup, turns, cards, derived values and the state check. They sit in the `#[cfg(test)] mod tests` at the bottom of the `src/` file that implements the rule, and they use only the public API plus `testkit`.
- Contract tests cover legality, rejection, determinism, IDs and derived values over random playouts. They live in `tests/contract/` and use only the public API.

## Layout

`src/lib.rs` declares the modules and re-exports the API below. `cards` is a public module; nothing else is.

| File | Holds |
|---|---|
| `ids.rs` | `PlayerId`, `DefId`, `ObjectId` |
| `action.rs` | `Action`, `Illegal` |
| `cards.rs` | the printed card table and the `cards` constants |
| `rng.rs` | `Rng` |
| `turn.rs` | seat order |
| `zones.rs` | the card containers |
| `game.rs` | `Game` with setup, turn start, `legal_actions`, `apply`, the queries and `Outcome` |
| `game/player.rs` | one player's record |
| `game/resolve.rs` | what `apply` does once an action is legal: card effects, picks and end of turn |
| `game/derived.rs` | attack, health and cost, computed on read |
| `game/check.rs` | the state check at the end of every `apply` |
| `testkit.rs` | helpers for the in-file tests, built only under `cfg(test)` |

Where a type lives inside the crate is yours to change. Move its rule tests with it.

| Test file | Holds |
|---|---|
| `tests/contract/support.rs` | builders and assertions the other modules share |
| `tests/contract/model.rs` | an independent model of this spec: printed data, costs in hand, both boards |
| `tests/contract/playout.rs` | the random playout driver and the invariants it checks at every step |
| `tests/contract/legality.rs` | P on exact positions |
| `tests/contract/properties.rs` | R1, L, P, B and C over seeded playouts |

## Rules

- Two players, `PlayerId::new(0)` and `PlayerId::new(1)`. Heroes start at 10 health.
- A player's identity (`PlayerId`) is separate from turn order. Deck `i` belongs to `PlayerId::new(i)` no matter who goes first.
- Every card in the game is an object with an `ObjectId`. No two objects share one, copies of the same card included. An `ObjectId` that has left every zone is never used again.
- Setup:
  1. `Game::new` seeds the game's `Rng` with `seed`, shuffles player 0's deck, then player 1's deck.
     `Game::with_deck_order` seeds the `Rng` the same way but doesn't shuffle.
  2. Each player draws 3, player 0 first.
  3. Player 0's first turn starts.
- Turn start, for the active player: max mana +1 (capped at 10), mana refills to max, draw 1.
- Draw: take the top card of your deck and append it to the end of your hand.
  If the deck is empty, your hero takes 1 damage instead.
- Playing a card: pay its current cost, remove it from your hand (other cards keep their order), resolve it.
  A spell is gone once it resolves. A minion enters at the right end of its owner's board.
- Minions don't attack. They sit on the board until they die.
- A minion's attack and max health are its printed values plus every buff that applies right now. Its health is its max health minus the damage marked on it. Damage stays marked while the minion is on the board.
- The state check runs at the end of every `apply`, after the action and any turn start it causes:
  1. Find every minion, on either board, whose health is 0 or less. Remove them all at once. The others keep their order.
  2. Repeat until a pass finds nothing.
  3. If both heroes are at 0 or less, the game is a draw. If one is, the other player wins.

## Cards

| Constant | Cost | Kind | Text |
|---|---|---|---|
| `SPARK` | 1 | spell | Deal 1 damage to the enemy hero. |
| `BOLT` | 2 | spell | Deal 2 damage to the enemy hero. |
| `WILD_BOLT` | 1 | spell | Deal 3 damage to a random hero, possibly your own. Use the game's `Rng`. |
| `FORAGE` | 1 | spell | Reveal the top 2 cards of your deck, or fewer if the deck is smaller. If nothing is revealed, nothing happens. Otherwise you must pick one: it goes to the end of your hand, and the rest go to the bottom of your deck in the order they were revealed. |
| `BLAST` | 3 | spell | Deal 2 damage to every character: both heroes and every minion on both boards. |
| `RECRUIT` | 2 | minion, 2/2 | |
| `CAPTAIN` | 3 | minion, 1/1 | Other minions on your board have +1/+1. |
| `GIANT` | 8 | minion, 5/5 | Costs 1 less for each spell you've cast this game, but never less than 0. |

"The enemy hero" is the caster's enemy, not whoever's turn is next. "You" on a card in hand is the player holding it. "Spells you've cast this game" counts every spell that player has played since setup, including ones played before the Giant was drawn.

## Legal actions

- Game over: nobody has any.
- Active player, Forage pending: exactly one `Pick { card }` per revealed card. Nothing else.
- Active player, nothing pending: `Play { card }` for each card in their hand whose current cost is at most their mana, plus `EndTurn`.
- The other player: none.
- No duplicates in a list. Order doesn't matter.

## Contract (nodes L and P)

- `apply(p, a)` is `Ok(())` if and only if `a` is in `legal_actions(p)`.
- If `apply` returns `Err`, it's `Err(Illegal { player: p, action: a })`, and the game is unchanged.
- Everything that affects the future lives in `Game` (node S). The core never reads a clock, OS randomness, or a std `HashMap`'s iteration order (node R1). This crate's `clippy.toml` bans `HashMap`, `HashSet`, `Instant::now` and `SystemTime::now`, and the workspace lints make any use of them a clippy error.

## API

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(/* private */ usize);
impl PlayerId {
    pub const fn new(idx: usize) -> PlayerId; // const, so callers can write `const P0: PlayerId = ...`
    pub fn idx(&self) -> usize;
}

/// Which printed card. Tests get one only from the `cards` constants.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DefId(/* private */);

/// One object in one game. Tests get one only from the zone queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(/* private */);

pub mod cards {
    pub const SPARK: DefId;
    pub const BOLT: DefId;
    pub const WILD_BOLT: DefId;
    pub const FORAGE: DefId;
    pub const BLAST: DefId;
    pub const RECRUIT: DefId;
    pub const CAPTAIN: DefId;
    pub const GIANT: DefId;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action { Play { card: ObjectId }, Pick { card: ObjectId }, EndTurn }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Illegal { pub player: PlayerId, pub action: Action }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Won(PlayerId), Draw }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Game { /* yours */ }

impl Game {
    pub fn new(seed: u64, decks: [Vec<DefId>; 2]) -> Game;             // shuffles
    pub fn with_deck_order(seed: u64, decks: [Vec<DefId>; 2]) -> Game; // index 0 = top
    pub fn legal_actions(&self, player: PlayerId) -> Vec<Action>;
    pub fn apply(&mut self, player: PlayerId, action: Action) -> Result<(), Illegal>;
    pub fn applied(&self, player: PlayerId, action: Action) -> Result<Game, Illegal>;
    pub fn outcome(&self) -> Option<Outcome>;         // None while the game goes on
    pub fn hero_health(&self, p: PlayerId) -> i32;
    pub fn mana(&self, p: PlayerId) -> u8;
    pub fn hand(&self, p: PlayerId) -> Vec<ObjectId>;     // oldest first
    pub fn deck(&self, p: PlayerId) -> Vec<ObjectId>;     // top first
    pub fn revealed(&self, p: PlayerId) -> Vec<ObjectId>; // pending Forage, in reveal order; empty if none
    pub fn board(&self, p: PlayerId) -> Vec<ObjectId>;    // left to right
    pub fn def(&self, id: ObjectId) -> Option<DefId>;     // Some iff id is in a hand, deck, reveal or board
    pub fn cost(&self, id: ObjectId) -> Option<u8>;       // Some iff id is in a hand
    pub fn attack(&self, id: ObjectId) -> Option<i32>;    // Some iff id is on a board
    pub fn health(&self, id: ObjectId) -> Option<i32>;    // Some iff id is on a board
}
```

Whether an object keeps its `ObjectId` when it moves from hand to board is yours to decide. The tests accept either.

## Running

```sh
cargo test -p rules
cargo clippy -p rules --all-targets
```
