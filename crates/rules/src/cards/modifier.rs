use crate::cards::definition::{ParsedTriggerDef, TriggerDef};
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
    pub effect: ParsedModifierEffect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModifierEffect {
    BuffAtk { amount: EffectAmount },
    BuffHealth { amount: EffectAmount },
    ReduceManaCost { amount: EffectAmount },
    GrantTrigger(TriggerDef),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParsedModifierEffect {
    BuffAtk { amount: EffectAmount },
    BuffHealth { amount: EffectAmount },
    ReduceManaCost { amount: EffectAmount },
    GrantTrigger(ParsedTriggerDef),
}

/// Modifiers sorted by what each changes: a stat, or the triggers. Amounts stay unevaluated until
/// a stat is asked for.
#[derive(Debug, Default)]
pub(crate) struct SortedModifiers<'a> {
    attack: Vec<&'a EffectAmount>,
    health: Vec<&'a EffectAmount>,
    cost: Vec<&'a EffectAmount>,
    triggers: Vec<(ObjectId, &'a ParsedTriggerDef)>,
}

impl<'a> SortedModifiers<'a> {
    pub(crate) fn add(&mut self, granted_by: ObjectId, effect: &'a ParsedModifierEffect) {
        match effect {
            ParsedModifierEffect::BuffAtk { amount } => self.attack.push(amount),
            ParsedModifierEffect::BuffHealth { amount } => self.health.push(amount),
            ParsedModifierEffect::ReduceManaCost { amount } => self.cost.push(amount),
            ParsedModifierEffect::GrantTrigger(trigger) => {
                self.triggers.push((granted_by, trigger))
            }
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
    pub(crate) fn granted_triggers(
        &self,
    ) -> impl Iterator<Item = (ObjectId, &'a ParsedTriggerDef)> {
        self.triggers.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::def_code::DefCode;
    use crate::cards::binder::def_id::DefId;
    use crate::cards::binder::set::{Set, SetCard};
    use crate::cards::binder::store::Binder;
    use crate::cards::definition::CardDef;
    use crate::cards::object::ObjectBag;
    use crate::history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter};
    use crate::ids::PlayerId;

    /// The `DefId` of a card that no test looks up.
    fn def_id() -> DefId {
        let card = SetCard {
            name: "card",
            version: 0,
            def: CardDef::default(),
        };
        let binder = Binder::new([Set {
            prefix: "unit",
            cards: vec![card],
        }])
        .unwrap();
        binder
            .def_id(DefCode {
                set: "unit",
                name: "card",
                version: 0,
            })
            .unwrap()
    }
    use crate::trigger::{TriggerPattern, TriggerSubject};

    fn ids<const N: usize>() -> [ObjectId; N] {
        let mut bag = ObjectBag::default();
        std::array::from_fn(|_| bag.insert(def_id(), PlayerId::new(0)))
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

    fn trigger(pattern: TriggerPattern) -> ParsedTriggerDef {
        ParsedTriggerDef {
            pattern,
            effects: Vec::new(),
        }
    }

    fn attack(amount: u8) -> ParsedModifierEffect {
        ParsedModifierEffect::BuffAtk {
            amount: amount.into(),
        }
    }

    fn health(amount: u8) -> ParsedModifierEffect {
        ParsedModifierEffect::BuffHealth {
            amount: amount.into(),
        }
    }

    fn cost(amount: u8) -> ParsedModifierEffect {
        ParsedModifierEffect::ReduceManaCost {
            amount: amount.into(),
        }
    }

    fn sorted(granted_by: ObjectId, effects: &[ParsedModifierEffect]) -> SortedModifiers<'_> {
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
            ParsedModifierEffect::GrantTrigger(died.clone()),
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
            ParsedModifierEffect::BuffAtk {
                amount: spells_played(),
            },
            ParsedModifierEffect::BuffHealth {
                amount: spells_played(),
            },
            ParsedModifierEffect::ReduceManaCost {
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
        let first_effects = [ParsedModifierEffect::GrantTrigger(died.clone())];
        let second_effects = [ParsedModifierEffect::GrantTrigger(damaged.clone())];
        let mut modifiers = sorted(first, &first_effects);
        modifiers.add(second, &second_effects[0]);

        assert_eq!(
            modifiers.granted_triggers().collect::<Vec<_>>(),
            [(first, &died), (second, &damaged)],
        );
    }
}
