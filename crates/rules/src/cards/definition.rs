use enumset::EnumSet;

use crate::cards::filter::{CharacterFilter, Side};
use crate::cards::modifier::{ModifierEffect, ParsedModifierEffect};
use crate::cards::object::ObjectId;
use crate::choice::Choice;
use crate::effect::{CardVerb, CharacterVerb, Effect, ParsedCharacterVerb, ParsedEffect};
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

/// A `CardDef` as its binder parsed it, naming each card by a `DefId` of that binder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedCardDef {
    pub name: String,
    pub kind: ParsedCardKindDef,
    pub mana_cost: u8,
    pub modifier_effects: Vec<ParsedModifierEffect>,
    pub play_effects: Vec<ParsedEffect<Declared>>,
    pub triggers: Vec<ParsedTriggerDef>,
}

impl ParsedCardDef {
    pub(crate) fn health(&self) -> Option<i32> {
        match &self.kind {
            ParsedCardKindDef::Spell(_) => None,
            ParsedCardKindDef::Minion(minion) => Some(minion.health),
            ParsedCardKindDef::Hero(hero) => Some(hero.health),
        }
    }

    pub(crate) fn attack(&self) -> Option<i32> {
        match &self.kind {
            ParsedCardKindDef::Spell(_) => None,
            ParsedCardKindDef::Minion(minion) => Some(minion.attack),
            ParsedCardKindDef::Hero(_) => None,
        }
    }

    pub(crate) fn card_kind(&self) -> CardKind {
        match self.kind {
            ParsedCardKindDef::Spell(_) => CardKind::Spell,
            ParsedCardKindDef::Minion(_) => CardKind::Minion,
            ParsedCardKindDef::Hero(_) => CardKind::Hero,
        }
    }

    pub(crate) fn character_kind(&self) -> Option<CharacterKind> {
        match self.kind {
            ParsedCardKindDef::Spell(_) => None,
            ParsedCardKindDef::Minion(_) => Some(CharacterKind::Minion),
            ParsedCardKindDef::Hero(_) => Some(CharacterKind::Hero),
        }
    }

    pub(crate) fn has_board_presence(&self) -> bool {
        match self.kind {
            ParsedCardKindDef::Spell(_) => false,
            ParsedCardKindDef::Minion(_) => true,
            ParsedCardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn receives_auras(&self) -> bool {
        match self.kind {
            ParsedCardKindDef::Spell(_) => false,
            ParsedCardKindDef::Minion(_) => true,
            ParsedCardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn leaves_zone_on_death(&self) -> bool {
        match self.kind {
            ParsedCardKindDef::Spell(_) => true,
            ParsedCardKindDef::Minion(_) => true,
            ParsedCardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn goes_to_graveyard(&self) -> bool {
        match self.kind {
            ParsedCardKindDef::Spell(_) => true,
            ParsedCardKindDef::Minion(_) => true,
            ParsedCardKindDef::Hero(_) => false,
        }
    }

    pub(crate) fn friendly_aura_effects(&self) -> &[ParsedModifierEffect] {
        match &self.kind {
            ParsedCardKindDef::Spell(_) => &[],
            ParsedCardKindDef::Minion(minion) => &minion.friendly_aura_effects,
            ParsedCardKindDef::Hero(_) => &[],
        }
    }

    pub(crate) fn hostile_aura_effects(&self) -> &[ParsedModifierEffect] {
        match &self.kind {
            ParsedCardKindDef::Spell(_) => &[],
            ParsedCardKindDef::Minion(minion) => &minion.hostile_aura_effects,
            ParsedCardKindDef::Hero(_) => &[],
        }
    }

    /// Declared by the play effects' `Targeted` selectors, in effect order. Each one's id is its
    /// place among them.
    pub(crate) fn targets(
        &self,
    ) -> impl Iterator<Item = (TargetId, TargetChoiceRef<'_, Declared>)> {
        self.play_effects
            .iter()
            .filter_map(ParsedEffect::target)
            .enumerate()
            .map(|(idx, target)| (TargetId(idx), target))
    }

    /// The play effects, each target replaced by the objects `targeted` gives for its id, a card
    /// target with its zone.
    pub(crate) fn bind_play_effects(
        &self,
        targeted: impl FnMut(TargetId) -> Vec<ObjectId>,
    ) -> Vec<ParsedEffect<Bound>> {
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardKindDef {
    Spell(SpellCardDef),
    Minion(MinionCardDef),
    Hero(HeroCardDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedCardKindDef {
    Spell(SpellCardDef),
    Minion(ParsedMinionCardDef),
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
pub(crate) struct ParsedMinionCardDef {
    pub attack: i32,
    pub health: i32,
    pub friendly_aura_effects: Vec<ParsedModifierEffect>,
    pub hostile_aura_effects: Vec<ParsedModifierEffect>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ParsedTriggerDef {
    pub pattern: TriggerPattern,
    pub effects: Vec<ParsedEffect<Triggered>>,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterSelector<S: TargetForm> {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Targeted(S::CharacterTarget),
    /// `others` acts on the options the player didn't pick, in the order shown, after the
    /// effect's verbs act on the characters picked.
    Picked {
        choice: Choice<CharacterFilter>,
        others: Vec<CharacterVerb>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedCharacterSelector<S: TargetForm> {
    All(CharacterFilter),
    Itself,
    OwnerHero,
    Random(CharacterFilter),
    Targeted(S::CharacterTarget),
    Picked {
        choice: Choice<CharacterFilter>,
        others: Vec<ParsedCharacterVerb>,
    },
}

impl<S: TargetForm> ParsedCharacterSelector<S> {
    pub(crate) fn target(&self) -> Option<&S::CharacterTarget> {
        match self {
            ParsedCharacterSelector::All(_)
            | ParsedCharacterSelector::Itself
            | ParsedCharacterSelector::OwnerHero
            | ParsedCharacterSelector::Random(_)
            | ParsedCharacterSelector::Picked {
                choice: _,
                others: _,
            } => None,
            ParsedCharacterSelector::Targeted(target) => Some(target),
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: impl FnOnce(S::CharacterTarget) -> R::CharacterTarget,
    ) -> ParsedCharacterSelector<R> {
        match self {
            ParsedCharacterSelector::All(filter) => ParsedCharacterSelector::All(filter),
            ParsedCharacterSelector::Itself => ParsedCharacterSelector::Itself,
            ParsedCharacterSelector::OwnerHero => ParsedCharacterSelector::OwnerHero,
            ParsedCharacterSelector::Random(filter) => ParsedCharacterSelector::Random(filter),
            ParsedCharacterSelector::Targeted(target) => {
                ParsedCharacterSelector::Targeted(f(target))
            }
            ParsedCharacterSelector::Picked { choice, others } => {
                ParsedCharacterSelector::Picked { choice, others }
            }
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
    use crate::cards::binder::def_code::DefCode;
    use crate::cards::binder::def_id::DefId;
    use crate::cards::binder::set::{Set, SetCard};
    use crate::cards::binder::store::Binder;
    use crate::cards::object::ObjectBag;
    use crate::choice::AmountBound;
    use crate::effect::{CardDestinationZone, ParsedCharacterVerb, ParsedPlayerVerb};
    use crate::ids::PlayerId;
    use crate::target::{CardTargetOptions, CardTargetZone};

    /// The `DefId` of a card that no test looks up.
    fn def_id() -> DefId {
        let card = SetCard {
            name: "card",
            version: 0,
            def: CardDef::default(),
        };
        let binder = Binder::new([Set {
            prefix: "unit",
            cards: vec![card],
        }])
        .unwrap();
        binder
            .def_id(DefCode {
                set: "unit",
                name: "card",
                version: 0,
            })
            .unwrap()
    }

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
        std::array::from_fn(|_| bag.insert(def_id(), PlayerId::new(0)))
    }

    fn card(play_effects: Vec<ParsedEffect<Declared>>) -> ParsedCardDef {
        ParsedCardDef {
            name: String::new(),
            kind: ParsedCardKindDef::Spell(SpellCardDef),
            mana_cost: 0,
            modifier_effects: Vec::new(),
            play_effects,
            triggers: Vec::new(),
        }
    }

    fn draw<S: TargetForm>() -> ParsedEffect<S> {
        ParsedEffect::Players {
            selector: PlayerSelector::Owner,
            verbs: vec![ParsedPlayerVerb::Draw(1.into())],
        }
    }

    fn damage<S: TargetForm>(selector: ParsedCharacterSelector<S>) -> ParsedEffect<S> {
        ParsedEffect::Characters {
            selector,
            verbs: vec![ParsedCharacterVerb::Damage(1.into())],
        }
    }

    fn bury<S: TargetForm>(selector: CardSelector<S>) -> ParsedEffect<S> {
        ParsedEffect::Cards {
            selector,
            verbs: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
        }
    }

    fn untargeted_effects<S: TargetForm>() -> Vec<ParsedEffect<S>> {
        vec![draw(), damage(ParsedCharacterSelector::Itself)]
    }

    fn two_targets_among_untargeted_effects<S: TargetForm>(
        first: S::CharacterTarget,
        second: S::CharacterTarget,
    ) -> Vec<ParsedEffect<S>> {
        vec![
            draw(),
            damage(ParsedCharacterSelector::Targeted(first)),
            damage(ParsedCharacterSelector::Itself),
            damage(ParsedCharacterSelector::Targeted(second)),
        ]
    }

    fn targets_of_both_subjects<S: TargetForm>(
        first: S::CharacterTarget,
        second: S::CardTarget,
        third: S::CharacterTarget,
    ) -> Vec<ParsedEffect<S>> {
        vec![
            damage(ParsedCharacterSelector::Targeted(first)),
            bury(CardSelector::Targeted(second)),
            damage(ParsedCharacterSelector::Targeted(third)),
        ]
    }

    fn untargeted_selectors() -> [ParsedCharacterSelector<Holds<usize>>; 5] {
        [
            ParsedCharacterSelector::All(CharacterFilter::all()),
            ParsedCharacterSelector::Itself,
            ParsedCharacterSelector::OwnerHero,
            ParsedCharacterSelector::Random(CharacterFilter::all()),
            ParsedCharacterSelector::Picked {
                choice: character_target(1),
                others: vec![ParsedCharacterVerb::Damage(1.into())],
            },
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
            ParsedCharacterSelector::<Holds<usize>>::Targeted(3).target(),
            Some(&3)
        );
    }

    #[test]
    fn a_selector_that_targets_nothing_has_no_target() {
        assert_eq!(
            untargeted_selectors()
                .each_ref()
                .map(ParsedCharacterSelector::target),
            [None; 5]
        );
    }

    #[test]
    fn mapping_a_targeted_selector_maps_what_it_holds() {
        assert_eq!(
            ParsedCharacterSelector::<Holds<usize>>::Targeted(3)
                .map_targeted::<Holds<_>>(|n| n * 2),
            ParsedCharacterSelector::Targeted(6),
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
