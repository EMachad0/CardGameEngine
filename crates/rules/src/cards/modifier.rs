use crate::{
    cards::{definition::TriggerDef, object::ObjectId},
    effect::EffectAmount,
};

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Modifiers(Vec<Modifier>);

impl Modifiers {
    pub(crate) fn add(&mut self, modifier: Modifier) {
        self.0.push(modifier);
    }

    pub(crate) fn as_slice(&self) -> &[Modifier] {
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

/// What a list of modifiers changes, each sorted to the stat its effect changes. Amounts stay
/// unevaluated until a stat is asked for.
#[derive(Debug, Default)]
pub(crate) struct StatChanges<'a> {
    attack: Vec<&'a EffectAmount>,
    health: Vec<&'a EffectAmount>,
    cost: Vec<&'a EffectAmount>,
    triggers: Vec<(ObjectId, &'a TriggerDef)>,
}

impl<'a> StatChanges<'a> {
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
    use super::*;

    use pretty_assertions::assert_eq;

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
            effects: Vec::new().into(),
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

    fn changes<'a>(granted_by: ObjectId, effects: &'a [ModifierEffect]) -> StatChanges<'a> {
        let mut changes = StatChanges::default();
        for effect in effects {
            changes.add(granted_by, effect);
        }
        changes
    }

    #[test]
    fn each_modifier_changes_only_its_own_stat() {
        let [granter] = ids();
        let died = trigger(TriggerPattern::Died(TriggerSubject::Itself));
        let effects = [
            attack(1),
            health(2),
            cost(3),
            ModifierEffect::GrantTrigger(died.clone()),
        ];
        let changes = changes(granter, &effects);

        assert_eq!(
            (
                changes.attack_bonus(evaluate),
                changes.health_bonus(evaluate),
                changes.cost_reduction(evaluate),
                changes.granted_triggers().collect::<Vec<_>>(),
            ),
            (1, 2, 3, vec![(granter, &died)]),
            "each stat takes only the modifier that changes it",
        );
    }

    #[test]
    fn modifiers_of_one_kind_add_up() {
        let [granter] = ids();
        let effects = [attack(1), attack(2), health(3), health(4), cost(1), cost(2)];
        let changes = changes(granter, &effects);

        assert_eq!(
            (
                changes.attack_bonus(evaluate),
                changes.health_bonus(evaluate),
                changes.cost_reduction(evaluate),
            ),
            (3, 7, 3),
        );
    }

    #[test]
    fn no_modifiers_change_nothing() {
        let changes = StatChanges::default();

        assert_eq!(
            (
                changes.attack_bonus(evaluate),
                changes.health_bonus(evaluate),
                changes.cost_reduction(evaluate),
                changes.granted_triggers().count(),
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
        let changes = changes(granter, &effects);

        assert_eq!(
            (
                changes.attack_bonus(evaluate),
                changes.health_bonus(evaluate),
                changes.cost_reduction(evaluate),
            ),
            (5, 5, 5),
            "each history amount must count as the 5 the evaluation returns",
        );
    }

    #[test]
    fn granted_triggers_keep_their_order_and_the_object_granting_each() {
        let [first, second] = ids();
        let died = trigger(TriggerPattern::Died(TriggerSubject::Itself));
        let damaged = trigger(TriggerPattern::Damaged(TriggerSubject::Itself));
        let first_effects = [ModifierEffect::GrantTrigger(died.clone())];
        let second_effects = [ModifierEffect::GrantTrigger(damaged.clone())];
        let mut changes = changes(first, &first_effects);
        changes.add(second, &second_effects[0]);

        assert_eq!(
            changes.granted_triggers().collect::<Vec<_>>(),
            [(first, &died), (second, &damaged)],
        );
    }
}
