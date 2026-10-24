use crate::cards::binder::def_id::DefId;
use crate::cards::definition::{CharacterSelector, PlayerSelector};
use crate::cards::modifier::ModifierEffect;
use crate::history::HistoryQuery;
use crate::target::TargetForm;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PickId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneMove {
    AddToHand,
    Bury,
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
    Reveal {
        pick_id: PickId,
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    ResolvePick {
        pick_id: PickId,
        picked_move: ZoneMove,
        other_move: ZoneMove,
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

impl<S: TargetForm> Effect<S> {
    pub(crate) fn target(&self) -> Option<&S::CharacterTarget> {
        match self {
            Effect::Players {
                selector: _,
                verbs: _,
            }
            | Effect::Reveal {
                pick_id: _,
                selector: _,
                amount: _,
            }
            | Effect::ResolvePick {
                pick_id: _,
                picked_move: _,
                other_move: _,
            } => None,
            Effect::Characters { selector, verbs: _ } => selector.target(),
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: impl FnOnce(S::CharacterTarget) -> R::CharacterTarget,
    ) -> Effect<R> {
        match self {
            Effect::Players { selector, verbs } => Effect::Players { selector, verbs },
            Effect::Characters { selector, verbs } => Effect::Characters {
                selector: selector.map_targeted(f),
                verbs,
            },
            Effect::Reveal {
                pick_id,
                selector,
                amount,
            } => Effect::Reveal {
                pick_id,
                selector,
                amount,
            },
            Effect::ResolvePick {
                pick_id,
                picked_move,
                other_move,
            } => Effect::ResolvePick {
                pick_id,
                picked_move,
                other_move,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::Debug;
    use std::marker::PhantomData;

    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct Holds<T>(PhantomData<T>);

    impl<T: Debug + Clone + PartialEq + Eq> TargetForm for Holds<T> {
        type CharacterTarget = T;
    }

    fn targeted(target: usize) -> Effect<Holds<usize>> {
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

    fn untargeted() -> [Effect<Holds<usize>>; 2] {
        [
            Effect::Players {
                selector: PlayerSelector::Owner,
                verbs: vec![PlayerVerb::Draw(1.into())],
            },
            Effect::ResolvePick {
                pick_id: PickId(1),
                picked_move: ZoneMove::AddToHand,
                other_move: ZoneMove::Bury,
            },
        ]
    }

    #[test]
    fn an_effect_gives_what_its_targeted_selector_holds() {
        assert_eq!(targeted(3).target(), Some(&3));
    }

    #[test]
    fn an_effect_with_no_character_selector_has_no_target() {
        assert_eq!(untargeted().each_ref().map(Effect::target), [None, None]);
    }

    #[test]
    fn mapping_the_targeted_selector_maps_what_it_holds() {
        assert_eq!(targeted(3).map_targeted(|n| n * 2), targeted(6));
    }

    #[test]
    fn mapping_keeps_an_effect_with_no_character_selector() {
        assert_eq!(
            untargeted().map(|effect| effect.map_targeted(|n| n * 2)),
            untargeted(),
        );
    }
}
