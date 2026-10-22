//! What one player can see. `view` is the one definition of it.

use crate::cards::definition::DefId;
use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;
use crate::game::player::Player;
use crate::game::player_input::PlayerInput;
use crate::game::resolve::aftermath::standings::Standings;
use crate::game::stats::{character_health, cost, hero_health, minion_attack};
use crate::ids::PlayerId;
use crate::outcome::Outcome;
use crate::zones::{InHand, OnBoard};

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
    /// The options of this player's picking, in reveal order. Empty unless they are picking.
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

pub(super) fn view(
    cards: CardsRef,
    players: &[Player],
    player_input: &PlayerInput,
    standings: &Standings,
    viewer: PlayerId,
) -> View {
    View {
        viewer,
        active_player: cards.turn_order.get_active_player_id(),
        players: players
            .iter()
            .map(|p| player_view(cards, player_input, viewer, p))
            .collect(),
        outcome: standings.outcome(),
    }
}

fn player_view(
    cards: CardsRef,
    player_input: &PlayerInput,
    viewer: PlayerId,
    player: &Player,
) -> PlayerView {
    PlayerView {
        player_id: player.id,
        hero: hero_card(cards, viewer, player.id),
        mana: player.mana.current(),
        max_mana: player.mana.max(),
        hand: cards
            .zones
            .hand_cards(player.id)
            .map(|card| hand_card(cards, viewer, player.id, card))
            .collect(),
        deck_size: cards.zones.deck_size(player.id),
        board: cards
            .zones
            .board_cards(player.id)
            .map(|card| board_card(cards, viewer, card))
            .collect(),
        revealed: revealed_view(cards, player_input, viewer, player.id),
        targeting: targeting_view(player_input, player.id),
    }
}

fn hand_card(cards: CardsRef, viewer: PlayerId, player_id: PlayerId, card: InHand) -> HandCard {
    let object_id = card.object_id();
    let face = (player_id == viewer).then(|| Face {
        def_id: cards.def_id(object_id),
        mana_cost: cost(cards, card),
    });

    HandCard { object_id, face }
}

fn board_card(cards: CardsRef, _viewer: PlayerId, card: OnBoard) -> BoardCard {
    let object_id = card.object_id();
    BoardCard {
        object_id,
        def_id: cards.def_id(object_id),
        attack: minion_attack(cards, card),
        health: character_health(cards, card.into()),
    }
}

fn revealed_view(
    cards: CardsRef,
    player_input: &PlayerInput,
    viewer: PlayerId,
    player_id: PlayerId,
) -> Vec<RevealedCard> {
    player_input
        .revealed(player_id)
        .iter()
        .map(|&object_id| RevealedCard {
            object_id,
            def_id: (player_id == viewer).then(|| cards.def_id(object_id)),
        })
        .collect()
}

fn hero_card(cards: CardsRef, _viewer: PlayerId, player_id: PlayerId) -> HeroCard {
    let object_id = cards.zones.hero(player_id);
    HeroCard {
        object_id,
        def_id: cards.def_id(object_id),
        health: hero_health(cards, player_id),
    }
}

fn targeting_view(player_input: &PlayerInput, player_id: PlayerId) -> Option<TargetingView> {
    player_input
        .targeting(player_id)
        .map(|targeting| TargetingView {
            object_id: targeting.object_id(),
            targeted: targeting.targeted_objects().collect(),
        })
}
