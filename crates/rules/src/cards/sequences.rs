use crate::cards::definition::{
    CardDef, CardKindDef, HeroCardDef, MinionCardDef, SpellCardDef, TriggerDef,
};
use crate::cards::modifier::ModifierEffect;
use crate::effect::Effect;
use crate::target::NoTarget;

/// The effects of every trigger on a card, printed or granted through modifier effects, auras,
/// `Buff` and `AddFriendlyAura`, at any depth. Those its play effects grant come first.
pub(super) fn trigger_sequences(def: &CardDef) -> Vec<&[Effect<NoTarget>]> {
    let CardDef {
        id: _,
        name: _,
        kind,
        mana_cost: _,
        modifier_effects,
        play_effects,
        triggers,
        targets: _,
    } = def;
    let mut sequences = Vec::new();
    collect_grants(&mut sequences, play_effects);
    for trigger in triggers {
        collect_trigger(&mut sequences, trigger);
    }
    for modifier_effect in modifier_effects {
        collect_modifier(&mut sequences, modifier_effect);
    }
    match kind {
        CardKindDef::Spell(SpellCardDef) => {}
        CardKindDef::Minion(MinionCardDef {
            attack: _,
            health: _,
            friendly_aura_effects,
            hostile_aura_effects,
        }) => {
            for modifier_effect in friendly_aura_effects.iter().chain(hostile_aura_effects) {
                collect_modifier(&mut sequences, modifier_effect);
            }
        }
        CardKindDef::Hero(HeroCardDef { health: _ }) => {}
    }
    sequences
}

fn collect_grants<'a, T>(sequences: &mut Vec<&'a [Effect<NoTarget>]>, effects: &'a [Effect<T>]) {
    for effect in effects {
        match effect {
            Effect::Draw {
                selector: _,
                amount: _,
            }
            | Effect::Reveal {
                pick_id: _,
                selector: _,
                amount: _,
            }
            | Effect::Damage {
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
            } => {}
            Effect::Buff {
                selector: _,
                effect,
            }
            | Effect::AddFriendlyAura {
                selector: _,
                effect,
            } => collect_modifier(sequences, effect),
        }
    }
}

fn collect_modifier<'a>(
    sequences: &mut Vec<&'a [Effect<NoTarget>]>,
    modifier_effect: &'a ModifierEffect,
) {
    match modifier_effect {
        ModifierEffect::BuffAtk { amount: _ }
        | ModifierEffect::BuffHealth { amount: _ }
        | ModifierEffect::ReduceManaCost { amount: _ } => {}
        ModifierEffect::GrantTrigger(trigger) => collect_trigger(sequences, trigger),
    }
}

fn collect_trigger<'a>(sequences: &mut Vec<&'a [Effect<NoTarget>]>, trigger: &'a TriggerDef) {
    let TriggerDef {
        pattern: _,
        effects,
    } = trigger;
    sequences.push(effects);
    collect_grants(sequences, effects);
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::definition::{CharacterSelector, DefId, PlayerSelector};
    use crate::target::Targets;
    use crate::trigger::{TriggerPattern, TriggerSubject};

    fn spell() -> CardDef {
        CardDef {
            id: DefId::new("test.spell.v0"),
            name: "Spell".to_string(),
            kind: CardKindDef::Spell(SpellCardDef),
            mana_cost: 1,
            modifier_effects: Vec::new(),
            play_effects: Vec::new(),
            triggers: Vec::new(),
            targets: Targets::default(),
        }
    }

    fn minion(
        friendly_aura_effects: Vec<ModifierEffect>,
        hostile_aura_effects: Vec<ModifierEffect>,
    ) -> CardDef {
        CardDef {
            kind: CardKindDef::Minion(MinionCardDef {
                attack: 1,
                health: 1,
                friendly_aura_effects,
                hostile_aura_effects,
            }),
            ..spell()
        }
    }

    fn draw<T>() -> Effect<T> {
        Effect::Draw {
            selector: PlayerSelector::Owner,
            amount: 1.into(),
        }
    }

    fn trigger(effects: Vec<Effect<NoTarget>>) -> TriggerDef {
        TriggerDef {
            pattern: TriggerPattern::Died(TriggerSubject::Itself),
            effects,
        }
    }

    fn grant(effects: Vec<Effect<NoTarget>>) -> ModifierEffect {
        ModifierEffect::GrantTrigger(trigger(effects))
    }

    #[test]
    fn play_effects_are_no_trigger_sequence() {
        let def = CardDef {
            play_effects: vec![draw()],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), Vec::<&[Effect<NoTarget>]>::new());
    }

    #[test]
    fn a_printed_trigger_is_a_trigger_sequence() {
        let def = CardDef {
            triggers: vec![trigger(vec![draw()])],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_by_a_modifier_effect_is_a_trigger_sequence() {
        let def = CardDef {
            modifier_effects: vec![grant(vec![draw()])],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_by_a_friendly_aura_is_a_trigger_sequence() {
        let def = minion(vec![grant(vec![draw()])], Vec::new());

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_by_a_hostile_aura_is_a_trigger_sequence() {
        let def = minion(Vec::new(), vec![grant(vec![draw()])]);

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_by_a_buff_is_a_trigger_sequence() {
        let buff = Effect::Buff {
            selector: CharacterSelector::Itself,
            effect: grant(vec![draw()]),
        };
        let def = CardDef {
            play_effects: vec![buff],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_by_an_added_aura_is_a_trigger_sequence() {
        let aura = Effect::AddFriendlyAura {
            selector: CharacterSelector::Itself,
            effect: grant(vec![draw()]),
        };
        let def = CardDef {
            play_effects: vec![aura],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_inside_a_granted_trigger_is_a_trigger_sequence() {
        let buff = Effect::Buff {
            selector: CharacterSelector::Itself,
            effect: grant(vec![draw()]),
        };
        let def = CardDef {
            modifier_effects: vec![grant(vec![buff.clone()])],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[buff][..], &[draw()][..]]);
    }

    #[test]
    fn a_modifier_effect_that_grants_no_trigger_holds_no_sequence() {
        let buff = Effect::Buff {
            selector: CharacterSelector::Itself,
            effect: ModifierEffect::BuffHealth { amount: 1.into() },
        };
        let def = CardDef {
            modifier_effects: vec![ModifierEffect::ReduceManaCost { amount: 1.into() }],
            play_effects: vec![buff],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), Vec::<&[Effect<NoTarget>]>::new());
    }
}
