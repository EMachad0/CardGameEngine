//! Cost, attack and health: the printed values, changed by the object's modifiers and, for
//! health, its damage.

use crate::{
    cards::object::ObjectId,
    effect::EffectAmount,
    game::Game,
    ids::PlayerId,
    zones::{InHand, InPlay, OnBoard},
};

impl Game {
    /// `None` unless the card is in its owner's hand. Panics on an id the game never made.
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        let owner = self.objects.get(object_id).player_id;
        let card = self.zones.find_in_hand(owner, object_id)?;
        Some(self.cost(card))
    }

    /// `None` unless the card is on its owner's board. Panics on an id the game never made.
    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        let owner = self.objects.get(object_id).player_id;
        let card = self.zones.find_on_board(owner, object_id)?;
        Some(self.minion_attack(card))
    }

    /// `None` unless the object is a character in play. Panics on an id the game never made.
    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        self.objects.get(object_id);
        let character = self.zones.find_in_play(object_id)?;
        Some(self.character_health(character))
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        self.character_health(self.zones.hero_in_play(player_id))
    }

    pub(super) fn cost(&self, card: InHand<'_>) -> u8 {
        let object_id = card.object_id();
        let printed_cost = self.card_def(object_id).mana_cost;
        let reduction = self
            .modifiers(object_id)
            .cost_reduction(self.evaluate_for_owner(object_id));
        printed_cost.saturating_sub(reduction)
    }

    pub(super) fn minion_attack(&self, card: OnBoard<'_>) -> i32 {
        let object_id = card.object_id();
        let printed_attack = self
            .card_def(object_id)
            .attack()
            .expect("board objects always have attack");
        let bonus = self
            .modifiers(object_id)
            .attack_bonus(self.evaluate_for_owner(object_id));
        printed_attack + bonus
    }

    pub(super) fn character_health(&self, character: InPlay<'_>) -> i32 {
        let object_id = character.object_id();
        let printed_health = self
            .card_def(object_id)
            .health()
            .expect("in play objects always have health");
        let bonus = self
            .modifiers(object_id)
            .health_bonus(self.evaluate_for_owner(object_id));
        printed_health + bonus - self.objects.get(object_id).damage() as i32
    }

    fn evaluate_for_owner(&self, object_id: ObjectId) -> impl Fn(&EffectAmount) -> u8 {
        let owner = self.objects.get(object_id).player_id;
        move |amount| self.effect_amount(amount, owner)
    }
}
