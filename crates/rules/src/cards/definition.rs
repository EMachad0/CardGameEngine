use crate::cards::{loader::PLACEHOLDER, modifier::ModifierEffect};

#[derive(Debug, Clone, Copy, PartialOrd, Ord, PartialEq, Eq)]
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
    pub modifer_effects: Vec<ModifierEffect>,
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
            kind: CardDefKind::Monster(MonsterCardDef::new(1, 1)),
            mana_cost: Default::default(),
            modifer_effects: Default::default(),
            on_play_effect: Default::default(),
            on_board_enter: Default::default(),
            on_board_leave: Default::default(),
            on_death: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardDefKind {
    Monster(MonsterCardDef),
    Spell(SpellCardDef),
}

impl CardDefKind {
    pub fn board_presence(&self) -> bool {
        match self {
            CardDefKind::Monster(_) => true,
            CardDefKind::Spell(_) => false,
        }
    }

    pub fn deck_presence(&self) -> bool {
        match self {
            CardDefKind::Monster(_) => true,
            CardDefKind::Spell(_) => true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterCardDef {
    pub health: i32,
    pub attack: i32,
    pub friendly_aura_effects: Vec<ModifierEffect>,
    pub hostile_aura_effects: Vec<ModifierEffect>,
}

impl MonsterCardDef {
    pub fn new(health: i32, attack: i32) -> Self {
        Self {
            health,
            attack,
            friendly_aura_effects: Default::default(),
            hostile_aura_effects: Default::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellCardDef {}

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
        targeteer: PlayerTargeteer,
        damage: u8,
    },
    DamageMonster {
        targeteer: MonsterTargeteer,
        damage: u8,
    },
    Draw {
        targeteer: PlayerTargeteer,
        count: usize,
    },
    Reveal {
        targeteer: PlayerTargeteer,
        count: usize,
    },
    AddFriendlyAura {
        targeteer: MonsterTargeteer,
        effect: ModifierEffect,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerTargeteer {
    All,
    Caster,
    RandomPlayer,
    NextPlayer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonsterTargeteer {
    All,
    Itself,
}
