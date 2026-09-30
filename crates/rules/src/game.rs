//! `Game`: the whole state, and the interface a shell drives it through.
//!
//! `legal_actions` is the one definition of legality. `apply` accepts exactly
//! what it lists, and an `Err` leaves the game unchanged. Child modules hold
//! the procedures behind that interface:
//! - `player`: one player's record (resources, zones, pending choice).
//! - `resolve`: the mutation half of `apply`.

mod player;
mod resolve;

use crate::action::{Action, Illegal};
use crate::cards::Card;
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Deck;
use player::{Player, PlayerInteractionState};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
}

impl Game {
    // -- Construction ------------------------------------------------------

    /// Seeds the game's `Rng`, shuffles each deck, deals, starts player 0's turn.
    pub fn new(seed: u64, decks: [Vec<Card>; 2]) -> Self {
        Self::setup(seed, decks, true)
    }

    /// Like `new`, but keeps each deck in the given order (index 0 = top).
    pub fn with_deck_order(seed: u64, decks: [Vec<Card>; 2]) -> Self {
        Self::setup(seed, decks, false)
    }

    fn setup(seed: u64, decks: [Vec<Card>; 2], shuffle: bool) -> Self {
        let mut rng = Rng::new(seed);
        let turn_order = TurnOrder::new(decks.len());

        let mut players = decks
            .into_iter()
            .enumerate()
            .map(|(i, deck)| Player::new(PlayerId::new(i), Deck::new(deck)))
            .collect::<Vec<_>>();

        for player in players.iter_mut() {
            if shuffle {
                player.deck.shuffle(&mut rng);
            }
            player.draw(3);
        }

        let mut game = Self {
            rng,
            turn_order,
            players,
        };
        game.start_turn();
        game
    }

    // -- Decisions ---------------------------------------------------------

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.winner().is_some() {
            return Vec::new();
        }
        if self.turn_order.get_current_player_id() != player_id {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Board => {
                let mana = self.mana(player_id);
                let hand = self.hand(player_id);
                for (i, card) in hand.iter().enumerate() {
                    if card.mana_cost() <= mana {
                        actions.push(Action::Play { hand_index: i });
                    }
                }

                actions.push(Action::EndTurn);
            }
            PlayerInteractionState::Picker { options } => {
                actions.extend((0..options.len()).map(|i| Action::Pick { index: i }));
            }
        }
        actions
    }

    /// `Ok` if and only if `action` is in `legal_actions(player_id)`.
    pub fn apply(&mut self, player_id: PlayerId, action: Action) -> Result<(), Illegal> {
        if !self.legal_actions(player_id).contains(&action) {
            Err(Illegal::new(player_id, action))
        } else {
            self.apply_action(player_id, action);
            Ok(())
        }
    }

    /// `apply` on a copy; `self` is untouched either way.
    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, Illegal> {
        let mut game = self.clone();
        game.apply(player_id, action)?;
        Ok(game)
    }

    // -- Queries -----------------------------------------------------------

    pub fn hand(&self, player_id: PlayerId) -> &[Card] {
        self.get_player(player_id).hand.as_slice()
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<Card> {
        self.get_player(player_id).deck.to_vec()
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    pub fn health(&self, player_id: PlayerId) -> i32 {
        self.get_player(player_id).health
    }

    /// Pending Forage cards; empty if none.
    pub fn revealed(&self, player_id: PlayerId) -> &[Card] {
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Board => &[],
            PlayerInteractionState::Picker { options } => options,
        }
    }

    pub fn winner(&self) -> Option<PlayerId> {
        let mut alive = self.players.iter().filter(|s| s.health > 0);
        match (alive.next(), alive.next()) {
            (Some(player), None) => Some(player.id),
            _ => None,
        }
    }

    // -- Internals shared with child modules --------------------------------

    fn start_turn(&mut self) {
        let current_player_id = self.turn_order.get_current_player_id();
        let player = self.get_player_mut(current_player_id);
        player.max_mana = (player.max_mana + 1).min(10);
        player.mana = player.max_mana;
        player.draw(1);
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }
}
