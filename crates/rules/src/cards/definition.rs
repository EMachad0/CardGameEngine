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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardDefKind {
    Monster(MonsterCardDef),
    Spell(SpellCardDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonsterCardDef {
    pub health: i32,
    pub attack: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpellCardDef {}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct EffectSequence(pub Vec<Effect>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    DamagePlayer {
        targeteer: PlayerTargeteer,
        damage: u8,
    },
    Draw {
        targeteer: PlayerTargeteer,
        count: usize,
    },
    RevealToPicker {
        targeteer: PlayerTargeteer,
        count: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerTargeteer {
    Caster,
    RandomPlayer,
    NextPlayer,
}
