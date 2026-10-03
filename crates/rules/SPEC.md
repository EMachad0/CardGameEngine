# Session 03 game: spec

The tests for this spec live in `tests/spec/`. `docs/testing.md` says where each test goes.

## Layout

`src/lib.rs` declares the modules and re-exports the API below. `cards` is a public module; nothing else is.

| File | Holds |
|---|---|
| `ids.rs` | `PlayerId` |
| `action.rs` | `Action`, `IllegalAction` |
| `outcome.rs` | `Outcome` |
| `event.rs` | `Event`, `Target` |
| `observer.rs` | `Observer`, `Views` |
| `cards/definition.rs` | `DefId`, `CardDef`, `Effect` |
| `cards/loader.rs` | the printed card table, the `cards` constants and the table's validation |
| `cards/binder.rs` | printed values by `DefId` |
| `cards/object.rs` | `ObjectId`, `Object`, `ObjectBag` |
| `rng.rs` | `Rng` |
| `turn.rs` | seat order |
| `zones.rs` | the card containers |
| `game.rs` | `Game` with setup, turn start, `legal_actions` and the zone queries |
| `game/player.rs` | one player's record |
| `game/resolve.rs` | `apply`, `applied`, `ApplyError`, and what `apply` does once an action is legal |
| `game/lookup.rs` | `def_id`, `mana_cost`, `attack`, `health`, `hero_health` and `LookupError` |
| `game/view.rs` | `View` and its per-player and per-card parts, and `Game::view` |

Where a type lives inside the crate is yours to change.

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
- Active player, Forage pending: exactly one `Pick { object_id }` per revealed card. Nothing else.
- Active player, nothing pending: `Play { object_id }` for each card in their hand whose current cost is at most their mana, plus `EndTurn`.
- The other player: none.
- No duplicates in a list. Order doesn't matter.

## Contract (nodes L and P)

- `apply(p, a)` is `Ok(())` if and only if `a` is in `legal_actions(p)`.
- If `apply` returns `Err`, it's `Err(ApplyError::IllegalAction(IllegalAction { player_id: p, action: a }))`, and the game is unchanged.
- Everything that affects the future lives in `Game` (node S). The core never reads a clock, OS randomness, or a std `HashMap`'s iteration order (node R1). This crate's `clippy.toml` bans `HashMap`, `HashSet`, `Instant::now` and `SystemTime::now`, and the workspace lints make any use of them a clippy error.

## Events and views (node D)

`apply(p, a, obs)` reports to `obs` while it resolves: `obs.event(&event)` for each thing that happens, in order, and `obs.checkpoint(views)` at each checkpoint, where `views.of(v)` is `view(v)` at that moment.

- A rejected action makes no call to `obs`.
- Setup isn't observed. The shell reads `view(v)` after `Game::new`.
- The observer can't change the game: any observer leaves the same `Game` as `&mut ()`.
- The same game and action make the same calls in the same order (R1).
- Checkpoints come after a card leaves the hand to be played, after the action has resolved (all of a card's effects and a minion's entry, a pick with its bury, or a turn change with its draw), and after each death pass that removes a minion. The last call of every accepted `apply` is a checkpoint, so its views are the ones `view` returns once `apply` is done.
- Events between two checkpoints happened together. Blast's hits on every character share one step.

Events name objects by `ObjectId` only. A card's identity and current values reach a viewer through its `View`.

| When | Events, in order |
|---|---|
| `EndTurn` by `p` | `TurnEnded { p }`, then the next player's turn start |
| Turn start for `q` | `TurnStarted { q }`, then `Drew { q, card }`, or `FatigueDamaged { amount: 1, q }` if `q`'s deck is empty |
| Playing a card | `Played { p, card }`, then its effects. A minion then `BoardEntered { p, card }`. |
| A hit | `Damaged { target, amount, source: the card }` for each character hit, `amount` as printed |
| Forage | `Revealed { p, cards in reveal order }`. If nothing is revealed, no event. |
| A pick | `Picked { p, card }`, then `Buried { p, card }` for each other revealed card, in the order they go to the bottom |
| A death pass | `Died { minion }` for each minion it removes |
| The game ends | `GameEnded { outcome }`, once, in the `apply` that ends it |

`view(v)` is everything `v` may see, and nothing else (node D3):

- `viewer` is `v`, `active_player` is whose turn it is, `players[i]` is `PlayerId::new(i)`'s, and `outcome` is `outcome()`.
- Public, to every viewer: hero health, mana, max mana, deck size, the board in order with each minion's `DefId`, attack and health, and the `ObjectId` of every card in a hand or pending Forage.
- A hand card's `face` (its `DefId` and current cost) is `Some` only in its owner's view.
- A pending Forage's options show their `DefId` only to the player choosing.
- A deck's cards are never listed, only counted.

## API

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct PlayerId(/* private */ usize);
impl PlayerId {
    pub const fn new(idx: usize) -> PlayerId; // const, so callers can write `const P0: PlayerId = ...`
    pub fn idx(&self) -> usize;
}

/// Which printed card. Tests get one only from the `cards` constants and `Game::def_id`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DefId(/* private */);

/// One object in one game. Tests get one only from the zone queries.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
pub enum Action { Play { object_id: ObjectId }, Pick { object_id: ObjectId }, EndTurn }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IllegalAction { pub player_id: PlayerId, pub action: Action }

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApplyError { IllegalAction(IllegalAction) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Event {
    TurnStarted { player_id: PlayerId },
    TurnEnded { player_id: PlayerId },
    Drew { player_id: PlayerId, object_id: ObjectId },
    Played { player_id: PlayerId, object_id: ObjectId },
    Damaged { target: Target, amount: u8, source: ObjectId },
    Revealed { player_id: PlayerId, object_ids: Vec<ObjectId> },
    Picked { player_id: PlayerId, object_id: ObjectId },
    Buried { player_id: PlayerId, object_id: ObjectId },
    BoardEntered { player_id: PlayerId, object_id: ObjectId },
    FatigueDamaged { amount: u8, player_id: PlayerId },
    Died { object_id: ObjectId },
    GameEnded { outcome: Outcome },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target { Hero(PlayerId), Monster(ObjectId) }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct View { pub viewer: PlayerId, pub active_player: PlayerId, pub players: Vec<PlayerView>, pub outcome: Option<Outcome> }

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerView {
    pub player_id: PlayerId,
    pub hero_health: i32,
    pub mana: u8,
    pub max_mana: u8,
    pub hand: Vec<HandCard>,         // oldest first
    pub deck_size: usize,
    pub board: Vec<BoardCard>,       // left to right
    pub revealed: Vec<RevealedCard>, // pending Forage, in reveal order; empty if none
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HandCard { pub object_id: ObjectId, pub face: Option<Face> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Face { pub def_id: DefId, pub mana_cost: u8 }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RevealedCard { pub object_id: ObjectId, pub def_id: Option<DefId> }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BoardCard { pub object_id: ObjectId, pub def_id: DefId, pub attack: i32, pub health: i32 }

pub trait Observer {
    fn event(&mut self, event: &Event);
    fn checkpoint(&mut self, views: Views<'_>);
}
impl Observer for () {} // ignores both

/// What a checkpoint can read. Holds the game privately.
pub struct Views<'g>(/* private */);
impl Views<'_> {
    pub fn of(&self, viewer: PlayerId) -> View;
}

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum LookupError { ObjectNotFound(ObjectId) }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome { Won(PlayerId), Draw }

#[derive(Clone, Debug, PartialEq)]
pub struct Game { /* yours */ }

impl Game {
    pub fn new(seed: u64, decks: [Vec<DefId>; 2]) -> Game;             // shuffles
    pub fn with_deck_order(seed: u64, decks: [Vec<DefId>; 2]) -> Game; // index 0 = top
    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action>;
    pub fn apply(&mut self, player_id: PlayerId, action: Action, obs: &mut impl Observer) -> Result<(), ApplyError>;
    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Game, ApplyError>;
    pub fn outcome(&self) -> Option<Outcome>;                   // None while the game goes on
    pub fn hero_health(&self, player_id: PlayerId) -> i32;
    pub fn mana(&self, player_id: PlayerId) -> u8;
    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId];     // oldest first
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId>;   // top first
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId]; // pending Forage, in reveal order; empty if none
    pub fn board(&self, player_id: PlayerId) -> &[ObjectId];    // left to right
    pub fn def_id(&self, object_id: ObjectId) -> Result<DefId, LookupError>;
    pub fn mana_cost(&self, object_id: ObjectId) -> Result<Option<u8>, LookupError>; // Some iff in a hand
    pub fn attack(&self, object_id: ObjectId) -> Result<Option<i32>, LookupError>;   // Some iff on a board
    pub fn health(&self, object_id: ObjectId) -> Result<Option<i32>, LookupError>;   // Some iff on a board
    pub fn view(&self, viewer: PlayerId) -> View;
}
```

A lookup returns `Err(LookupError::ObjectNotFound(id))` only for an id the game never made. An object that has left every zone is still found. `def_id` returns its card, and `mana_cost`, `attack` and `health` return `Ok(None)`.

Whether an object keeps its `ObjectId` when it moves from hand to board is yours to decide. The tests accept either.

## Running

```sh
cargo test -p rules
cargo clippy -p rules --all-targets
```
