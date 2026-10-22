use enumset::EnumSet;

use crate::cards::filter::{CharacterFilter, Side};
use crate::cards::modifier::ModifierEffect;
use crate::effect::Effect;
use crate::target::{NoTarget, Targets};
use crate::trigger::TriggerPattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef<T> {
    pub name: String,
    pub kind: CardKindDef,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub play_effects: Vec<Effect<T>>,
    pub triggers: Vec<TriggerDef>,
    pub targets: Targets,
}

impl<T> CardDef<T> {
    pub(crate) fn health(&self) -> Option<i32> {
        match &self.kind {
            CardKindDef::Spell(_) => None,
            CardKindDef::Minion(minion) => Some(minion.health),
            CardKindDef::Hero(hero) => Some(hero.health),
        }
    }

    pub(crate) fn attack(&self) -> Option<i32> {
        match &self.kind {
            CardKindDef::Spell(_) => None,
            CardKindDef::Minion(minion) => Some(minion.attack),
            CardKindDef::Hero(_) => None,
        }
    }

    pub(crate) fn card_kind(&self) -> CardKind {
        match self.kind {
            CardKindDef::Spell(_) => CardKind::Spell,
            CardKindDef::Minion(_) => CardKind::Minion,
            CardKindDef::Hero(_) => CardKind::Hero,
        }
    }

    pub(crate) fn character_kind(&self) -> Option<CharacterKind> {
        match self.kind {
            CardKindDef::Spell(_) => None,
            CardKindDef::Minion(_) => Some(CharacterKind::Minion),
            CardKindDef::Hero(_) => Some(CharacterKind::Hero),
        }
    }

    pub(crate) fn has_board_presence(&self) -> bool {
        match self.kind {
            CardKindDef::Spell(_) => false,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn receives_auras(&self) -> bool {
        match self.kind {
            CardKindDef::Spell(_) => false,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn leaves_zone_on_death(&self) -> bool {
        match self.kind {
            CardKindDef::Spell(_) => true,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn goes_to_graveyard(&self) -> bool {
        match self.kind {
            CardKindDef::Spell(_) => true,
            CardKindDef::Minion(_) => true,
            CardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn friendly_aura_effects(&self) -> &[ModifierEffect] {
        match &self.kind {
            CardKindDef::Spell(_) => &[],
            CardKindDef::Minion(minion) => &minion.friendly_aura_effects,
            CardKindDef::Hero(_) => &[],
        }
    }

    pub(crate) fn hostile_aura_effects(&self) -> &[ModifierEffect] {
        match &self.kind {
            CardKindDef::Spell(_) => &[],
            CardKindDef::Minion(minion) => &minion.hostile_aura_effects,
            CardKindDef::Hero(_) => &[],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKindDef {
    Spell(SpellCardDef),
    Minion(MinionCardDef),
    Hero(HeroCardDef),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerDef {
    pub pattern: TriggerPattern,
    pub effects: Vec<Effect<NoTarget>>,
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
pub enum CharacterSelector<T> {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Targeted(T),
}

impl<T> CharacterSelector<T> {
    pub(crate) fn try_map_targeted<U, E>(
        self,
        f: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<CharacterSelector<U>, E> {
        Ok(match self {
            CharacterSelector::All(filter) => CharacterSelector::All(filter),
            CharacterSelector::Itself => CharacterSelector::Itself,
            CharacterSelector::OwnerHero => CharacterSelector::OwnerHero,
            CharacterSelector::Random(filter) => CharacterSelector::Random(filter),
            CharacterSelector::Targeted(target) => CharacterSelector::Targeted(f(target)?),
        })
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    fn double(n: usize) -> Result<usize, ()> {
        Ok(n * 2)
    }

    #[test]
    fn mapping_a_targeted_selector_maps_what_it_holds() {
        assert_eq!(
            CharacterSelector::Targeted(3).try_map_targeted(double),
            Ok(CharacterSelector::Targeted(6)),
        );
    }

    #[test]
    fn mapping_keeps_a_selector_that_targets_nothing() {
        let selectors = [
            CharacterSelector::All(CharacterFilter::all()),
            CharacterSelector::Itself,
            CharacterSelector::OwnerHero,
            CharacterSelector::Random(CharacterFilter::all()),
        ];

        assert_eq!(
            selectors.map(|selector| selector.try_map_targeted(double)),
            [
                Ok(CharacterSelector::All(CharacterFilter::all())),
                Ok(CharacterSelector::Itself),
                Ok(CharacterSelector::OwnerHero),
                Ok(CharacterSelector::Random(CharacterFilter::all())),
            ],
        );
    }

    #[test]
    fn mapping_a_targeted_selector_fails_when_the_map_does() {
        assert_eq!(
            CharacterSelector::Targeted(3).try_map_targeted(|_| Err::<usize, _>("undeclared")),
            Err("undeclared"),
        );
    }
}
