//! What one player can see. `Game::view` is the one definition of it.

use crate::{
    cards::{definition::DefId, object::ObjectId},
    game::{Game, player::Player},
    ids::PlayerId,
    outcome::Outcome,
    zones::{InHand, OnBoard},
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
    pub hero: HeroCard,
    pub mana: u8,
    pub max_mana: u8,
    /// Oldest first.
    pub hand: Vec<HandCard>,
    pub deck_size: usize,
    /// Left to right.
    pub board: Vec<BoardCard>,
    /// A pending Forage's options, in reveal order. Empty if none is pending.
    pub revealed: Vec<RevealedCard>,
    /// The card in targeting and what it has targeted so far. `None` if not targeting.
    pub targeting: Option<TargetingView>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HeroCard {
    pub object_id: ObjectId,
    pub def_id: DefId,
    pub health: i32,
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
    /// `None` unless the viewer is the one picking.
    pub def_id: Option<DefId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoardCard {
    pub object_id: ObjectId,
    pub def_id: DefId,
    pub attack: i32,
    pub health: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetingView {
    pub object_id: ObjectId,
    pub targeted: Vec<ObjectId>,
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
            outcome: self.outcome(),
        }
    }

    fn player_view(&self, viewer: PlayerId, player: &Player) -> PlayerView {
        PlayerView {
            player_id: player.id,
            hero: self.hero_card(viewer, player.id),
            mana: player.mana.current(),
            max_mana: player.mana.max(),
            hand: self
                .zones
                .hand_cards(player.id)
                .map(|card| self.hand_card(viewer, player.id, card))
                .collect(),
            deck_size: self.zones.deck_size(player.id),
            board: self
                .zones
                .board_cards(player.id)
                .map(|card| self.board_card(viewer, card))
                .collect(),
            revealed: self.revealed_view(viewer, player.id),
            targeting: self.targeting_view(player.id),
        }
    }

    fn hand_card(&self, viewer: PlayerId, player_id: PlayerId, card: InHand<'_>) -> HandCard {
        let object_id = card.object_id();
        let face = (player_id == viewer).then(|| Face {
            def_id: self.def_id(object_id),
            mana_cost: self.cost(card),
        });

        HandCard { object_id, face }
    }

    fn board_card(&self, _viewer: PlayerId, card: OnBoard<'_>) -> BoardCard {
        let object_id = card.object_id();
        BoardCard {
            object_id,
            def_id: self.def_id(object_id),
            attack: self.minion_attack(card),
            health: self.character_health(card.into()),
        }
    }

    fn revealed_view(&self, viewer: PlayerId, player_id: PlayerId) -> Vec<RevealedCard> {
        self.player_input
            .revealed(player_id)
            .iter()
            .map(|&object_id| RevealedCard {
                object_id,
                def_id: (player_id == viewer).then(|| self.def_id(object_id)),
            })
            .collect()
    }

    fn hero_card(&self, _viewer: PlayerId, player_id: PlayerId) -> HeroCard {
        let object_id = self.hero_id(player_id);
        HeroCard {
            object_id,
            def_id: self.def_id(object_id),
            health: self.hero_health(player_id),
        }
    }

    fn targeting_view(&self, player_id: PlayerId) -> Option<TargetingView> {
        self.player_input
            .targeting(player_id)
            .map(|targeting| TargetingView {
                object_id: targeting.object_id(),
                targeted: targeting.targeted().iter().map(|(o, _)| *o).collect(),
            })
    }
}
