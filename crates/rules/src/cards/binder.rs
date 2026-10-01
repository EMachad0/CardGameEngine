use std::collections::BTreeMap;

use crate::cards::{
    DefId, Effect,
    definition::{CardDef, CardDefKind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Binder {
    pub defs: BTreeMap<DefId, CardDef>,
}

impl Binder {
    pub fn new(defs: Vec<CardDef>) -> Self {
        let mut map = BTreeMap::new();
        defs.into_iter().for_each(|def| {
            map.insert(def.id, def);
        });

        Self { defs: map }
    }

    fn get(&self, def_id: DefId) -> &CardDef {
        self.defs.get(&def_id).expect("card def not found")
    }

    pub fn health(&self, def_id: DefId) -> Option<i32> {
        match &self.get(def_id).kind {
            CardDefKind::Monster(monster_card_def) => Some(monster_card_def.health),
            CardDefKind::Spell(_spell_card_def) => None,
        }
    }

    pub fn attack(&self, def_id: DefId) -> Option<i32> {
        match &self.get(def_id).kind {
            CardDefKind::Monster(monster_card_def) => Some(monster_card_def.atk),
            CardDefKind::Spell(_spell_card_def) => None,
        }
    }

    pub fn mana_cost(&self, def_id: DefId) -> u8 {
        self.get(def_id).mana_cost
    }

    pub fn on_play_effect(&self, def_id: DefId) -> &[Effect] {
        &self.get(def_id).on_play_effect.0
    }
}
