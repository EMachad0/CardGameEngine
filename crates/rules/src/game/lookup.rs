//! Attack, health and cost, computed on read from history and printed data.

use crate::{
    cards::{
        definition::{CardDef, DefId},
        filter::CharacterKindFilter,
        modifier::{Modifier, ModifierEffect, Modifiers},
        object::{Object, ObjectId},
    },
    effect::Effect,
    game::Game,
    ids::PlayerId,
    trigger::{Trigger, TriggerPattern, TriggerSubject},
};

impl Game {
    fn object(&self, object_id: ObjectId) -> &Object {
        self.objects.get(object_id)
    }

    pub fn def_id(&self, object_id: ObjectId) -> DefId {
        self.object(object_id).def_id
    }

    pub(super) fn card_def(&self, object_id: ObjectId) -> &CardDef {
        self.binder.get(self.def_id(object_id))
    }

    pub fn hero_id(&self, player_id: PlayerId) -> ObjectId {
        self.get_player(player_id).zones.hero
    }

    pub fn mana_cost(&self, object_id: ObjectId) -> Option<u8> {
        let obj = self.object(object_id);
        self.get_player(obj.player_id)
            .zones
            .hand
            .contains(&object_id)
            .then(|| {
                let printed_cost = self.card_def(object_id).mana_cost;
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
                printed_cost.saturating_sub(neg_modifiers)
            })
    }

    pub fn hero_health(&self, player_id: PlayerId) -> i32 {
        self.health(self.hero_id(player_id))
            .expect("in play heroes always have health")
    }

    pub fn health(&self, object_id: ObjectId) -> Option<i32> {
        let obj = self.object(object_id);
        let card_def = self.card_def(object_id);

        let is_in_play = match card_def.character_kind() {
            Some(CharacterKindFilter::Heroes) => self.hero_id(obj.player_id) == object_id,
            Some(CharacterKindFilter::Minions) => self
                .get_player(obj.player_id)
                .zones
                .board
                .contains(&object_id),
            None => false,
        };

        is_in_play.then(|| {
            let printed_health = card_def
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
            printed_health + modifiers - damage
        })
    }

    pub fn attack(&self, object_id: ObjectId) -> Option<i32> {
        let obj = self.object(object_id);
        self.get_player(obj.player_id)
            .zones
            .board
            .contains(&object_id)
            .then(|| {
                let printed_attack = self
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
                printed_attack + modifiers
            })
    }

    pub(crate) fn death_rattles(&self, object_id: ObjectId) -> Vec<Trigger> {
        self.triggers(object_id)
            .into_iter()
            .filter(|t| matches!(t.pattern, TriggerPattern::Died(TriggerSubject::Itself)))
            .collect()
    }

    pub(crate) fn triggers(&self, object_id: ObjectId) -> Vec<Trigger> {
        let def = self.card_def(object_id).triggers.iter().map(|td| Trigger {
            source: object_id,
            pattern: td.pattern,
            effects: td.effects.clone(),
        });
        let mut modifiers = self
            .modifiers(object_id)
            .as_slice()
            .iter()
            .filter_map(|m| match &m.effect {
                ModifierEffect::GrantTrigger(trigger) => Some(Trigger {
                    source: m.source,
                    pattern: trigger.pattern,
                    effects: trigger.effects.clone(),
                }),
                _ => None,
            })
            .collect::<Vec<_>>();
        modifiers.extend(def);
        modifiers.sort_by_key(|t| self.objects.get(t.source).order);
        modifiers
    }

    fn modifiers(&self, object_id: ObjectId) -> Modifiers {
        let obj = self.object(object_id);
        let card_def = self.card_def(object_id);
        let modifiers = card_def
            .modifier_effects
            .iter()
            .map(|e| Modifier {
                source: object_id,
                effect: e.clone(),
            })
            .collect::<Vec<_>>();
        let mut modifiers = Modifiers::new(modifiers);
        modifiers.extend(obj.modifiers.clone());

        if card_def.receives_auras() {
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

    pub(crate) fn play_effects(&self, object_id: ObjectId) -> &[Effect] {
        self.card_def(object_id).play_effects.as_slice()
    }

    pub(crate) fn friendly_aura(&self, object_id: ObjectId) -> Modifiers {
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

    pub(crate) fn hostile_aura(&self, object_id: ObjectId) -> Modifiers {
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

    pub(crate) fn playing(&self, player_id: PlayerId) -> bool {
        self.get_player(player_id).playing
    }
}
