use crate::cards::{loader::PLACEHOLDER, modifier::ModifierEffect};

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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EffectSequence(pub Vec<Effect>);

impl EffectSequence {
    pub fn as_slice(&self) -> &[Effect] {
        self.0.as_slice()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    DamagePlayer {
        selector: PlayerSelector,
        damage: u8,
    },
    DamageMinion {
        selector: MinionSelector,
        damage: u8,
    },
    Draw {
        selector: PlayerSelector,
        count: usize,
    },
    Reveal {
        selector: PlayerSelector,
        count: usize,
    },
    AddFriendlyAura {
        selector: MinionSelector,
        effect: ModifierEffect,
    },
    Summon {
        selector: PlayerSelector,
        def_id: DefId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerSelector {
    All,
    Caster,
    RandomPlayer,
    NextPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MinionSelector {
    All,
    Itself,
}
