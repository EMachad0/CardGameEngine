//! Cost, attack and health: the printed values, changed by the object's modifiers and, for
//! health, its damage.

use crate::cards::object::ObjectId;
use crate::effect::EffectAmount;
use crate::game::amount::effect_amount;
use crate::game::cards::Cards;
use crate::game::modifiers::modifiers;
use crate::game::state::Game;
use crate::ids::PlayerId;
use crate::zones::{InHand, InPlay, OnBoard};

impl Game {
    /// `None` unless the card is in its owner's hand. Panics on an id the game never made.
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        mana_cost(Cards::new(self), object_id)
    }

    /// `None` unless the card is on its owner's board. Panics on an id the game never made.
    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        attack(Cards::new(self), object_id)
    }

    /// `None` unless the object is a character in play. Panics on an id the game never made.
    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        health(Cards::new(self), object_id)
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        hero_health(Cards::new(self), player_id)
    }
}

fn mana_cost(cards: Cards<'_>, object_id: ObjectId) -> Option<u8> {
    let owner = cards.objects.get(object_id).player_id;
    let card = cards.zones.find_in_hand(owner, object_id)?;
    Some(cost(cards, card))
}

fn attack(cards: Cards<'_>, object_id: ObjectId) -> Option<i32> {
    let owner = cards.objects.get(object_id).player_id;
    let card = cards.zones.find_on_board(owner, object_id)?;
    Some(minion_attack(cards, card))
}

fn health(cards: Cards<'_>, object_id: ObjectId) -> Option<i32> {
    cards.objects.get(object_id);
    let character = cards.zones.find_in_play(object_id)?;
    Some(character_health(cards, character))
}

fn hero_health(cards: Cards<'_>, player_id: PlayerId) -> i32 {
    character_health(cards, cards.zones.hero_in_play(player_id))
}

pub(super) fn cost(cards: Cards<'_>, card: InHand<'_>) -> u8 {
    let object_id = card.object_id();
    let printed_cost = cards.card_def(object_id).mana_cost;
    let reduction =
        modifiers(cards, object_id).cost_reduction(evaluate_for_owner(cards, object_id));
    printed_cost.saturating_sub(reduction)
}

pub(super) fn minion_attack(cards: Cards<'_>, card: OnBoard<'_>) -> i32 {
    let object_id = card.object_id();
    let printed_attack = cards
        .card_def(object_id)
        .attack()
        .expect("board objects always have attack");
    let bonus = modifiers(cards, object_id).attack_bonus(evaluate_for_owner(cards, object_id));
    printed_attack + bonus
}

pub(super) fn character_health(cards: Cards<'_>, character: InPlay<'_>) -> i32 {
    let object_id = character.object_id();
    let printed_health = cards
        .card_def(object_id)
        .health()
        .expect("in play objects always have health");
    let bonus = modifiers(cards, object_id).health_bonus(evaluate_for_owner(cards, object_id));
    printed_health + bonus - cards.objects.get(object_id).damage() as i32
}

fn evaluate_for_owner(cards: Cards<'_>, object_id: ObjectId) -> impl Fn(&EffectAmount) -> u8 {
    let owner = cards.objects.get(object_id).player_id;
    move |amount| effect_amount(cards, amount, owner)
}
