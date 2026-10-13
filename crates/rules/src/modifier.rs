//! Granted modifiers, and the stat bonuses and triggers they add up to.

use crate::{
    cards::{amount::EffectAmount, modifier::ModifierEffect, trigger::TriggerDef},
    object::ObjectId,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub(crate) fn new(modifiers: Vec<Modifier>) -> Self {
        Self(modifiers)
    }

    pub(crate) fn add(&mut self, modifier: Modifier) {
        self.0.push(modifier);
    }

    pub(crate) fn extend(&mut self, modifiers: Self) {
        self.0.extend(modifiers.0);
    }

    pub(crate) fn attack_bonus(&self, evaluate: impl Fn(&EffectAmount) -> u8) -> i32 {
        self.0
            .iter()
            .filter_map(|m| match &m.effect {
                ModifierEffect::BuffAtk { amount } => Some(evaluate(amount) as i32),
                ModifierEffect::BuffHealth { .. }
                | ModifierEffect::ReduceManaCost { .. }
                | ModifierEffect::GrantTrigger(_) => None,
            })
            .sum::<i32>()
    }

    pub(crate) fn health_bonus(&self, evaluate: impl Fn(&EffectAmount) -> u8) -> i32 {
        self.0
            .iter()
            .filter_map(|m| match &m.effect {
                ModifierEffect::BuffHealth { amount } => Some(evaluate(amount) as i32),
                ModifierEffect::BuffAtk { .. }
                | ModifierEffect::ReduceManaCost { .. }
                | ModifierEffect::GrantTrigger(_) => None,
            })
            .sum::<i32>()
    }

    pub(crate) fn cost_reduction(&self, evaluate: impl Fn(&EffectAmount) -> u8) -> u8 {
        self.0
            .iter()
            .filter_map(|m| match &m.effect {
                ModifierEffect::ReduceManaCost { amount } => Some(evaluate(amount)),
                ModifierEffect::BuffAtk { .. }
                | ModifierEffect::BuffHealth { .. }
                | ModifierEffect::GrantTrigger(_) => None,
            })
            .sum::<u8>()
    }

    pub(crate) fn granted_triggers(&self) -> impl Iterator<Item = (ObjectId, &TriggerDef)> {
        self.0.iter().filter_map(|m| match &m.effect {
            ModifierEffect::GrantTrigger(trigger) => Some((m.granted_by, trigger)),
            ModifierEffect::BuffAtk { .. }
            | ModifierEffect::BuffHealth { .. }
            | ModifierEffect::ReduceManaCost { .. } => None,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Modifier {
    pub granted_by: ObjectId,
    pub effect: ModifierEffect,
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::cards::{
        amount::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter},
        def_ids::static_card_definition::SPARK,
        trigger::{TriggerPattern, TriggerSubject},
    };
    use crate::ids::PlayerId;
    use crate::object::ObjectBag;

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(SPARK, PlayerId::new(0)))
    }

    fn evaluate(amount: &EffectAmount) -> u8 {
        match amount {
            EffectAmount::Static(value) => *value,
            EffectAmount::History(_) => 5,
        }
    }

    fn spells_played() -> EffectAmount {
        EffectAmount::History(HistoryQuery {
            kind: HistoryQueryKind::SpellsPlayed,
            scope: PlayerFilter::Owner,
            turn: TurnFilter::All,
        })
    }

    fn trigger(pattern: TriggerPattern) -> TriggerDef {
        TriggerDef {
            pattern,
            effects: Vec::new().into(),
        }
    }

    fn modifiers(granted_by: ObjectId, effects: Vec<ModifierEffect>) -> Modifiers {
        Modifiers::new(
            effects
                .into_iter()
                .map(|effect| Modifier { granted_by, effect })
                .collect(),
        )
    }

    fn grant_death_rattle() -> ModifierEffect {
        ModifierEffect::GrantTrigger(trigger(TriggerPattern::Died(TriggerSubject::Itself)))
    }

    fn attack(amount: u8) -> ModifierEffect {
        ModifierEffect::BuffAtk {
            amount: EffectAmount::Static(amount),
        }
    }

    fn health(amount: u8) -> ModifierEffect {
        ModifierEffect::BuffHealth {
            amount: EffectAmount::Static(amount),
        }
    }

    fn cost(amount: u8) -> ModifierEffect {
        ModifierEffect::ReduceManaCost {
            amount: EffectAmount::Static(amount),
        }
    }

    #[test]
    fn attack_bonus_sums_every_attack_buff() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![attack(1), attack(2)]);

        assert_eq!(modifiers.attack_bonus(evaluate), 3);
    }

    #[test]
    fn attack_bonus_ignores_health_cost_and_trigger_modifiers() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![health(1), cost(2), grant_death_rattle()]);

        assert_eq!(modifiers.attack_bonus(evaluate), 0);
    }

    #[test]
    fn health_bonus_sums_every_health_buff() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![health(1), health(2)]);

        assert_eq!(modifiers.health_bonus(evaluate), 3);
    }

    #[test]
    fn health_bonus_ignores_attack_cost_and_trigger_modifiers() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![attack(1), cost(2), grant_death_rattle()]);

        assert_eq!(modifiers.health_bonus(evaluate), 0);
    }

    #[test]
    fn cost_reduction_sums_every_cost_reduction() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![cost(1), cost(2)]);

        assert_eq!(modifiers.cost_reduction(evaluate), 3);
    }

    #[test]
    fn cost_reduction_ignores_attack_health_and_trigger_modifiers() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![attack(1), health(2), grant_death_rattle()]);

        assert_eq!(modifiers.cost_reduction(evaluate), 0);
    }

    #[test]
    fn stat_amounts_are_what_the_callers_evaluation_returns() {
        let [granter] = ids();
        let modifiers = modifiers(
            granter,
            vec![
                ModifierEffect::BuffAtk {
                    amount: spells_played(),
                },
                ModifierEffect::BuffHealth {
                    amount: spells_played(),
                },
                ModifierEffect::ReduceManaCost {
                    amount: spells_played(),
                },
            ],
        );

        let bonuses = (
            modifiers.attack_bonus(evaluate),
            modifiers.health_bonus(evaluate),
            modifiers.cost_reduction(evaluate),
        );

        assert_eq!(
            bonuses,
            (5, 5, 5),
            "each history amount must count as the 5 the evaluation returns",
        );
    }

    #[test]
    fn granted_triggers_lists_each_trigger_in_order_with_the_object_granting_it() {
        let [first, second] = ids();
        let died = trigger(TriggerPattern::Died(TriggerSubject::Itself));
        let damaged = trigger(TriggerPattern::Damaged(TriggerSubject::Itself));
        let mut modifiers = modifiers(first, vec![ModifierEffect::GrantTrigger(died.clone())]);
        modifiers.add(Modifier {
            granted_by: second,
            effect: ModifierEffect::GrantTrigger(damaged.clone()),
        });

        let triggers = modifiers.granted_triggers().collect::<Vec<_>>();

        assert_eq!(triggers, [(first, &died), (second, &damaged)]);
    }

    #[test]
    fn granted_triggers_ignores_stat_modifiers() {
        let [granter] = ids();
        let modifiers = modifiers(granter, vec![attack(1), health(2), cost(3)]);

        assert_eq!(modifiers.granted_triggers().count(), 0);
    }
}
