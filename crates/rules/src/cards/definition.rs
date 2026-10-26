use enumset::EnumSet;

use crate::cards::filter::{CharacterFilter, Side};
use crate::cards::modifier::ModifierEffect;
use crate::cards::object::ObjectId;
use crate::choice::Choice;
use crate::effect::{CardVerb, Effect};
use crate::pick::CardPickOptions;
use crate::target::{
    Bound, BoundCards, Declared, MapTargets, TargetChoiceRef, TargetForm, TargetId, Triggered,
};
use crate::trigger::TriggerPattern;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardDef {
    pub name: String,
    pub kind: CardKindDef,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ModifierEffect>,
    pub play_effects: Vec<Effect<Declared>>,
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
    pub(crate) fn targets(
        &self,
    ) -> impl Iterator<Item = (TargetId, TargetChoiceRef<'_, Declared>)> {
        self.play_effects
            .iter()
            .filter_map(Effect::target)
            .enumerate()
            .map(|(idx, target)| (TargetId(idx), target))
    }

    /// The play effects, each target replaced by the objects `targeted` gives for its id, a card
    /// target with its zone.
    pub(crate) fn bind_play_effects(
        &self,
        targeted: impl FnMut(TargetId) -> Vec<ObjectId>,
    ) -> Vec<Effect<Bound>> {
        let target_ids = self.targets().map(|(target_id, _)| target_id);
        let mut bind_targets = BindTargets {
            target_ids,
            targeted,
        };
        self.play_effects
            .iter()
            .map(|effect| effect.clone().map_targeted(&mut bind_targets))
            .collect()
    }
}

impl Default for CardDef {
    fn default() -> Self {
        Self {
            name: Default::default(),
            kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
            mana_cost: Default::default(),
            modifier_effects: Default::default(),
            play_effects: Default::default(),
            triggers: Default::default(),
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
    pub effects: Vec<Effect<Triggered>>,
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
pub enum CharacterSelector<S: TargetForm> {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Targeted(S::CharacterTarget),
}

impl<S: TargetForm> CharacterSelector<S> {
    pub(crate) fn target(&self) -> Option<&S::CharacterTarget> {
        match self {
            CharacterSelector::All(_)
            | CharacterSelector::Itself
            | CharacterSelector::OwnerHero
            | CharacterSelector::Random(_) => None,
            CharacterSelector::Targeted(target) => Some(target),
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: impl FnOnce(S::CharacterTarget) -> R::CharacterTarget,
    ) -> CharacterSelector<R> {
        match self {
            CharacterSelector::All(filter) => CharacterSelector::All(filter),
            CharacterSelector::Itself => CharacterSelector::Itself,
            CharacterSelector::OwnerHero => CharacterSelector::OwnerHero,
            CharacterSelector::Random(filter) => CharacterSelector::Random(filter),
            CharacterSelector::Targeted(target) => CharacterSelector::Targeted(f(target)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardSelector<S: TargetForm> {
    Targeted(S::CardTarget),
    /// `others` acts on the cards the pick covered and the player didn't pick, in the order
    /// shown, after the effect's verbs act on the cards picked.
    Picked {
        choice: Choice<CardPickOptions>,
        others: Vec<CardVerb>,
    },
}

impl<S: TargetForm> CardSelector<S> {
    pub(crate) fn target(&self) -> Option<&S::CardTarget> {
        match self {
            CardSelector::Targeted(target) => Some(target),
            CardSelector::Picked {
                choice: _,
                others: _,
            } => None,
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: impl FnOnce(S::CardTarget) -> R::CardTarget,
    ) -> CardSelector<R> {
        match self {
            CardSelector::Targeted(target) => CardSelector::Targeted(f(target)),
            CardSelector::Picked { choice, others } => CardSelector::Picked { choice, others },
        }
    }
}

struct BindTargets<I, F> {
    target_ids: I,
    targeted: F,
}

impl<I, F> BindTargets<I, F>
where
    I: Iterator<Item = TargetId>,
    F: FnMut(TargetId) -> Vec<ObjectId>,
{
    fn next_targeted(&mut self) -> Vec<ObjectId> {
        let target_id = self
            .target_ids
            .next()
            .expect("every target the play effects declare should be listed by `targets`");
        (self.targeted)(target_id)
    }
}

impl<I, F> MapTargets<Declared, Bound> for BindTargets<I, F>
where
    I: Iterator<Item = TargetId>,
    F: FnMut(TargetId) -> Vec<ObjectId>,
{
    fn map_character(
        &mut self,
        _: <Declared as TargetForm>::CharacterTarget,
    ) -> <Bound as TargetForm>::CharacterTarget {
        self.next_targeted()
    }

    fn map_card(
        &mut self,
        choice: <Declared as TargetForm>::CardTarget,
    ) -> <Bound as TargetForm>::CardTarget {
        BoundCards {
            zone: choice.options.zone,
            object_ids: self.next_targeted(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::marker::PhantomData;

    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;
    use crate::choice::AmountBound;
    use crate::effect::{CardDestinationZone, CharacterVerb, PlayerVerb};
    use crate::ids::PlayerId;
    use crate::target::{CardTargetOptions, CardTargetZone};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Holds<T>(PhantomData<T>);

    impl<T: Debug + Clone + PartialEq + Eq> TargetForm for Holds<T> {
        type CharacterTarget = T;
        type CardTarget = T;
    }

    fn character_target(count: u8) -> Choice<CharacterFilter> {
        Choice {
            options: CharacterFilter::all(),
            count: count.into(),
            bound: AmountBound::Exactly,
            different: true,
        }
    }

    fn card_target(count: u8) -> Choice<CardTargetOptions> {
        Choice {
            options: CardTargetOptions {
                zone: CardTargetZone::Hand,
                kind: CardKind::Minion.into(),
            },
            count: count.into(),
            bound: AmountBound::Exactly,
            different: true,
        }
    }

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    fn card(play_effects: Vec<Effect<Declared>>) -> CardDef {
        CardDef {
            play_effects,
            ..Default::default()
        }
    }

    fn draw<S: TargetForm>() -> Effect<S> {
        Effect::Players {
            selector: PlayerSelector::Owner,
            verbs: vec![PlayerVerb::Draw(1.into())],
        }
    }

    fn damage<S: TargetForm>(selector: CharacterSelector<S>) -> Effect<S> {
        Effect::Characters {
            selector,
            verbs: vec![CharacterVerb::Damage(1.into())],
        }
    }

    fn bury<S: TargetForm>(selector: CardSelector<S>) -> Effect<S> {
        Effect::Cards {
            selector,
            verbs: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
        }
    }

    fn untargeted_effects<S: TargetForm>() -> Vec<Effect<S>> {
        vec![draw(), damage(CharacterSelector::Itself)]
    }

    fn two_targets_among_untargeted_effects<S: TargetForm>(
        first: S::CharacterTarget,
        second: S::CharacterTarget,
    ) -> Vec<Effect<S>> {
        vec![
            draw(),
            damage(CharacterSelector::Targeted(first)),
            damage(CharacterSelector::Itself),
            damage(CharacterSelector::Targeted(second)),
        ]
    }

    fn targets_of_both_subjects<S: TargetForm>(
        first: S::CharacterTarget,
        second: S::CardTarget,
        third: S::CharacterTarget,
    ) -> Vec<Effect<S>> {
        vec![
            damage(CharacterSelector::Targeted(first)),
            bury(CardSelector::Targeted(second)),
            damage(CharacterSelector::Targeted(third)),
        ]
    }

    fn untargeted_selectors() -> [CharacterSelector<Holds<usize>>; 4] {
        [
            CharacterSelector::All(CharacterFilter::all()),
            CharacterSelector::Itself,
            CharacterSelector::OwnerHero,
            CharacterSelector::Random(CharacterFilter::all()),
        ]
    }

    #[test]
    fn each_targets_id_is_its_place_among_the_play_effects_targeted_selectors() {
        let (first, second) = (character_target(1), character_target(2));
        let def = card(two_targets_among_untargeted_effects(first, second));

        assert_eq!(
            def.targets().collect::<Vec<_>>(),
            [
                (TargetId(0), TargetChoiceRef::Character(&first)),
                (TargetId(1), TargetChoiceRef::Character(&second)),
            ],
        );
    }

    #[test]
    fn character_and_card_targets_take_their_ids_from_one_count() {
        let (first, second, third) = (character_target(1), card_target(1), character_target(2));
        let def = card(targets_of_both_subjects(first, second, third));

        assert_eq!(
            def.targets().collect::<Vec<_>>(),
            [
                (TargetId(0), TargetChoiceRef::Character(&first)),
                (TargetId(1), TargetChoiceRef::Card(&second)),
                (TargetId(2), TargetChoiceRef::Character(&third)),
            ],
        );
    }

    #[test]
    fn play_effects_with_no_targeted_selector_declare_no_target() {
        let def = card(untargeted_effects());

        assert_eq!(def.targets().next(), None);
    }

    #[test]
    fn binding_replaces_each_target_with_what_is_given_for_its_id() {
        let [first, second] = ids();
        let def = card(two_targets_among_untargeted_effects(
            character_target(1),
            character_target(2),
        ));

        assert_eq!(
            def.bind_play_effects(|target_id| vec![[first, second][target_id.0]]),
            two_targets_among_untargeted_effects::<Bound>(vec![first], vec![second]),
        );
    }

    #[test]
    fn binding_replaces_a_card_target_with_its_zone_and_what_is_given_for_its_id() {
        let [first, second, third] = ids();
        let def = card(targets_of_both_subjects(
            character_target(1),
            card_target(1),
            character_target(1),
        ));

        assert_eq!(
            def.bind_play_effects(|target_id| vec![[first, second, third][target_id.0]]),
            targets_of_both_subjects::<Bound>(
                vec![first],
                BoundCards {
                    zone: CardTargetZone::Hand,
                    object_ids: vec![second],
                },
                vec![third],
            ),
        );
    }

    #[test]
    fn binding_play_effects_that_declare_no_target_asks_for_nothing() {
        let def = card(untargeted_effects());
        let mut asked = Vec::new();

        def.bind_play_effects(|target_id| {
            asked.push(target_id);
            Vec::new()
        });

        assert_eq!(asked, Vec::<TargetId>::new());
    }

    #[test]
    fn a_targeted_selector_gives_what_it_holds() {
        assert_eq!(
            CharacterSelector::<Holds<usize>>::Targeted(3).target(),
            Some(&3)
        );
    }

    #[test]
    fn a_selector_that_targets_nothing_has_no_target() {
        assert_eq!(
            untargeted_selectors()
                .each_ref()
                .map(CharacterSelector::target),
            [None; 4]
        );
    }

    #[test]
    fn mapping_a_targeted_selector_maps_what_it_holds() {
        assert_eq!(
            CharacterSelector::<Holds<usize>>::Targeted(3).map_targeted::<Holds<_>>(|n| n * 2),
            CharacterSelector::Targeted(6),
        );
    }

    #[test]
    fn mapping_keeps_a_selector_that_targets_nothing() {
        assert_eq!(
            untargeted_selectors().map(|selector| selector.map_targeted(|n| n * 2)),
            untargeted_selectors(),
        );
    }
}
