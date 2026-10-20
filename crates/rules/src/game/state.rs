//! `Game`, the whole state of one game, and the questions a shell asks of one of its owners.

use crate::cards::binder::store::Binder;
use crate::cards::object::{ObjectBag, ObjectId};
use crate::game::effect_stack::EffectStack;
use crate::game::player::Player;
use crate::game::player_input::PlayerInput;
use crate::game::resolve::aftermath::standings::Standings;
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

    pub(super) fn get_player(&self, player_id: PlayerId) -> &Player {
        &self.players[player_id.idx()]
    }

    pub(super) fn get_player_mut(&mut self, player_id: PlayerId) -> &mut Player {
        &mut self.players[player_id.idx()]
    }
}
