//! `Game`, the whole state of one game, its constructors, and what callers ask it.

use crate::action::Action;
use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::Binder;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::EffectStack;
use crate::game::legal_actions::LegalAction;
use crate::game::player::{Player, Players};
use crate::game::player_input::awaiting::PlayerInput;
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
    pub(super) players: Players,
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
        self.list_legal_actions(player_id)
            .iter()
            .map(Action::from)
            .collect()
    }

    /// `None` unless `legal_actions(player_id)` lists `action`.
    pub(super) fn legal_action(&self, player_id: PlayerId, action: Action) -> Option<LegalAction> {
        self.list_legal_actions(player_id)
            .into_iter()
            .find(|legal| Action::from(legal) == action)
    }

    pub fn view(&self, viewer: PlayerId) -> View {
        view::view(
            self.cards(),
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
        self.cards().def_id(object_id)
    }

    /// `None` unless the card is in its owner's hand. Panics on an id the game never made.
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        stats::mana_cost(self.cards(), object_id)
    }

    /// `None` unless the card is on its owner's board. Panics on an id the game never made.
    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        stats::attack(self.cards(), object_id)
    }

    /// `None` unless the object is a character in play. Panics on an id the game never made.
    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        stats::health(self.cards(), object_id)
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        stats::hero_health(self.cards(), player_id)
    }

    pub fn mana(&self, player_id: PlayerId) -> u8 {
        self.get_player(player_id).mana.current()
    }

    /// The options of the player's picking. Empty unless they are picking.
    pub fn revealed(&self, player_id: PlayerId) -> &[ObjectId] {
        self.player_input.revealed(player_id)
    }

    pub fn outcome(&self) -> Option<Outcome> {
        self.standings.outcome()
    }

    pub fn players(&self) -> Vec<PlayerId> {
        self.players.iter().map(Player::id).collect()
    }

    fn list_legal_actions(&self, player_id: PlayerId) -> Vec<LegalAction> {
        legal_actions::legal_actions(
            self.cards(),
            &self.standings,
            &self.player_input,
            self.get_player(player_id),
        )
    }

    fn get_player(&self, player_id: PlayerId) -> &Player {
        self.players.get(player_id)
    }

    pub(super) fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        self.players.get_mut(player_id)
    }

    pub(super) fn cards(&self) -> CardsRef<'_> {
        CardsRef::new(
            &self.binder,
            &self.objects,
            &self.zones,
            &self.history,
            &self.turn_order,
        )
    }
}
