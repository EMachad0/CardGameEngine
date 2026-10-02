use std::collections::BTreeMap;

use crate::cards::{
    definition::{CardDef, CardDefKind, DefId, Effect},
    object::ModifierEffect,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binder {
    pub defs: BTreeMap<DefId, CardDef>,
}

impl Binder {
    pub(crate) fn new(defs: Vec<CardDef>) -> Self {
        let mut map = BTreeMap::new();
        defs.into_iter().for_each(|def| {
            map.insert(def.id, def);
        });

        Self { defs: map }
    }

    pub(crate) fn get(&self, def_id: DefId) -> &CardDef {
        self.defs.get(&def_id).expect("card def not found")
    }

    pub(crate) fn health(&self, def_id: DefId) -> Option<i32> {
        match &self.get(def_id).kind {
            CardDefKind::Monster(monster_card_def) => Some(monster_card_def.health),
            CardDefKind::Spell(_spell_card_def) => None,
        }
    }

    pub(crate) fn attack(&self, def_id: DefId) -> Option<i32> {
        match &self.get(def_id).kind {
            CardDefKind::Monster(monster_card_def) => Some(monster_card_def.attack),
            CardDefKind::Spell(_spell_card_def) => None,
        }
    }

    pub(crate) fn mana_cost(&self, def_id: DefId) -> u8 {
        self.get(def_id).mana_cost
    }

    pub(crate) fn on_play_effect(&self, def_id: DefId) -> &[Effect] {
        &self.get(def_id).on_play_effect.0
    }

    pub(crate) fn on_board_enter(&self, def_id: DefId) -> &[Effect] {
        &self.get(def_id).on_board_enter.0
    }

    pub(crate) fn on_board_leave(&self, def_id: DefId) -> &[Effect] {
        &self.get(def_id).on_board_leave.0
    }

    pub(crate) fn on_death(&self, def_id: DefId) -> &[Effect] {
        &self.get(def_id).on_death.0
    }

    pub(crate) fn friendly_aura_effects(&self, def_id: DefId) -> &[ModifierEffect] {
        match &self.get(def_id).kind {
            CardDefKind::Monster(monster_card_def) => &monster_card_def.friendly_aura_effects,
            CardDefKind::Spell(_spell_card_def) => &[],
        }
    }

    pub(crate) fn hostile_aura_effects(&self, def_id: DefId) -> &[ModifierEffect] {
        match &self.get(def_id).kind {
            CardDefKind::Monster(monster_card_def) => &monster_card_def.hostile_aura_effects,
            CardDefKind::Spell(_spell_card_def) => &[],
        }
    }
}
