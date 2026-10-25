use crate::cards::definition::{
    CardDef, CardKindDef, HeroCardDef, MinionCardDef, SpellCardDef, TriggerDef,
};
use crate::cards::modifier::ModifierEffect;
use crate::effect::{CharacterVerb, Effect, PlayerVerb};
use crate::target::{TargetForm, Triggered};

/// The effects of every trigger on a card, printed or granted through modifier effects, auras,
/// `Modify` and `AddFriendlyAura`, at any depth. Those its play effects grant come first.
pub(super) fn trigger_sequences(def: &CardDef) -> Vec<&[Effect<Triggered>]> {
    let CardDef {
        name: _,
        kind,
        mana_cost: _,
        modifier_effects,
        play_effects,
        triggers,
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

fn collect_grants<'a, S: TargetForm>(
    sequences: &mut Vec<&'a [Effect<Triggered>]>,
    effects: &'a [Effect<S>],
) {
    for effect in effects {
        match effect {
            Effect::Players { selector: _, verbs } => {
                for verb in verbs {
                    match verb {
                        PlayerVerb::Draw(_) | PlayerVerb::Summon(_) => {}
                    }
                }
            }
            Effect::Characters { selector: _, verbs } => {
                for verb in verbs {
                    match verb {
                        CharacterVerb::Damage(_) => {}
                        CharacterVerb::Modify(modifier_effect)
                        | CharacterVerb::AddFriendlyAura(modifier_effect) => {
                            collect_modifier(sequences, modifier_effect)
                        }
                    }
                }
            }
            Effect::Reveal {
                pick_id: _,
                selector: _,
                amount: _,
            }
            | Effect::ResolvePick {
                pick_id: _,
                picked_move: _,
                other_move: _,
            }
            | Effect::Cards {
                selector: _,
                verbs: _,
            } => {}
        }
    }
}

fn collect_modifier<'a>(
    sequences: &mut Vec<&'a [Effect<Triggered>]>,
    modifier_effect: &'a ModifierEffect,
) {
    match modifier_effect {
        ModifierEffect::BuffAtk { amount: _ }
        | ModifierEffect::BuffHealth { amount: _ }
        | ModifierEffect::ReduceManaCost { amount: _ } => {}
        ModifierEffect::GrantTrigger(trigger) => collect_trigger(sequences, trigger),
    }
}

fn collect_trigger<'a>(sequences: &mut Vec<&'a [Effect<Triggered>]>, trigger: &'a TriggerDef) {
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
    use crate::cards::binder::sets::base::static_card_definition::SQUIRE;
    use crate::cards::definition::{CharacterSelector, PlayerSelector};
    use crate::trigger::{TriggerPattern, TriggerSubject};

    fn spell() -> CardDef {
        CardDef {
            name: "Spell".to_string(),
            kind: CardKindDef::Spell(SpellCardDef),
            mana_cost: 1,
            modifier_effects: Vec::new(),
            play_effects: Vec::new(),
            triggers: Vec::new(),
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

    fn draw<S: TargetForm>() -> Effect<S> {
        Effect::Players {
            selector: PlayerSelector::Owner,
            verbs: vec![PlayerVerb::Draw(1.into())],
        }
    }

    fn trigger(effects: Vec<Effect<Triggered>>) -> TriggerDef {
        TriggerDef {
            pattern: TriggerPattern::Died(TriggerSubject::Itself),
            effects,
        }
    }

    fn grant(effects: Vec<Effect<Triggered>>) -> ModifierEffect {
        ModifierEffect::GrantTrigger(trigger(effects))
    }

    #[test]
    fn play_effects_are_not_a_trigger_sequence() {
        let def = CardDef {
            play_effects: vec![draw()],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), Vec::<&[Effect<Triggered>]>::new());
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
    fn a_trigger_granted_by_modify_is_a_trigger_sequence() {
        let modify = Effect::Characters {
            selector: CharacterSelector::Itself,
            verbs: vec![CharacterVerb::Modify(grant(vec![draw()]))],
        };
        let def = CardDef {
            play_effects: vec![modify],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_by_an_added_aura_is_a_trigger_sequence() {
        let aura = Effect::Characters {
            selector: CharacterSelector::Itself,
            verbs: vec![CharacterVerb::AddFriendlyAura(grant(vec![draw()]))],
        };
        let def = CardDef {
            play_effects: vec![aura],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[draw()][..]]);
    }

    #[test]
    fn a_trigger_granted_inside_a_granted_trigger_is_a_trigger_sequence() {
        let modify = Effect::Characters {
            selector: CharacterSelector::Itself,
            verbs: vec![CharacterVerb::Modify(grant(vec![draw()]))],
        };
        let def = CardDef {
            modifier_effects: vec![grant(vec![modify.clone()])],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), vec![&[modify][..], &[draw()][..]]);
    }

    #[test]
    fn each_verb_of_an_effect_can_grant_a_trigger_sequence() {
        let summon = Effect::Players {
            selector: PlayerSelector::Owner,
            verbs: vec![PlayerVerb::Summon(SQUIRE)],
        };
        let def = CardDef {
            play_effects: vec![Effect::Characters {
                selector: CharacterSelector::Itself,
                verbs: vec![
                    CharacterVerb::Damage(1.into()),
                    CharacterVerb::Modify(grant(vec![draw()])),
                    CharacterVerb::AddFriendlyAura(grant(vec![summon.clone()])),
                ],
            }],
            ..spell()
        };

        assert_eq!(
            trigger_sequences(&def),
            vec![&[draw()][..], &[summon][..]],
            "the second and third verbs each grant one",
        );
    }

    #[test]
    fn a_modifier_effect_that_grants_no_trigger_holds_no_sequence() {
        let buff = Effect::Characters {
            selector: CharacterSelector::Itself,
            verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffHealth {
                amount: 1.into(),
            })],
        };
        let def = CardDef {
            modifier_effects: vec![ModifierEffect::ReduceManaCost { amount: 1.into() }],
            play_effects: vec![buff],
            ..spell()
        };

        assert_eq!(trigger_sequences(&def), Vec::<&[Effect<Triggered>]>::new());
    }
}
