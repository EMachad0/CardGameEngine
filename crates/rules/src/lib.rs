//! Rules core. Session 01 toy game: see ../SPEC.md.

mod rng;
use std::{collections::VecDeque, ops::Add};

pub use rng::Rng;

// Yours from here: PlayerId, Card, Action, Illegal, Game, and the methods in SPEC.md.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PlayerId(pub usize);

impl PlayerId {
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }

    pub fn idx(&self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnOrder {
    current_idx: usize,
    players: Vec<PlayerId>,
}

impl TurnOrder {
    pub fn new(player_count: usize) -> Self {
        let players = (0..player_count).map(PlayerId).collect();
        Self {
            current_idx: 0,
            players,
        }
    }

    pub fn get_current_player_id(&self) -> PlayerId {
        self.players[self.current_idx]
    }

    fn get_next_player_idx(&self) -> usize {
        (self.current_idx + 1) % self.players.len()
    }

    pub fn get_next_player_id(&self) -> PlayerId {
        self.players[self.get_next_player_idx()]
    }

    pub fn end_turn(&mut self) {
        self.current_idx = self.get_next_player_idx();
    }

    pub fn get_player_id_from_turn_index(&self, idx: usize) -> PlayerId {
        self.players[idx]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Card {
    Bolt { damage: u8 },
    WildBolt,
    Forage,
}

impl Card {
    pub fn mana_cost(&self) -> u8 {
        match self {
            Card::Bolt { damage } => *damage,
            Card::WildBolt => 1,
            Card::Forage => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deck(VecDeque<Card>);

impl Deck {
    pub fn new<I: Into<VecDeque<Card>>>(deck: I) -> Self {
        Self(deck.into())
    }

    pub fn as_slice(&self) -> Vec<Card> {
        self.0.iter().cloned().collect::<Vec<_>>()
    }

    pub fn as_mut_slice(&mut self) -> &mut [Card] {
        self.0.make_contiguous()
    }

    pub fn shuffle(&mut self, rng: &mut Rng) {
        rng.shuffle(self.as_mut_slice());
    }

    pub fn pop_front(&mut self) -> Option<Card> {
        self.0.pop_front()
    }

    pub fn push_back(&mut self, card: Card) {
        self.0.push_back(card);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hand(Vec<Card>);

impl Hand {
    pub fn new<I: Into<Vec<Card>>>(hand: I) -> Self {
        Self(hand.into())
    }

    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, card: Card) {
        self.0.push(card);
    }

    pub fn remove(&mut self, index: usize) -> Card {
        self.0.remove(index)
    }

    pub fn as_slice(&self) -> &[Card] {
        self.0.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Play { hand_index: usize },
    Pick { index: usize },
    EndTurn,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Illegal {
    pub player: PlayerId,
    pub action: Action,
}

impl Illegal {
    pub fn new(player: PlayerId, action: Action) -> Self {
        Self { player, action }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Player {
    id: PlayerId,
    mana: u8,
    max_mana: u8,
    health: i32,
    hand: Hand,
    deck: Deck,
    revealed_cards: Vec<Card>,
}

impl Player {
    pub fn new(id: PlayerId, deck: Deck) -> Self {
        Self {
            id,
            mana: 0,
            max_mana: 0,
            health: 10,
            hand: Hand::empty(),
            revealed_cards: Vec::new(),
            deck,
        }
    }

    pub fn draw(&mut self, count: u8) {
        for _ in 0..count {
            if let Some(card) = self.deck.pop_front() {
                self.hand.add(card);
            } else {
                self.health -= 1;
            };
        }
    }

    pub fn reveal(&mut self, count: u8) {
        for _ in 0..count {
            let Some(card) = self.deck.pop_front() else {
                break;
            };

            self.revealed_cards.push(card);
        }
    }

    pub fn pick_revealed(&mut self, index: usize) -> (Card, Vec<Card>) {
        let mut cards = self.revealed_cards.drain(..).collect::<Vec<_>>();
        let picked = cards.remove(index);
        (picked, cards)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    rng: rng::Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
}

impl Game {
    pub fn new(seed: u64, decks: [Vec<Card>; 2]) -> Self {
        Self::setup(seed, decks, true)
    }

    pub fn with_deck_order(seed: u64, decks: [Vec<Card>; 2]) -> Self {
        Self::setup(seed, decks, false)
    }

    fn setup(seed: u64, decks: [Vec<Card>; 2], shuffle: bool) -> Self {
        let mut rng = Rng::new(seed);
        let turn_order = TurnOrder::new(decks.len());

        let mut players = decks
            .into_iter()
            .enumerate()
            .map(|(i, deck)| {
                let player_id = turn_order.get_player_id_from_turn_index(i);
                let player_deck = Deck::new(deck);
                Player::new(player_id, player_deck)
            })
            .collect::<Vec<_>>();

        for player in players.iter_mut() {
            if shuffle {
                rng.shuffle(player.deck.as_mut_slice());
            }
            player.draw(3);
        }

        let mut game = Self {
            rng,
            players,
            turn_order,
        };
        game.start_turn();
        game
    }

    pub fn apply(&mut self, player_id: PlayerId, action: Action) -> Result<(), Illegal> {
        if !self.legal_actions(player_id).contains(&action) {
            Err(Illegal::new(player_id, action))
        } else {
            match action {
                Action::Play { hand_index } => {
                    let card = self.get_player_mut(player_id).hand.remove(hand_index);
                    self.apply_card(player_id, card);
                }
                Action::Pick { index } => {
                    let player = self.get_player_mut(player_id);
                    let (picked, other_cards) = player.pick_revealed(index);
                    player.hand.add(picked);
                    other_cards
                        .into_iter()
                        .for_each(|c| player.deck.push_back(c));
                }
                Action::EndTurn => {
                    self.turn_order.end_turn();
                    self.start_turn();
                }
            }
            Ok(())
        }
    }

    fn start_turn(&mut self) {
        let current_player_id = self.turn_order.get_current_player_id();
        let player = self.get_player_mut(current_player_id);
        player.max_mana = player.max_mana.add(1).min(10);
        player.mana = player.max_mana;
        player.draw(1);
    }

    pub fn apply_card(&mut self, player_id: PlayerId, card: Card) {
        match card {
            Card::Bolt { damage } => {
                let player = self.get_player_mut(player_id);
                player.mana -= card.mana_cost();

                let target_player_id = self.turn_order.get_next_player_id();
                let target_player = self.get_player_mut(target_player_id);
                target_player.health -= damage as i32;
            }
            Card::WildBolt => {}
            Card::Forage => {
                let player = self.get_player_mut(player_id);
                player.reveal(2);
                player.mana -= card.mana_cost();
            }
        }
    }

    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, Illegal> {
        let mut game = self.clone();
        game.apply(player_id, action)?;
        Ok(game)
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        let mut actions = Vec::new();
        if self.turn_order.get_current_player_id() != player_id {
            return actions;
        }
        if self.winner().is_some() {
            return actions;
        }

        let revealed_cards = self.revealed(player_id);
        if !revealed_cards.is_empty() {
            actions.extend((0..revealed_cards.len()).map(|i| Action::Pick { index: i }));
            return actions;
        }

        let mana = self.mana(player_id);
        let hand = self.hand(player_id);
        for (i, card) in hand.iter().enumerate() {
            if card.mana_cost() <= mana {
                actions.push(Action::Play { hand_index: i });
            }
        }

        actions.push(Action::EndTurn);
        actions
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }

    pub fn hand(&self, player_id: PlayerId) -> &[Card] {
        self.get_player(player_id).hand.as_slice()
    }

    pub fn deck(&self, player_id: PlayerId) -> Vec<Card> {
        self.get_player(player_id).deck.as_slice()
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    pub fn health(&self, player_id: PlayerId) -> i32 {
        self.get_player(player_id).health
    }

    pub fn revealed(&self, player_id: PlayerId) -> &[Card] {
        &self.get_player(player_id).revealed_cards
    }

    pub fn winner(&self) -> Option<PlayerId> {
        let mut iter = self.players.iter().filter(|s| s.health > 0);
        if let Some(first) = iter.next()
            && iter.next().is_none()
        {
            Some(first.id)
        } else {
            None
        }
    }
}
