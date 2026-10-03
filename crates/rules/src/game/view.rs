//! What one player can see. `Game::view` is the one definition of it.

use crate::{
    DefId, Game, ObjectId, Outcome, PlayerId,
    game::{Player, PlayerInteractionState},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct View {
    pub viewer: PlayerId,
    pub active_player: PlayerId,
    pub players: Vec<PlayerView>,
    pub outcome: Option<Outcome>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerView {
    pub player_id: PlayerId,
    pub hero_health: i32,
    pub mana: u8,
    pub max_mana: u8,
    /// Oldest first.
    pub hand: Vec<HandCard>,
    pub deck_size: usize,
    /// Left to right.
    pub board: Vec<BoardCard>,
    /// A pending Forage's options, in reveal order. Empty if none is pending.
    pub revealed: Vec<RevealedCard>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HandCard {
    pub object_id: ObjectId,
    /// `None` if not currently visible.
    pub face: Option<Face>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Face {
    pub def_id: DefId,
    pub mana_cost: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RevealedCard {
    pub object_id: ObjectId,
    /// `None` unless the viewer is the one choosing.
    pub def_id: Option<DefId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCard {
    pub object_id: ObjectId,
    pub def_id: DefId,
    pub attack: i32,
    pub health: i32,
}

impl Game {
    pub fn view(&self, viewer: PlayerId) -> View {
        View {
            viewer,
            active_player: self.turn_order.get_active_player_id(),
            players: self
                .players
                .iter()
                .map(|p| self.player_view(viewer, p))
                .collect(),
            outcome: self.outcome,
        }
    }

    fn player_view(&self, viewer: PlayerId, player: &Player) -> PlayerView {
        PlayerView {
            player_id: player.id,
            hero_health: player.health,
            mana: player.mana,
            max_mana: player.max_mana,
            hand: player
                .zones
                .hand
                .as_slice()
                .iter()
                .map(|o| self.hand_card(viewer, player.id, *o))
                .collect(),
            deck_size: player.zones.deck.len(),
            board: player
                .zones
                .board
                .as_slice()
                .iter()
                .map(|o| self.board_card(viewer, *o))
                .collect(),
            revealed: self.revealed_view(viewer, player.id, &player.interaction_state),
        }
    }

    fn hand_card(&self, viewer: PlayerId, player_id: PlayerId, object_id: ObjectId) -> HandCard {
        let face = (player_id == viewer).then(|| Face {
            def_id: self.def_id(object_id),
            mana_cost: self
                .mana_cost(object_id)
                .expect("hand objects always have mana cost"),
        });

        HandCard { object_id, face }
    }

    fn board_card(&self, _viewer: PlayerId, object_id: ObjectId) -> BoardCard {
        BoardCard {
            object_id,
            def_id: self.def_id(object_id),
            attack: self
                .attack(object_id)
                .expect("board objects always have attack"),
            health: self
                .health(object_id)
                .expect("board objects always have health"),
        }
    }

    fn revealed_view(
        &self,
        viewer: PlayerId,
        player_id: PlayerId,
        interaction_state: &PlayerInteractionState,
    ) -> Vec<RevealedCard> {
        match interaction_state {
            PlayerInteractionState::Idle => Vec::new(),
            PlayerInteractionState::PendingPick { options } => options
                .iter()
                .map(|&object_id| RevealedCard {
                    object_id,
                    def_id: (player_id == viewer).then(|| self.def_id(object_id)),
                })
                .collect(),
        }
    }
}
