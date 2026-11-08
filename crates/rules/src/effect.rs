use crate::cards::binder::def_code::DefCode;
use crate::cards::binder::def_id::DefId;
use crate::cards::definition::{
    CardSelector, CharacterSelector, ParsedCharacterSelector, PlayerSelector,
};
use crate::cards::modifier::{ModifierEffect, ParsedModifierEffect};
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedEffect<S: TargetForm> {
    Players {
        selector: PlayerSelector,
        verbs: Vec<ParsedPlayerVerb>,
    },
    Characters {
        selector: ParsedCharacterSelector<S>,
        verbs: Vec<ParsedCharacterVerb>,
    },
    Cards {
        selector: CardSelector<S>,
        verbs: Vec<CardVerb>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerVerb {
    Draw(EffectAmount),
    Summon(DefCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParsedPlayerVerb {
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
pub(crate) enum ParsedCharacterVerb {
    Damage(EffectAmount),
    Modify(ParsedModifierEffect),
    AddFriendlyAura(ParsedModifierEffect),
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

impl<S: TargetForm> ParsedEffect<S> {
    pub(crate) fn target(&self) -> Option<TargetChoiceRef<'_, S>> {
        match self {
            ParsedEffect::Players {
                selector: _,
                verbs: _,
            } => None,
            ParsedEffect::Characters { selector, verbs: _ } => {
                selector.target().map(|t| TargetChoiceRef::Character(t))
            }
            ParsedEffect::Cards { selector, verbs: _ } => {
                selector.target().map(|t| TargetChoiceRef::Card(t))
            }
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: &mut impl MapTargets<S, R>,
    ) -> ParsedEffect<R> {
        match self {
            ParsedEffect::Players { selector, verbs } => ParsedEffect::Players { selector, verbs },
            ParsedEffect::Characters { selector, verbs } => ParsedEffect::Characters {
                selector: selector.map_targeted(|target| f.map_character(target)),
                verbs,
            },
            ParsedEffect::Cards { selector, verbs } => ParsedEffect::Cards {
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
    use crate::cards::filter::CharacterFilter;
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

    fn targets_characters<T: Debug + Clone + PartialEq + Eq>(target: T) -> ParsedEffect<Holds<T>> {
        let buff = ParsedModifierEffect::BuffAtk { amount: 1.into() };
        ParsedEffect::Characters {
            selector: ParsedCharacterSelector::Targeted(target),
            verbs: vec![
                ParsedCharacterVerb::Damage(1.into()),
                ParsedCharacterVerb::Modify(buff.clone()),
                ParsedCharacterVerb::AddFriendlyAura(buff),
            ],
        }
    }

    fn targets_cards<T: Debug + Clone + PartialEq + Eq>(target: T) -> ParsedEffect<Holds<T>> {
        ParsedEffect::Cards {
            selector: CardSelector::Targeted(target),
            verbs: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
        }
    }

    fn untargeted<S: TargetForm>() -> [ParsedEffect<S>; 3] {
        [
            ParsedEffect::Players {
                selector: PlayerSelector::Owner,
                verbs: vec![ParsedPlayerVerb::Draw(1.into())],
            },
            ParsedEffect::Characters {
                selector: ParsedCharacterSelector::Picked {
                    choice: Choice {
                        options: CharacterFilter::all(),
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                    others: vec![ParsedCharacterVerb::Damage(1.into())],
                },
                verbs: vec![ParsedCharacterVerb::Damage(2.into())],
            },
            ParsedEffect::Cards {
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
            untargeted::<Holds<usize>>()
                .each_ref()
                .map(ParsedEffect::target),
            [None, None, None],
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
