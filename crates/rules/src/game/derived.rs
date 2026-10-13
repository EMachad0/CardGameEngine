//! Attack, health and cost, computed on read from history and printed data.

use crate::{
    cards::modifier::ModifierEffect,
    game::Game,
    ids::PlayerId,
    object::{Modifier, Modifiers, ObjectId},
};

impl Game {
    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        let obj = self.object(object_id);
        self.get_player(obj.player_id)
            .zones
            .hand
            .contains(&object_id)
            .then(|| {
                let def = self.card_def(object_id).mana_cost;
                let neg_modifiers = self
                    .modifiers(object_id)
                    .as_slice()
                    .iter()
                    .filter_map(|m| match m.effect {
                        ModifierEffect::ReduceManaCost { amount } => Some(amount),
                        _ => None,
                    })
                    .map(|a| self.effect_amount(&a, obj.player_id))
                    .sum::<u8>();
                def.saturating_sub(neg_modifiers)
            })
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        self.health(self.hero_id(player_id))
            .expect("in play heroes always have health")
    }

    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        let obj = self.object(object_id);

        let card_def = self.card_def(object_id);
        let is_in_play_hero = card_def.is_hero() && self.hero_id(obj.player_id) == object_id;
        let is_in_play_minion = card_def.is_minion()
            && self
                .get_player(obj.player_id)
                .zones
                .board
                .contains(&object_id);

        let is_in_play = is_in_play_hero || is_in_play_minion;

        is_in_play.then(|| {
            let def = card_def
                .health()
                .expect("in play objects always have health");
            let modifiers = self
                .modifiers(object_id)
                .as_slice()
                .iter()
                .filter_map(|m| match m.effect {
                    ModifierEffect::BuffHealth { amount } => Some(amount),
                    _ => None,
                })
                .map(|a| self.effect_amount(&a, obj.player_id) as i32)
                .sum::<i32>();
            let damage = obj.damage as i32;
            def + modifiers - damage
        })
    }

    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        let obj = self.object(object_id);
        self.get_player(obj.player_id)
            .zones
            .board
            .contains(&object_id)
            .then(|| {
                let def = self
                    .card_def(object_id)
                    .attack()
                    .expect("board objects always have attack");
                let modifiers = self
                    .modifiers(object_id)
                    .as_slice()
                    .iter()
                    .filter_map(|m| match m.effect {
                        ModifierEffect::BuffAtk { amount } => Some(amount),
                        _ => None,
                    })
                    .map(|a| self.effect_amount(&a, obj.player_id) as i32)
                    .sum::<i32>();
                def + modifiers
            })
    }

    pub(super) fn modifiers(&self, object_id: ObjectId) -> Modifiers {
        let obj = self.object(object_id);
        let modifiers = self
            .card_def(object_id)
            .modifier_effects
            .iter()
            .map(|e| Modifier {
                source: object_id,
                effect: e.clone(),
            })
            .collect::<Vec<_>>();
        let mut modifiers = Modifiers::new(modifiers);
        modifiers.extend(obj.modifiers.clone());

        if self.card_def(object_id).is_minion() {
            for other_player_id in self.players().into_iter() {
                for other_object_id in self.player_board(other_player_id).iter().copied() {
                    if object_id == other_object_id {
                        continue;
                    }

                    let other_modifiers = if obj.player_id == other_player_id {
                        self.friendly_aura(other_object_id)
                    } else {
                        self.hostile_aura(other_object_id)
                    };
                    modifiers.extend(other_modifiers);
                }
            }
        }

        modifiers
    }

    fn friendly_aura(&self, object_id: ObjectId) -> Modifiers {
        let obj = self.object(object_id);
        let modifiers = self
            .card_def(object_id)
            .friendly_aura_effects()
            .iter()
            .map(|e| Modifier {
                source: object_id,
                effect: e.clone(),
            })
            .collect::<Vec<_>>();
        let mut modifiers = Modifiers::new(modifiers);
        modifiers.extend(obj.friendly_aura.clone());
        modifiers
    }

    fn hostile_aura(&self, object_id: ObjectId) -> Modifiers {
        let modifiers = self
            .card_def(object_id)
            .hostile_aura_effects()
            .iter()
            .map(|e| Modifier {
                source: object_id,
                effect: e.clone(),
            })
            .collect::<Vec<_>>();
        Modifiers::new(modifiers)
    }
}
