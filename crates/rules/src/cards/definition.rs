use crate::cards::{
    choice::{CharacterChoice, ChoiceId},
    effect::EffectSequence,
    modifier::ModifierEffect,
    trigger::TriggerDef,
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct DefId(&'static str);

impl DefId {
    pub(super) const fn new(id: &'static str) -> Self {
        Self(id)
    }
}

pub(super) const PLACEHOLDER: DefId = DefId::new("placeholder");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef {
    pub id: DefId,
    pub name: String,
    pub kind: CardDefKind,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub play_effects: EffectSequence,
    pub triggers: Vec<TriggerDef>,
    pub choices: Vec<CharacterChoice>,
}

impl Default for CardDef {
    fn default() -> Self {
        Self {
            id: PLACEHOLDER,
            name: Default::default(),
            kind: CardDefKind::Minion(MinionCardDef::new(1, 1)),
            mana_cost: Default::default(),
            modifier_effects: Default::default(),
            play_effects: Default::default(),
            triggers: Default::default(),
            choices: Default::default(),
        }
    }
}

impl CardDef {
    pub(crate) fn health(&self) -> Option<i32> {
        match &self.kind {
            CardDefKind::Spell(_) => None,
            CardDefKind::Minion(minion) => Some(minion.health),
            CardDefKind::Hero(hero) => Some(hero.health),
        }
    }

    pub(crate) fn attack(&self) -> Option<i32> {
        match &self.kind {
            CardDefKind::Spell(_) => None,
            CardDefKind::Minion(minion) => Some(minion.attack),
            CardDefKind::Hero(_) => None,
        }
    }

    pub(crate) fn is_spell(&self) -> bool {
        matches!(self.kind, CardDefKind::Spell(_))
    }

    pub(crate) fn is_minion(&self) -> bool {
        matches!(self.kind, CardDefKind::Minion(_))
    }

    pub(crate) fn is_hero(&self) -> bool {
        matches!(self.kind, CardDefKind::Hero(_))
    }

    pub(crate) fn has_board_presence(&self) -> bool {
        match self.kind {
            CardDefKind::Spell(_) => false,
            CardDefKind::Minion(_) => true,
            CardDefKind::Hero(_) => false,
        }
    }

    pub(crate) fn has_deck_presence(&self) -> bool {
        match self.kind {
            CardDefKind::Spell(_) => true,
            CardDefKind::Minion(_) => true,
            CardDefKind::Hero(_) => false,
        }
    }

    pub(crate) fn friendly_aura_effects(&self) -> &[ModifierEffect] {
        match &self.kind {
            CardDefKind::Spell(_) => &[],
            CardDefKind::Minion(minion) => &minion.friendly_aura_effects,
            CardDefKind::Hero(_) => &[],
        }
    }

    pub(crate) fn hostile_aura_effects(&self) -> &[ModifierEffect] {
        match &self.kind {
            CardDefKind::Spell(_) => &[],
            CardDefKind::Minion(minion) => &minion.hostile_aura_effects,
            CardDefKind::Hero(_) => &[],
        }
    }

    pub(crate) fn choice(&self, id: ChoiceId) -> Option<&CharacterChoice> {
        self.choices.iter().find(|choice| choice.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardDefKind {
    Spell(SpellCardDef),
    Minion(MinionCardDef),
    Hero(HeroCardDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinionCardDef {
    pub attack: i32,
    pub health: i32,
    pub friendly_aura_effects: Vec<ModifierEffect>,
    pub hostile_aura_effects: Vec<ModifierEffect>,
}

impl MinionCardDef {
    pub fn new(attack: i32, health: i32) -> Self {
        Self {
            attack,
            health,
            friendly_aura_effects: Default::default(),
            hostile_aura_effects: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellCardDef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeroCardDef {
    pub health: i32,
}
