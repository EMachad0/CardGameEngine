//! Cost, attack and health: the printed values, changed by the object's modifiers and, for
//! health, its damage.

use crate::cards::object::ObjectId;
use crate::effect::EffectAmount;
use crate::game::amount::effect_amount;
use crate::game::cards_ref::CardsRef;
use crate::game::modifiers::modifiers;
use crate::ids::PlayerId;
use crate::zones::{InHand, InPlay, OnBoard};

pub(super) fn mana_cost(cards: CardsRef, object_id: ObjectId) -> Option<u8> {
    let owner = cards.objects.get(object_id).player_id;
    let card = cards.zones.find_in_hand(owner, object_id)?;
    Some(cost(cards, card))
}

pub(super) fn attack(cards: CardsRef, object_id: ObjectId) -> Option<i32> {
    let owner = cards.objects.get(object_id).player_id;
    let card = cards.zones.find_on_board(owner, object_id)?;
    Some(minion_attack(cards, card))
}

pub(super) fn health(cards: CardsRef, object_id: ObjectId) -> Option<i32> {
    cards.objects.get(object_id);
    let character = cards.zones.find_in_play(object_id)?;
    Some(character_health(cards, character))
}

pub(super) fn hero_health(cards: CardsRef, player_id: PlayerId) -> i32 {
    character_health(cards, cards.zones.hero_in_play(player_id))
}

pub(super) fn cost(cards: CardsRef, card: InHand) -> u8 {
    let object_id = card.object_id();
    let printed_cost = cards.card_def(object_id).mana_cost();
    let reduction =
        modifiers(cards, object_id).cost_reduction(evaluate_for_owner(cards, object_id));
    printed_cost.saturating_sub(reduction)
}

pub(super) fn minion_attack(cards: CardsRef, card: OnBoard) -> i32 {
    let object_id = card.object_id();
    let printed_attack = cards
        .card_def(object_id)
        .kind()
        .attack()
        .expect("a card on a board should be a minion");
    let bonus = modifiers(cards, object_id).attack_bonus(evaluate_for_owner(cards, object_id));
    printed_attack + bonus
}

pub(super) fn character_health(cards: CardsRef, character: InPlay) -> i32 {
    let object_id = character.object_id();
    let printed_health = cards
        .card_def(object_id)
        .kind()
        .health()
        .expect("a character in play should be a hero or a minion");
    let bonus = modifiers(cards, object_id).health_bonus(evaluate_for_owner(cards, object_id));
    printed_health + bonus - cards.objects.get(object_id).damage() as i32
}

fn evaluate_for_owner(
    cards: CardsRef<'_>,
    object_id: ObjectId,
) -> impl Fn(&EffectAmount) -> u8 + '_ {
    let owner = cards.objects.get(object_id).player_id;
    move |amount| effect_amount(cards, amount, owner)
}
