use std::convert::Infallible;

use crate::cards::binder::def_id::DefId;
use crate::cards::definition::{CharacterSelector, PlayerSelector};
use crate::cards::modifier::ModifierEffect;
use crate::history::HistoryQuery;

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
pub enum Effect<T> {
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
        selector: CharacterSelector<T>,
        amount: EffectAmount,
    },
    Buff {
        selector: CharacterSelector<T>,
        effect: ModifierEffect,
    },
    AddFriendlyAura {
        selector: CharacterSelector<T>,
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

impl<T> Effect<T> {
    pub(crate) fn map_targeted<U>(self, f: impl FnOnce(T) -> U) -> Effect<U> {
        let Ok(mapped) = self.try_map_targeted(|target| Ok::<_, Infallible>(f(target)));
        mapped
    }

    pub(crate) fn try_map_targeted<U, E>(
        self,
        f: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<Effect<U>, E> {
        Ok(match self {
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
                selector: selector.try_map_targeted(f)?,
                amount,
            },
            Effect::Buff { selector, effect } => Effect::Buff {
                selector: selector.try_map_targeted(f)?,
                effect,
            },
            Effect::AddFriendlyAura { selector, effect } => Effect::AddFriendlyAura {
                selector: selector.try_map_targeted(f)?,
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
    fn mapping_the_targeted_selector_maps_what_it_holds() {
        let buff = ModifierEffect::BuffAtk { amount: 1.into() };
        let effects = [
            Effect::Damage {
                selector: CharacterSelector::Targeted(3),
                amount: 1.into(),
            },
            Effect::Buff {
                selector: CharacterSelector::Targeted(4),
                effect: buff.clone(),
            },
            Effect::AddFriendlyAura {
                selector: CharacterSelector::Targeted(5),
                effect: buff.clone(),
            },
        ];

        assert_eq!(
            effects.map(|effect| effect.try_map_targeted(double)),
            [
                Ok(Effect::Damage {
                    selector: CharacterSelector::Targeted(6),
                    amount: 1.into(),
                }),
                Ok(Effect::Buff {
                    selector: CharacterSelector::Targeted(8),
                    effect: buff.clone(),
                }),
                Ok(Effect::AddFriendlyAura {
                    selector: CharacterSelector::Targeted(10),
                    effect: buff,
                }),
            ],
        );
    }

    #[test]
    fn mapping_the_targeted_selector_keeps_an_effect_with_no_character_selector() {
        let resolve_pick = Effect::<usize>::ResolvePick {
            pick_id: PickId(1),
            picked_move: ZoneMove::AddToHand,
            other_move: ZoneMove::Bury,
        };

        assert_eq!(
            resolve_pick.try_map_targeted(double),
            Ok(Effect::ResolvePick {
                pick_id: PickId(1),
                picked_move: ZoneMove::AddToHand,
                other_move: ZoneMove::Bury,
            }),
        );
    }

    #[test]
    fn mapping_the_targeted_selector_fails_when_the_map_does() {
        let damage = Effect::Damage {
            selector: CharacterSelector::Targeted(3),
            amount: 1.into(),
        };

        assert_eq!(
            damage.try_map_targeted(|_| Err::<usize, _>("undeclared")),
            Err("undeclared"),
        );
    }
}
