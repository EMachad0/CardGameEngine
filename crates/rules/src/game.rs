//! `Game`: the whole state, and the interface a shell drives it through.
//!
//! `legal_actions` is the one definition of legality. `apply` accepts exactly
//! what it lists, and an `Err` leaves the game unchanged. Child modules:
//! - `player`: one player's resources, zones and pending choice.
//! - `resolve`: the mutation half of `apply`.
//! - `lookup`: attack, health and cost, computed on read.

mod lookup;
mod player;
mod resolve;
mod state_check;
pub(crate) mod view;
mod zone_move;

use crate::action::Action;
use crate::cards::CardDefLoader;
use crate::cards::binder::Binder;
use crate::cards::object::ObjectBag;
use crate::game::player::{Player, PlayerInteractionState};
use crate::history::History;
use crate::ids::PlayerId;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Deck;
use crate::{DefId, Event, ObjectId, Observer, Outcome};

pub use resolve::ApplyError;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    rng: Rng,
    turn_order: TurnOrder,
    players: Vec<Player>,
    objects: ObjectBag,
    binder: Binder,
    history: History,
    outcome: Option<Outcome>,
}

impl Game {
    /// Seeds the `Rng`, shuffles both decks, deals 3 cards each and starts player 0's turn.
    pub fn new(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        Self::setup(seed, decks, true)
    }

    /// Like `new`, but keeps each deck in the given order, index 0 on top.
    pub fn with_deck_order(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        Self::setup(seed, decks, false)
    }

    fn setup(seed: u64, decks: [Vec<DefId>; 2], shuffle: bool) -> Self {
        let rng = Rng::new(seed);

        // may explode not sure how to handle yet
        let defs = CardDefLoader.load_and_validate().unwrap();
        let binder = Binder::new(defs);

        let mut objects = ObjectBag::default();

        let player_count = decks.len();
        let player_ids = (0..player_count).map(PlayerId::new).collect::<Vec<_>>();
        let mut players = player_ids
            .iter()
            .map(|player_id| Player::new(*player_id))
            .collect::<Vec<_>>();
        for (player, deck_defs) in players.iter_mut().zip(decks) {
            let deck = deck_defs
                .into_iter()
                .map(|def_id| objects.insert(def_id, player.id))
                .collect::<Vec<_>>();
            player.zones.deck = Deck::new(deck);
        }
        let turn_order = TurnOrder::new(player_ids.clone());
        let history = History::default();

        let mut game = Self {
            rng,
            turn_order,
            players,
            objects,
            binder,
            history,
            outcome: None,
        };

        for player in game.players.iter_mut() {
            if shuffle {
                player.zones.deck.shuffle(&mut game.rng);
            }
        }

        for player_id in player_ids {
            game.draw(player_id, 3, &mut ());
        }

        game.start_turn(&mut ());
        game
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        if self.outcome().is_some() {
            return Vec::new();
        }

        let mut actions = Vec::new();
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Idle => {
                if self.turn_order.get_active_player_id() != player_id {
                    return Vec::new();
                }

                let mana = self.mana(player_id);
                let hand = self.hand(player_id);
                for object_id in hand.iter().cloned() {
                    let mana_cost = self
                        .mana_cost(object_id)
                        .expect("objects in hand always have mana cost");
                    if mana_cost <= mana {
                        actions.push(Action::Play { object_id });
                    }
                }

                actions.push(Action::EndTurn);
            }
            PlayerInteractionState::PendingPick { options } => {
                actions.extend(
                    options
                        .iter()
                        .cloned()
                        .map(|object_id| Action::Pick { object_id }),
                );
            }
        }
        actions
    }

    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.get_player(player_id).zones.hand.as_slice()
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.get_player(player_id).zones.deck.to_vec()
    }

    pub fn board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.get_player(player_id).zones.board.as_slice()
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana
    }

    /// The cards a pending Forage revealed. Empty if nothing is pending.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        match &self.get_player(player_id).interaction_state {
            PlayerInteractionState::Idle => &[],
            PlayerInteractionState::PendingPick { options } => options,
        }
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.outcome
    }

    fn end_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        obs.event(&Event::TurnEnded {
            player_id: active_player_id,
        });
        self.turn_order.end_turn();
    }

    fn start_turn(&mut self, obs: &mut impl Observer) {
        let active_player_id = self.turn_order.get_active_player_id();
        obs.event(&Event::TurnStarted {
            player_id: active_player_id,
        });
        let player = self.get_player_mut(active_player_id);
        player.max_mana = (player.max_mana + 1).min(10);
        player.mana = player.max_mana;
        self.draw(active_player_id, 1, obs);
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }

    pub fn players(&self) -> Vec<PlayerId> {
        self.players.iter().map(|p| p.id).collect()
    }
}
