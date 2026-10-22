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
    use crate::cards::filter::CharacterFilter;

    fn double(n: usize) -> Result<usize, ()> {
        Ok(n * 2)
    }

    #[test]
    fn mapping_the_targeted_selector_maps_what_it_holds() {
        let damage = Effect::Damage {
            selector: CharacterSelector::Targeted(3),
            amount: 1.into(),
        };

        assert_eq!(
            damage.try_map_targeted(double),
            Ok(Effect::Damage {
                selector: CharacterSelector::Targeted(6),
                amount: 1.into(),
            }),
        );
    }

    #[test]
    fn mapping_the_targeted_selector_keeps_any_other_selector() {
        let buff = Effect::<usize>::Buff {
            selector: CharacterSelector::All(CharacterFilter::all()),
            effect: ModifierEffect::BuffAtk { amount: 1.into() },
        };

        assert_eq!(
            buff.try_map_targeted(double),
            Ok(Effect::Buff {
                selector: CharacterSelector::All(CharacterFilter::all()),
                effect: ModifierEffect::BuffAtk { amount: 1.into() },
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
