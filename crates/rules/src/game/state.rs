//! `Game`, the whole state of one game, its constructors, and what callers ask it.

use crate::action::Action;
use crate::cards::binder::store::Binder;
use crate::cards::definition::DefId;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::EffectStack;
use crate::game::legal_actions::LegalAction;
use crate::game::player::Player;
use crate::game::player_input::PlayerInput;
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::resolve::apply::ApplyError;
use crate::game::view::{self, View};
use crate::game::{legal_actions, setup, stats};
use crate::history::History;
use crate::ids::PlayerId;
use crate::outcome::Outcome;
use crate::rng::Rng;
use crate::turn::TurnOrder;
use crate::zones::Zones;

#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    pub(super) rng: Rng,
    pub(super) turn_order: TurnOrder,
    pub(super) players: Vec<Player>,
    pub(super) zones: Zones,
    pub(super) player_input: PlayerInput,
    pub(super) objects: ObjectBag,
    pub(super) binder: Binder,
    pub(super) history: History,
    pub(super) standings: Standings,
    pub(super) effect_stack: EffectStack,
}

impl Game {
    /// Seeds the `Rng`, shuffles both decks, deals 3 cards each and starts player 0's turn.
    pub fn new(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        setup::setup(seed, decks, true)
    }

    /// Like `new`, but keeps each deck in the given order, index 0 on top.
    pub fn with_deck_order(seed: u64, decks: [Vec<DefId>; 2]) -> Self {
        setup::setup(seed, decks, false)
    }

    pub fn applied(&self, player_id: PlayerId, action: Action) -> Result<Self, ApplyError> {
        let mut game = self.clone();
        game.apply(player_id, action, &mut ())?;
        Ok(game)
    }

    pub fn legal_actions(&self, player_id: PlayerId) -> Vec<Action> {
        self.parsed_legal_actions(player_id)
            .iter()
            .map(LegalAction::action)
            .collect()
    }

    /// `None` unless `legal_actions(player_id)` lists `action`.
    pub(super) fn legal_action(&self, player_id: PlayerId, action: Action) -> Option<LegalAction> {
        self.parsed_legal_actions(player_id)
            .into_iter()
            .find(|legal| legal.action() == action)
    }

    pub fn view(&self, viewer: PlayerId) -> View {
        view::view(
            CardsRef::new(self),
            &self.players,
            &self.player_input,
            &self.standings,
            viewer,
        )
    }

    pub fn hand(&self, player_id: PlayerId) -> &[ObjectId] {
        self.zones.hand(player_id)
    }

    /// A copy, top first.
    pub fn deck(&self, player_id: PlayerId) -> Vec<ObjectId> {
        self.zones.deck(player_id)
    }

    pub fn player_board(&self, player_id: PlayerId) -> &[ObjectId] {
        self.zones.board(player_id)
    }

    pub fn hero_id(&self, player_id: PlayerId) -> ObjectId {
        self.zones.hero(player_id)
    }

    pub fn def_id(&self, object_id: ObjectId) -> DefId {
        CardsRef::new(self).def_id(object_id)
    }

    /// `None` unless the card is in its owner's hand. Panics on an id the game never made.
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        stats::mana_cost(CardsRef::new(self), object_id)
    }

    /// `None` unless the card is on its owner's board. Panics on an id the game never made.
    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        stats::attack(CardsRef::new(self), object_id)
    }

    /// `None` unless the object is a character in play. Panics on an id the game never made.
    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        stats::health(CardsRef::new(self), object_id)
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        stats::hero_health(CardsRef::new(self), player_id)
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana.current()
    }

    /// The cards a pending Forage revealed. Empty if nothing is pending.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        self.player_input.revealed(player_id)
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.standings.outcome()
    }

    pub fn players(&self) -> Vec<PlayerId> {
        self.players.iter().map(|p| p.id).collect()
    }

    fn parsed_legal_actions(&self, player_id: PlayerId) -> Vec<LegalAction> {
        legal_actions::legal_actions(
            CardsRef::new(self),
            &self.standings,
            &self.player_input,
            self.get_player(player_id),
        )
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    pub(super) fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }
}
