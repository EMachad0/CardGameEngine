//! Cost, attack, health and triggers: the printed values, changed by the object's modifiers, the
//! auras that reach it and, for health, its damage.

use crate::{
    cards::{modifier::StatChanges, object::ObjectId},
    effect::EffectAmount,
    game::Game,
    ids::PlayerId,
    trigger::Trigger,
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
            .stat_changes(object_id)
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
            .stat_changes(object_id)
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
            .stat_changes(object_id)
            .health_bonus(self.evaluate_for_owner(object_id));
        printed_health + bonus - self.objects.get(object_id).damage() as i32
    }

    /// Evaluates an amount for the object's owner.
    fn evaluate_for_owner(&self, object_id: ObjectId) -> impl Fn(&EffectAmount) -> u8 {
        let owner = self.objects.get(object_id).player_id;
        move |amount| self.effect_amount(amount, owner)
    }

    pub(super) fn triggers(&self, object_id: ObjectId) -> Vec<Trigger> {
        let changes = self.stat_changes(object_id);
        let granted = changes.granted_triggers().map(|(granted_by, def)| Trigger {
            granted_by,
            def: def.clone(),
        });
        let printed = self.card_def(object_id).triggers.iter().map(|def| Trigger {
            granted_by: object_id,
            def: def.clone(),
        });
        let mut triggers = granted.chain(printed).collect::<Vec<_>>();
        triggers.sort_by_key(|t| self.zones.grant_order(t.granted_by));
        triggers
    }

    fn stat_changes(&self, object_id: ObjectId) -> StatChanges<'_> {
        let object = self.objects.get(object_id);
        let card_def = self.card_def(object_id);
        let mut changes = StatChanges::default();
        for effect in &card_def.modifier_effects {
            changes.add(object_id, effect);
        }
        for modifier in object.modifiers() {
            changes.add(modifier.granted_by, &modifier.effect);
        }

        if card_def.receives_auras() {
            for other_player_id in self.players().into_iter() {
                for other_object_id in self.player_board(other_player_id).iter().copied() {
                    if object_id == other_object_id {
                        continue;
                    }

                    if object.player_id == other_player_id {
                        self.add_friendly_aura(other_object_id, &mut changes);
                    } else {
                        self.add_hostile_aura(other_object_id, &mut changes);
                    }
                }
            }
        }

        changes
    }

    fn add_friendly_aura<'a>(&'a self, object_id: ObjectId, changes: &mut StatChanges<'a>) {
        for effect in self.card_def(object_id).friendly_aura_effects() {
            changes.add(object_id, effect);
        }
        for modifier in self.objects.get(object_id).friendly_aura() {
            changes.add(modifier.granted_by, &modifier.effect);
        }
    }

    fn add_hostile_aura<'a>(&'a self, object_id: ObjectId, changes: &mut StatChanges<'a>) {
        for effect in self.card_def(object_id).hostile_aura_effects() {
            changes.add(object_id, effect);
        }
    }
}
