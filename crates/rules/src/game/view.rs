//! What one player can see. `view` is the one definition of it.

use crate::cards::binder::def_id::DefId;
use crate::cards::object::ObjectId;
use crate::game::cards_ref::CardsRef;
use crate::game::effect_stack::EffectStack;
use crate::game::picking::Picking;
use crate::game::player::{Player, Players};
use crate::game::player_input::awaiting::{Awaiting, PlayerInput, awaiting};
use crate::game::player_input::targeting::Targeting;
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
    /// `None` unless this player is picking.
    pub picking: Option<PickingView>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickingView {
    /// What the chooser sees.
    Chooser(ChooserPicking),
    /// What every other player sees.
    Other { option_count: usize },
}

/// What the chooser sees of a pick, by its subject.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChooserPicking {
    /// Every card the pick covers, in the order shown.
    Cards(Vec<PickingCard>),
    /// The options, in the order shown.
    Characters(Vec<ObjectId>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickingCard {
    pub object_id: ObjectId,
    pub def_id: DefId,
    /// Whether it is one of the options, rather than a card the pick's filter left out.
    pub pickable: bool,
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
    players: &Players,
    player_input: &PlayerInput,
    effect_stack: &EffectStack,
    standings: &Standings,
    viewer: PlayerId,
) -> View {
    View {
        viewer,
        active_player: cards.turn_order.get_active_player_id(),
        players: players
            .iter()
            .map(|p| player_view(cards, player_input, effect_stack, viewer, p))
            .collect(),
        outcome: standings.outcome(),
    }
}

fn player_view(
    cards: CardsRef,
    player_input: &PlayerInput,
    effect_stack: &EffectStack,
    viewer: PlayerId,
    player: &Player,
) -> PlayerView {
    let player_id = player.id();
    let (picking, targeting) = match awaiting(cards, player_input, effect_stack, player_id) {
        Awaiting::Nothing(_) => (None, None),
        Awaiting::Picking(awaits_picking) => (
            Some(picking_view(
                cards,
                awaits_picking.picking(),
                player_id,
                viewer,
            )),
            None,
        ),
        Awaiting::Targeting(awaits_targeting) => {
            (None, Some(targeting_view(awaits_targeting.targeting())))
        }
    };
    PlayerView {
        player_id,
        hero: hero_card(cards, viewer, player_id),
        mana: player.mana.current(),
        max_mana: player.mana.max(),
        hand: cards
            .zones
            .hand_cards(player_id)
            .map(|card| hand_card(cards, viewer, player_id, card))
            .collect(),
        deck_size: cards.zones.deck_size(player_id),
        board: cards
            .zones
            .board_cards(player_id)
            .map(|card| board_card(cards, viewer, card))
            .collect(),
        picking,
        targeting,
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

fn hero_card(cards: CardsRef, _viewer: PlayerId, player_id: PlayerId) -> HeroCard {
    let object_id = cards.zones.hero(player_id);
    HeroCard {
        object_id,
        def_id: cards.def_id(object_id),
        health: hero_health(cards, player_id),
    }
}

fn picking_view(
    cards: CardsRef,
    picking: &Picking,
    chooser: PlayerId,
    viewer: PlayerId,
) -> PickingView {
    if viewer != chooser {
        return PickingView::Other {
            option_count: picking.option_count(),
        };
    }
    PickingView::Chooser(match picking {
        Picking::Cards(card_picking) => ChooserPicking::Cards(
            card_picking
                .covered()
                .map(|(object_id, pickable)| PickingCard {
                    object_id,
                    def_id: cards.def_id(object_id),
                    pickable,
                })
                .collect(),
        ),
        Picking::Characters(character_picking) => {
            ChooserPicking::Characters(character_picking.options().collect())
        }
    })
}

fn targeting_view(targeting: &Targeting) -> TargetingView {
    TargetingView {
        object_id: targeting.object_id(),
        targeted: targeting.targeted_objects().collect(),
    }
}
