use enumset::EnumSet;

use crate::{
    cards::{loader::PLACEHOLDER, modifier::ModifierEffect},
    choice::{CharacterChoice, ChoiceId, EffectAmount},
};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq, Hash)]
pub struct DefId(&'static str);

impl DefId {
    pub(super) const fn new(id: &'static str) -> Self {
        Self(id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef {
    pub id: DefId,
    pub name: String,
    pub kind: CardDefKind,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub on_play_effect: EffectSequence,
    pub on_board_enter: EffectSequence,
    pub on_board_leave: EffectSequence,
    pub on_death: EffectSequence,
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
            on_play_effect: Default::default(),
            on_board_enter: Default::default(),
            on_board_leave: Default::default(),
            on_death: Default::default(),
            choices: Default::default(),
        }
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Draw {
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Reveal {
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Damage {
        selector: CharacterSelector,
        amount: EffectAmount,
    },
    AddFriendlyAura {
        selector: CharacterSelector,
        effect: ModifierEffect,
    },
    Summon {
        selector: PlayerSelector,
        def_id: DefId,
    },
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EffectSequence(pub Vec<Effect>);

impl EffectSequence {
    pub fn as_slice(&self) -> &[Effect] {
        self.0.as_slice()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSelectorFilter {
    pub side: EnumSet<CharacterSideFilter>,
}

impl PlayerSelectorFilter {
    pub fn all() -> Self {
        Self {
            side: EnumSet::all(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSelector {
    All(PlayerSelectorFilter),
    Owner,
    Random(PlayerSelectorFilter),
}

#[derive(Debug, enumset::EnumSetType)]
pub enum CharacterKindFilter {
    Heroes,
    Minions,
}

#[derive(Debug, enumset::EnumSetType)]
pub enum CharacterSideFilter {
    Friendly,
    Enemy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CharacterSelectorFilter {
    pub kind: EnumSet<CharacterKindFilter>,
    pub side: EnumSet<CharacterSideFilter>,
}

impl CharacterSelectorFilter {
    pub fn all() -> Self {
        Self {
            kind: EnumSet::all(),
            side: EnumSet::all(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSelector {
    All(CharacterSelectorFilter),
    Itself,
    OwnerHero,
    Random(CharacterSelectorFilter),
    Chosen(ChoiceId),
}
