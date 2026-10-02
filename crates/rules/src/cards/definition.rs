use crate::cards::object::ModifierEffect;

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
    pub on_play_effect: EffectSequence,
    pub on_board_enter: EffectSequence,
    pub on_board_leave: EffectSequence,
    pub on_death: EffectSequence,
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
