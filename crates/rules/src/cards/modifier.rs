use crate::cards::definition::TriggerDef;
use crate::cards::object::ObjectId;
use crate::effect::EffectAmount;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(super) struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub(super) fn add(&mut self, modifier: Modifier) {
        self.0.push(modifier);
    }

    pub(super) fn as_slice(&self) -> &[Modifier] {
        self.0.as_slice()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Modifier {
    pub granted_by: ObjectId,
    pub effect: ModifierEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierEffect {
    BuffAtk { amount: EffectAmount },
    BuffHealth { amount: EffectAmount },
    ReduceManaCost { amount: EffectAmount },
    GrantTrigger(TriggerDef),
}

/// Modifiers sorted by what each changes: a stat, or the triggers. Amounts stay unevaluated until
/// a stat is asked for.
#[derive(Debug, Default)]
pub(crate) struct SortedModifiers<'a> {
    attack: Vec<&'a EffectAmount>,
    health: Vec<&'a EffectAmount>,
    cost: Vec<&'a EffectAmount>,
    triggers: Vec<(ObjectId, &'a TriggerDef)>,
}

impl<'a> SortedModifiers<'a> {
    pub(crate) fn add(&mut self, granted_by: ObjectId, effect: &'a ModifierEffect) {
        match effect {
            ModifierEffect::BuffAtk { amount } => self.attack.push(amount),
            ModifierEffect::BuffHealth { amount } => self.health.push(amount),
            ModifierEffect::ReduceManaCost { amount } => self.cost.push(amount),
            ModifierEffect::GrantTrigger(trigger) => self.triggers.push((granted_by, trigger)),
        }
    }

    pub(crate) fn attack_bonus(&self, evaluate: impl Fn(&EffectAmount) -> u8) -> i32 {
        self.attack.iter().map(|a| evaluate(a) as i32).sum()
    }

    pub(crate) fn health_bonus(&self, evaluate: impl Fn(&EffectAmount) -> u8) -> i32 {
        self.health.iter().map(|a| evaluate(a) as i32).sum()
    }

    pub(crate) fn cost_reduction(&self, evaluate: impl Fn(&EffectAmount) -> u8) -> u8 {
        self.cost.iter().map(|a| evaluate(a)).sum()
    }

    /// In the order they were added, each with the object granting it.
    pub(crate) fn granted_triggers(&self) -> impl Iterator<Item = (ObjectId, &'a TriggerDef)> {
        self.triggers.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base::static_card_definition::SPARK;
    use crate::cards::object::ObjectBag;
    use crate::history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter};
    use crate::ids::PlayerId;
    use crate::trigger::{TriggerPattern, TriggerSubject};

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
            effects: Vec::new(),
        }
    }

    fn attack(amount: u8) -> ModifierEffect {
        ModifierEffect::BuffAtk {
            amount: amount.into(),
        }
    }

    fn health(amount: u8) -> ModifierEffect {
        ModifierEffect::BuffHealth {
            amount: amount.into(),
        }
    }

    fn cost(amount: u8) -> ModifierEffect {
        ModifierEffect::ReduceManaCost {
            amount: amount.into(),
        }
    }

    fn sorted(granted_by: ObjectId, effects: &[ModifierEffect]) -> SortedModifiers<'_> {
        let mut modifiers = SortedModifiers::default();
        for effect in effects {
            modifiers.add(granted_by, effect);
        }
        modifiers
    }

    #[test]
    fn each_modifier_is_sorted_under_what_it_changes() {
        let [granter] = ids();
        let died = trigger(TriggerPattern::Died(TriggerSubject::Itself));
        let effects = [
            attack(1),
            health(2),
            cost(3),
            ModifierEffect::GrantTrigger(died.clone()),
        ];
        let modifiers = sorted(granter, &effects);

        assert_eq!(
            (
                modifiers.attack_bonus(evaluate),
                modifiers.health_bonus(evaluate),
                modifiers.cost_reduction(evaluate),
                modifiers.granted_triggers().collect::<Vec<_>>(),
            ),
            (1, 2, 3, vec![(granter, &died)]),
            "each modifier counts only toward the stat or the triggers it changes",
        );
    }

    #[test]
    fn modifiers_of_one_kind_add_up() {
        let [granter] = ids();
        let effects = [attack(1), attack(2), health(3), health(4), cost(1), cost(2)];
        let modifiers = sorted(granter, &effects);

        assert_eq!(
            (
                modifiers.attack_bonus(evaluate),
                modifiers.health_bonus(evaluate),
                modifiers.cost_reduction(evaluate),
            ),
            (3, 7, 3),
        );
    }

    #[test]
    fn no_modifiers_change_nothing() {
        let modifiers = SortedModifiers::default();

        assert_eq!(
            (
                modifiers.attack_bonus(evaluate),
                modifiers.health_bonus(evaluate),
                modifiers.cost_reduction(evaluate),
                modifiers.granted_triggers().count(),
            ),
            (0, 0, 0, 0),
        );
    }

    #[test]
    fn an_amount_counts_as_the_callers_evaluation_of_it() {
        let [granter] = ids();
        let effects = [
            ModifierEffect::BuffAtk {
                amount: spells_played(),
            },
            ModifierEffect::BuffHealth {
                amount: spells_played(),
            },
            ModifierEffect::ReduceManaCost {
                amount: spells_played(),
            },
        ];
        let modifiers = sorted(granter, &effects);

        assert_eq!(
            (
                modifiers.attack_bonus(evaluate),
                modifiers.health_bonus(evaluate),
                modifiers.cost_reduction(evaluate),
            ),
            (5, 5, 5),
            "each history amount must count as the 5 the evaluation returns",
        );
    }

    #[test]
    fn granted_triggers_come_in_the_order_they_were_added() {
        let [first, second] = ids();
        let died = trigger(TriggerPattern::Died(TriggerSubject::Itself));
        let damaged = trigger(TriggerPattern::Damaged(TriggerSubject::Itself));
        let first_effects = [ModifierEffect::GrantTrigger(died.clone())];
        let second_effects = [ModifierEffect::GrantTrigger(damaged.clone())];
        let mut modifiers = sorted(first, &first_effects);
        modifiers.add(second, &second_effects[0]);

        assert_eq!(
            modifiers.granted_triggers().collect::<Vec<_>>(),
            [(first, &died), (second, &damaged)],
        );
    }
}
