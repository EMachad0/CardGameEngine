//! Cost, attack, health and triggers: the printed values, changed by every modifier on the object
//! and every aura that reaches it.

use crate::{
    cards::{modifier::StatChanges, object::ObjectId},
    game::Game,
    ids::PlayerId,
    trigger::Trigger,
};

impl Game {
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        let owner = self.objects.get(object_id).player_id;
        self.zones.in_hand(owner, object_id).then(|| {
            let printed_cost = self.card_def(object_id).mana_cost;
            let reduction = self
                .stat_changes(object_id)
                .cost_reduction(|amount| self.effect_amount(amount, owner));
            printed_cost.saturating_sub(reduction)
        })
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        self.health(self.hero_id(player_id))
            .expect("in play heroes always have health")
    }

    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        let object = self.objects.get(object_id);
        let card_def = self.card_def(object_id);

        let is_in_play = self
            .zones
            .characters_in_play()
            .any(|character| character == object_id);

        is_in_play.then(|| {
            let printed_health = card_def
                .health()
                .expect("in play objects always have health");
            let bonus = self
                .stat_changes(object_id)
                .health_bonus(|amount| self.effect_amount(amount, object.player_id));
            printed_health + bonus - object.damage() as i32
        })
    }

    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        let owner = self.objects.get(object_id).player_id;
        self.zones.on_board(owner, object_id).then(|| {
            let printed_attack = self
                .card_def(object_id)
                .attack()
                .expect("board objects always have attack");
            let bonus = self
                .stat_changes(object_id)
                .attack_bonus(|amount| self.effect_amount(amount, owner));
            printed_attack + bonus
        })
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
