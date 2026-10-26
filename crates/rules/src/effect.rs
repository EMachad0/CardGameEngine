use crate::cards::binder::def_id::DefId;
use crate::cards::definition::{CardSelector, CharacterSelector, PlayerSelector};
use crate::cards::modifier::ModifierEffect;
use crate::history::HistoryQuery;
use crate::target::{MapTargets, TargetChoiceRef, TargetForm};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectAmount {
    Static(u8),
    History(HistoryQuery),
}

impl From<u8> for EffectAmount {
    fn from(value: u8) -> Self {
        Self::Static(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect<S: TargetForm> {
    Players {
        selector: PlayerSelector,
        verbs: Vec<PlayerVerb>,
    },
    Characters {
        selector: CharacterSelector<S>,
        verbs: Vec<CharacterVerb>,
    },
    Cards {
        selector: CardSelector<S>,
        verbs: Vec<CardVerb>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerVerb {
    Draw(EffectAmount),
    Summon(DefId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CharacterVerb {
    Damage(EffectAmount),
    Modify(ModifierEffect),
    AddFriendlyAura(ModifierEffect),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CardVerb {
    Move(CardDestinationZone),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardDestinationZone {
    Hand,
    DeckBottom,
}

impl<S: TargetForm> Effect<S> {
    pub(crate) fn target(&self) -> Option<TargetChoiceRef<'_, S>> {
        match self {
            Effect::Players {
                selector: _,
                verbs: _,
            } => None,
            Effect::Characters { selector, verbs: _ } => {
                selector.target().map(|t| TargetChoiceRef::Character(t))
            }
            Effect::Cards { selector, verbs: _ } => {
                selector.target().map(|t| TargetChoiceRef::Card(t))
            }
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(self, f: &mut impl MapTargets<S, R>) -> Effect<R> {
        match self {
            Effect::Players { selector, verbs } => Effect::Players { selector, verbs },
            Effect::Characters { selector, verbs } => Effect::Characters {
                selector: selector.map_targeted(|target| f.map_character(target)),
                verbs,
            },
            Effect::Cards { selector, verbs } => Effect::Cards {
                selector: selector.map_targeted(|target| f.map_card(target)),
                verbs,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::marker::PhantomData;

    use enumset::EnumSet;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::choice::{AmountBound, Choice};
    use crate::pick::{CardPickOptions, CardPickZone, DeckWindow};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Holds<T>(PhantomData<T>);

    impl<T: Debug + Clone + PartialEq + Eq> TargetForm for Holds<T> {
        type CharacterTarget = T;
        type CardTarget = T;
    }

    struct TagsSubject;

    impl MapTargets<Holds<usize>, Holds<(&'static str, usize)>> for TagsSubject {
        fn map_character(&mut self, target: usize) -> (&'static str, usize) {
            ("character", target)
        }

        fn map_card(&mut self, target: usize) -> (&'static str, usize) {
            ("card", target)
        }
    }

    fn targets_characters<T: Debug + Clone + PartialEq + Eq>(target: T) -> Effect<Holds<T>> {
        let buff = ModifierEffect::BuffAtk { amount: 1.into() };
        Effect::Characters {
            selector: CharacterSelector::Targeted(target),
            verbs: vec![
                CharacterVerb::Damage(1.into()),
                CharacterVerb::Modify(buff.clone()),
                CharacterVerb::AddFriendlyAura(buff),
            ],
        }
    }

    fn targets_cards<T: Debug + Clone + PartialEq + Eq>(target: T) -> Effect<Holds<T>> {
        Effect::Cards {
            selector: CardSelector::Targeted(target),
            verbs: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
        }
    }

    fn untargeted<S: TargetForm>() -> [Effect<S>; 2] {
        [
            Effect::Players {
                selector: PlayerSelector::Owner,
                verbs: vec![PlayerVerb::Draw(1.into())],
            },
            Effect::Cards {
                selector: CardSelector::Picked {
                    choice: Choice {
                        options: CardPickOptions {
                            zone: CardPickZone::YourDeck(DeckWindow::Top(2)),
                            kind: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                    others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                },
                verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
            },
        ]
    }

    #[test]
    fn a_characters_effect_gives_its_target_as_a_character_target() {
        assert_eq!(
            targets_characters(3).target(),
            Some(TargetChoiceRef::Character(&3)),
        );
    }

    #[test]
    fn a_cards_effect_gives_its_target_as_a_card_target() {
        assert_eq!(targets_cards(3).target(), Some(TargetChoiceRef::Card(&3)));
    }

    #[test]
    fn an_effect_with_no_targeted_selector_has_no_target() {
        assert_eq!(
            untargeted::<Holds<usize>>().each_ref().map(Effect::target),
            [None, None],
        );
    }

    #[test]
    fn mapping_a_characters_effect_maps_its_target_as_a_character() {
        assert_eq!(
            targets_characters(3).map_targeted(&mut TagsSubject),
            targets_characters(("character", 3)),
        );
    }

    #[test]
    fn mapping_a_cards_effect_maps_its_target_as_a_card() {
        assert_eq!(
            targets_cards(3).map_targeted(&mut TagsSubject),
            targets_cards(("card", 3)),
        );
    }

    #[test]
    fn mapping_keeps_an_effect_with_no_targeted_selector() {
        assert_eq!(
            untargeted::<Holds<usize>>().map(|effect| effect.map_targeted(&mut TagsSubject)),
            untargeted(),
        );
    }
}
