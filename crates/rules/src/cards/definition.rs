use enumset::EnumSet;

use crate::cards::filter::{CharacterFilter, Side};
use crate::cards::modifier::ModifierEffect;
use crate::effect::Effect;
use crate::target::{CharacterTarget, NoTarget, TargetId};
use crate::trigger::TriggerPattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef {
    pub name: String,
    pub kind: CardKindDef,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub play_effects: Vec<Effect<CharacterTarget>>,
    pub triggers: Vec<TriggerDef>,
}

impl CardDef {
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

    /// Declared by the play effects' `Targeted` selectors, in effect order. Each one's id is its
    /// place among them.
    pub(crate) fn targets(&self) -> impl Iterator<Item = (TargetId, &CharacterTarget)> {
        self.play_effects
            .iter()
            .filter_map(Effect::targeted)
            .enumerate()
            .map(|(idx, target)| (TargetId(idx), target))
    }

    /// The play effects, each target replaced by what `targeted` gives for its id.
    pub(crate) fn bind_play_effects<U>(
        &self,
        mut targeted: impl FnMut(TargetId) -> U,
    ) -> Vec<Effect<U>> {
        let mut target_ids = self.targets().map(|(target_id, _)| target_id);
        self.play_effects
            .iter()
            .map(|effect| {
                effect.clone().map_targeted(|_| {
                    let target_id = target_ids.next().expect(
                        "every target the play effects declare should be listed by `targets`",
                    );
                    targeted(target_id)
                })
            })
            .collect()
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
    pub(crate) fn targeted(&self) -> Option<&T> {
        match self {
            CharacterSelector::All(_)
            | CharacterSelector::Itself
            | CharacterSelector::OwnerHero
            | CharacterSelector::Random(_) => None,
            CharacterSelector::Targeted(target) => Some(target),
        }
    }

    pub(crate) fn map_targeted<U>(self, f: impl FnOnce(T) -> U) -> CharacterSelector<U> {
        match self {
            CharacterSelector::All(filter) => CharacterSelector::All(filter),
            CharacterSelector::Itself => CharacterSelector::Itself,
            CharacterSelector::OwnerHero => CharacterSelector::OwnerHero,
            CharacterSelector::Random(filter) => CharacterSelector::Random(filter),
            CharacterSelector::Targeted(target) => CharacterSelector::Targeted(f(target)),
        }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::target::AmountBound;

    fn target(count: u8) -> CharacterTarget {
        CharacterTarget {
            filter: CharacterFilter::all(),
            count: count.into(),
            bound: AmountBound::Exactly,
            different: true,
        }
    }

    fn card(play_effects: Vec<Effect<CharacterTarget>>) -> CardDef {
        CardDef {
            play_effects,
            ..Default::default()
        }
    }

    fn draw<T>() -> Effect<T> {
        Effect::Draw {
            selector: PlayerSelector::Owner,
            amount: 1.into(),
        }
    }

    fn damage<T>(selector: CharacterSelector<T>) -> Effect<T> {
        Effect::Damage {
            selector,
            amount: 1.into(),
        }
    }

    fn two_targeted_damages<T>(first: T, second: T) -> Vec<Effect<T>> {
        vec![
            draw(),
            damage(CharacterSelector::Targeted(first)),
            damage(CharacterSelector::Itself),
            damage(CharacterSelector::Targeted(second)),
        ]
    }

    fn untargeted() -> [CharacterSelector<usize>; 4] {
        [
            CharacterSelector::All(CharacterFilter::all()),
            CharacterSelector::Itself,
            CharacterSelector::OwnerHero,
            CharacterSelector::Random(CharacterFilter::all()),
        ]
    }

    #[test]
    fn each_targets_id_is_its_place_among_the_play_effects_targeted_selectors() {
        let (first, second) = (target(1), target(2));
        let def = card(two_targeted_damages(first, second));

        assert_eq!(
            def.targets().collect::<Vec<_>>(),
            [(TargetId(0), &first), (TargetId(1), &second)],
        );
    }

    #[test]
    fn play_effects_with_no_targeted_selector_declare_no_target() {
        let def = card(vec![draw(), damage(CharacterSelector::Itself)]);

        assert_eq!(def.targets().next(), None);
    }

    #[test]
    fn binding_replaces_each_target_with_what_is_given_for_its_id() {
        let def = card(two_targeted_damages(target(1), target(2)));

        assert_eq!(
            def.bind_play_effects(|target_id| target_id),
            two_targeted_damages(TargetId(0), TargetId(1)),
        );
    }

    #[test]
    fn a_targeted_selector_gives_what_it_holds() {
        assert_eq!(CharacterSelector::Targeted(3).targeted(), Some(&3));
    }

    #[test]
    fn a_selector_that_targets_nothing_gives_nothing_targeted() {
        assert_eq!(
            untargeted().each_ref().map(CharacterSelector::targeted),
            [None; 4]
        );
    }

    #[test]
    fn mapping_a_targeted_selector_maps_what_it_holds() {
        assert_eq!(
            CharacterSelector::Targeted(3).map_targeted(|n| n * 2),
            CharacterSelector::Targeted(6),
        );
    }

    #[test]
    fn mapping_keeps_a_selector_that_targets_nothing() {
        assert_eq!(
            untargeted().map(|selector| selector.map_targeted(|n| n * 2)),
            untargeted(),
        );
    }
}
