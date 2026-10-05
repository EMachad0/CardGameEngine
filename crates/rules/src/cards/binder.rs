use std::{collections::BTreeMap, sync::Arc};

use crate::cards::{
    definition::{CardDef, CardDefKind, DefId, Effect},
    modifier::ModifierEffect,
};
use crate::choice::{CharacterChoice, ChoiceId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Binder {
    /// Arc so clone and eq are cheap
    defs: Arc<BTreeMap<DefId, CardDef>>,
}

impl Binder {
    pub(crate) fn new(defs: Vec<CardDef>) -> Self {
        let mut map = BTreeMap::new();
        defs.into_iter().for_each(|def| {
            map.insert(def.id, def);
        });

        Self {
            defs: Arc::new(map),
        }
    }

    pub(crate) fn get(&self, def_id: DefId) -> &CardDef {
        self.defs.get(&def_id).expect("card def not found")
    }

    pub(crate) fn health(&self, def_id: DefId) -> Option<i32> {
        match &self.get(def_id).kind {
            CardDefKind::Spell(_spell_card_def) => None,
            CardDefKind::Minion(minion_card_def) => Some(minion_card_def.health),
            CardDefKind::Hero(hero_card_def) => Some(hero_card_def.health),
        }
    }

    pub(crate) fn attack(&self, def_id: DefId) -> Option<i32> {
        match &self.get(def_id).kind {
            CardDefKind::Spell(_spell_card_def) => None,
            CardDefKind::Minion(minion_card_def) => Some(minion_card_def.attack),
            CardDefKind::Hero(_hero_card_def) => None,
        }
    }

    pub(crate) fn is_spell(&self, def_id: DefId) -> bool {
        matches!(self.get(def_id).kind, CardDefKind::Spell(_))
    }

    pub(crate) fn is_minion(&self, def_id: DefId) -> bool {
        matches!(self.get(def_id).kind, CardDefKind::Minion(_))
    }

    pub(crate) fn is_hero(&self, def_id: DefId) -> bool {
        matches!(self.get(def_id).kind, CardDefKind::Hero(_))
    }

    pub(crate) fn has_board_presence(&self, def_id: DefId) -> bool {
        match self.get(def_id).kind {
            CardDefKind::Spell(_) => false,
            CardDefKind::Minion(_) => true,
            CardDefKind::Hero(_) => false,
        }
    }

    pub(crate) fn has_deck_presence(&self, def_id: DefId) -> bool {
        match self.get(def_id).kind {
            CardDefKind::Minion(_) => true,
            CardDefKind::Spell(_) => true,
            CardDefKind::Hero(_) => false,
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
            CardDefKind::Spell(_spell_card_def) => &[],
            CardDefKind::Minion(minion_card_def) => &minion_card_def.friendly_aura_effects,
            CardDefKind::Hero(_hero_card_def) => &[],
        }
    }

    pub(crate) fn hostile_aura_effects(&self, def_id: DefId) -> &[ModifierEffect] {
        match &self.get(def_id).kind {
            CardDefKind::Spell(_spell_card_def) => &[],
            CardDefKind::Minion(minion_card_def) => &minion_card_def.hostile_aura_effects,
            CardDefKind::Hero(_hero_card_def) => &[],
        }
    }

    pub(crate) fn modifier_effects(&self, def_id: DefId) -> &[ModifierEffect] {
        &self.get(def_id).modifier_effects
    }

    pub(crate) fn choice(&self, def_id: DefId, choice_id: ChoiceId) -> Option<&CharacterChoice> {
        self.get(def_id).choices.iter().find(|c| c.id == choice_id)
    }

    pub(crate) fn choices(&self, def_id: DefId) -> &[CharacterChoice] {
        &self.get(def_id).choices
    }
}
