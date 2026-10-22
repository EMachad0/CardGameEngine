use enumset::EnumSet;

use crate::cards::filter::{CharacterFilter, Side};
use crate::cards::modifier::ModifierEffect;
use crate::effect::EffectSequence;
use crate::target::{CharacterTarget, TargetId};
use crate::trigger::TriggerPattern;

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
    pub kind: CardKindDef,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub play_effects: EffectSequence,
    pub triggers: Vec<TriggerDef>,
    pub targets: Vec<CharacterTarget>,
}

impl CardDef {
    pub(crate) fn target(&self, id: TargetId) -> Option<&CharacterTarget> {
        self.targets.iter().find(|target| target.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKindDef<G = TargetId> {
    Spell(SpellCardDef),
    Minion(MinionCardDef<G>),
    Hero(HeroCardDef),
}

impl<G> CardKindDef<G> {
    pub(crate) fn health(&self) -> Option<i32> {
        match self {
            CardKindDef::Spell(_) => None,
            CardKindDef::Minion(minion) => Some(minion.health),
            CardKindDef::Hero(hero) => Some(hero.health),
        }
    }

    pub(crate) fn attack(&self) -> Option<i32> {
        match self {
            CardKindDef::Spell(_) => None,
            CardKindDef::Minion(minion) => Some(minion.attack),
            CardKindDef::Hero(_) => None,
        }
    }

    pub(crate) fn card_kind(&self) -> CardKind {
        match self {
            CardKindDef::Spell(_) => CardKind::Spell,
            CardKindDef::Minion(_) => CardKind::Minion,
            CardKindDef::Hero(_) => CardKind::Hero,
        }
    }

    pub(crate) fn character_kind(&self) -> Option<CharacterKind> {
        match self {
            CardKindDef::Spell(_) => None,
            CardKindDef::Minion(_) => Some(CharacterKind::Minion),
            CardKindDef::Hero(_) => Some(CharacterKind::Hero),
        }
    }

    pub(crate) fn has_board_presence(&self) -> bool {
        match self {
            CardKindDef::Spell(_) => false,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn receives_auras(&self) -> bool {
        match self {
            CardKindDef::Spell(_) => false,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn leaves_zone_on_death(&self) -> bool {
        match self {
            CardKindDef::Spell(_) => true,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn goes_to_graveyard(&self) -> bool {
        match self {
            CardKindDef::Spell(_) => true,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn friendly_aura_effects(&self) -> &[ModifierEffect<G>] {
        match self {
            CardKindDef::Spell(_) => &[],
            CardKindDef::Minion(minion) => &minion.friendly_aura_effects,
            CardKindDef::Hero(_) => &[],
        }
    }

    pub(crate) fn hostile_aura_effects(&self) -> &[ModifierEffect<G>] {
        match self {
            CardKindDef::Spell(_) => &[],
            CardKindDef::Minion(minion) => &minion.hostile_aura_effects,
            CardKindDef::Hero(_) => &[],
        }
    }
}

#[derive(Debug, enumset::EnumSetType)]
pub enum CardKind {
    Spell,
    Minion,
    Hero,
}

#[derive(Debug, enumset::EnumSetType)]
pub enum CharacterKind {
    Hero,
    Minion,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MinionCardDef<G = TargetId> {
    pub attack: i32,
    pub health: i32,
    pub friendly_aura_effects: Vec<ModifierEffect<G>>,
    pub hostile_aura_effects: Vec<ModifierEffect<G>>,
}

impl<G> MinionCardDef<G> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDef<G = TargetId> {
    pub pattern: TriggerPattern,
    pub effects: EffectSequence<G>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlayerSelectorFilter {
    pub side: EnumSet<Side>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterSelector<T = TargetId> {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Targeted(T),
}
