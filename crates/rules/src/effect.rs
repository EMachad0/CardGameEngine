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
    Draw {
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Reveal {
        pick_id: PickId,
        selector: PlayerSelector,
        amount: EffectAmount,
    },
    Damage {
        selector: CharacterSelector<S>,
        amount: EffectAmount,
    },
    Buff {
        selector: CharacterSelector<S>,
        effect: ModifierEffect,
    },
    AddFriendlyAura {
        selector: CharacterSelector<S>,
        effect: ModifierEffect,
    },
    Summon {
        selector: PlayerSelector,
        def_id: DefId,
    },
    ResolvePick {
        pick_id: PickId,
        picked_move: ZoneMove,
        other_move: ZoneMove,
    },
}

impl<S: TargetForm> Effect<S> {
    pub(crate) fn target(&self) -> Option<&S::CharacterTarget> {
        match self {
            Effect::Draw {
                selector: _,
                amount: _,
            }
            | Effect::Reveal {
                pick_id: _,
                selector: _,
                amount: _,
            }
            | Effect::Summon {
                selector: _,
                def_id: _,
            }
            | Effect::ResolvePick {
                pick_id: _,
                picked_move: _,
                other_move: _,
            } => None,
            Effect::Damage {
                selector,
                amount: _,
            }
            | Effect::Buff {
                selector,
                effect: _,
            }
            | Effect::AddFriendlyAura {
                selector,
                effect: _,
            } => selector.target(),
        }
    }

    pub(crate) fn map_targeted<R: TargetForm>(
        self,
        f: impl FnOnce(S::CharacterTarget) -> R::CharacterTarget,
    ) -> Effect<R> {
        match self {
            Effect::Draw { selector, amount } => Effect::Draw { selector, amount },
            Effect::Reveal {
                pick_id,
                selector,
                amount,
            } => Effect::Reveal {
                pick_id,
                selector,
                amount,
            },
            Effect::Damage { selector, amount } => Effect::Damage {
                selector: selector.map_targeted(f),
                amount,
            },
            Effect::Buff { selector, effect } => Effect::Buff {
                selector: selector.map_targeted(f),
                effect,
            },
            Effect::AddFriendlyAura { selector, effect } => Effect::AddFriendlyAura {
                selector: selector.map_targeted(f),
                effect,
            },
            Effect::Summon { selector, def_id } => Effect::Summon { selector, def_id },
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
    use pretty_assertions::assert_eq;

    use super::*;

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Numbers {}

    impl TargetForm for Numbers {
        type CharacterTarget = usize;
    }

    fn targeted_effects([damaged, buffed, aura]: [usize; 3]) -> [Effect<Numbers>; 3] {
        let buff = ModifierEffect::BuffAtk { amount: 1.into() };
        [
            Effect::Damage {
                selector: CharacterSelector::Targeted(damaged),
                amount: 1.into(),
            },
            Effect::Buff {
                selector: CharacterSelector::Targeted(buffed),
                effect: buff.clone(),
            },
            Effect::AddFriendlyAura {
                selector: CharacterSelector::Targeted(aura),
                effect: buff,
            },
        ]
    }

    fn resolve_pick() -> Effect<Numbers> {
        Effect::ResolvePick {
            pick_id: PickId(1),
            picked_move: ZoneMove::AddToHand,
            other_move: ZoneMove::Bury,
        }
    }

    #[test]
    fn an_effect_gives_what_its_targeted_selector_holds() {
        assert_eq!(
            targeted_effects([3, 4, 5]).each_ref().map(Effect::target),
            [Some(&3), Some(&4), Some(&5)],
        );
    }

    #[test]
    fn an_effect_with_no_character_selector_has_no_target() {
        assert_eq!(resolve_pick().target(), None);
    }

    #[test]
    fn mapping_the_targeted_selector_maps_what_it_holds() {
        assert_eq!(
            targeted_effects([3, 4, 5]).map(|effect| effect.map_targeted::<Numbers>(|n| n * 2)),
            targeted_effects([6, 8, 10]),
        );
    }

    #[test]
    fn mapping_keeps_an_effect_with_no_character_selector() {
        assert_eq!(
            resolve_pick().map_targeted::<Numbers>(|n| n * 2),
            resolve_pick(),
        );
    }
}
