use crate::cards::{amount::EffectAmount, trigger::TriggerDef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierEffect {
    BuffAtk { amount: EffectAmount },
    BuffHealth { amount: EffectAmount },
    ReduceManaCost { amount: EffectAmount },
    GrantTrigger(TriggerDef),
}
