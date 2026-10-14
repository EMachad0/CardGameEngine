use crate::{
    PickId,
    cards::{
        binder::base::{PLACEHOLDER, static_card_definition::ALL_DEF_ID},
        definition::{CardDef, CardDefKind, CharacterSelector, DefId, MinionCardDef, TriggerDef},
        modifier::ModifierEffect,
    },
    choice::ChoiceId,
    effect::Effect,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum CardDefError {
    #[error("Card definition not found {0:?}")]
    NotFound(DefId),
    #[error("Duplicate {0:?}")]
    Duplicate(DefId),
    #[error("Placeholder card def present")]
    Placeholder,
    #[error("Duplicate choice {1:?} on {0:?}")]
    DuplicateChoice(DefId, ChoiceId),
    #[error("Choice {1:?} not found on {0:?}")]
    ChoiceNotFound(DefId, ChoiceId),
    #[error("Choice {1:?} used by a trigger on {0:?}")]
    ChoiceInTrigger(DefId, ChoiceId),
    #[error("Pick {1:?} resolved with no reveal before it on {0:?}")]
    PickNotFound(DefId, PickId),
    #[error("Pick {1:?} revealed and never resolved on {0:?}")]
    PickNotResolved(DefId, PickId),
}

pub(super) fn validate(defs: &[CardDef]) -> Result<(), CardDefError> {
    validate_placeholder(defs)?;
    validate_duplicates(defs)?;
    validate_not_found(defs)?;
    validate_choice_duplicates(defs)?;
    validate_choice_not_found(defs)?;
    validate_choice_on_trigger(defs)?;
    validate_picks(defs)?;
    Ok(())
}

fn validate_placeholder(defs: &[CardDef]) -> Result<(), CardDefError> {
    match defs.iter().any(|def| def.id == PLACEHOLDER) {
        true => Err(CardDefError::Placeholder),
        false => Ok(()),
    }
}

fn validate_duplicates(defs: &[CardDef]) -> Result<(), CardDefError> {
    let mut seen = std::collections::BTreeSet::new();
    for def in defs.iter() {
        let def_id = def.id;
        if !seen.insert(def_id) {
            return Err(CardDefError::Duplicate(def_id));
        }
    }
    Ok(())
}

fn validate_not_found(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def_id in ALL_DEF_ID.iter().copied() {
        if !defs.iter().any(|def| def.id == def_id) {
            return Err(CardDefError::NotFound(def_id));
        }
    }
    Ok(())
}

fn validate_choice_duplicates(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for (index, choice) in def.choices.iter().enumerate() {
            if def.choices[..index].iter().any(|seen| seen.id == choice.id) {
                return Err(CardDefError::DuplicateChoice(def.id, choice.id));
            }
        }
    }
    Ok(())
}

fn validate_choice_not_found(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        let effects = def.play_effects.as_slice().iter();
        for choice_id in effects.filter_map(chosen_id) {
            if !def.choices.iter().any(|choice| choice.id == choice_id) {
                return Err(CardDefError::ChoiceNotFound(def.id, choice_id));
            }
        }
    }
    Ok(())
}

fn chosen_id(effect: &Effect) -> Option<ChoiceId> {
    let selector = match effect {
        Effect::Damage { selector, .. }
        | Effect::Buff { selector, .. }
        | Effect::AddFriendlyAura { selector, .. } => selector,
        Effect::Draw { .. }
        | Effect::Reveal { .. }
        | Effect::Summon { .. }
        | Effect::ResolvePick { .. } => return None,
    };
    match selector {
        CharacterSelector::Chosen(choice_id) => Some(*choice_id),
        CharacterSelector::All(_)
        | CharacterSelector::Itself
        | CharacterSelector::OwnerHero
        | CharacterSelector::Random(_) => None,
    }
}

fn validate_choice_on_trigger(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for trigger in def.triggers.iter() {
            if let Some(choice_id) = choice_on_trigger(trigger) {
                return Err(CardDefError::ChoiceInTrigger(def.id, choice_id));
            }
        }

        if let CardDefKind::Minion(MinionCardDef {
            friendly_aura_effects,
            hostile_aura_effects,
            ..
        }) = &def.kind
        {
            let auras = friendly_aura_effects
                .iter()
                .chain(hostile_aura_effects.iter());
            for modifier_effect in auras {
                if let ModifierEffect::GrantTrigger(trigger) = modifier_effect
                    && let Some(choice_id) = choice_on_trigger(trigger)
                {
                    return Err(CardDefError::ChoiceInTrigger(def.id, choice_id));
                }
            }
        }
    }
    Ok(())
}

fn choice_on_trigger(trigger: &TriggerDef) -> Option<ChoiceId> {
    trigger.effects.as_slice().iter().find_map(chosen_id)
}

fn validate_picks(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        validate_pick_order(def.id, def.play_effects.as_slice())?;
        for trigger in def.triggers.iter() {
            validate_pick_order(def.id, trigger.effects.as_slice())?;
        }
        if let CardDefKind::Minion(MinionCardDef {
            friendly_aura_effects,
            hostile_aura_effects,
            ..
        }) = &def.kind
        {
            for modifier_effect in friendly_aura_effects.iter().chain(hostile_aura_effects) {
                if let ModifierEffect::GrantTrigger(trigger) = modifier_effect {
                    validate_pick_order(def.id, trigger.effects.as_slice())?;
                }
            }
        }
    }
    Ok(())
}

fn validate_pick_order(def_id: DefId, effects: &[Effect]) -> Result<(), CardDefError> {
    let mut unresolved = Vec::new();
    for effect in effects {
        match effect {
            Effect::Reveal { pick_id, .. } => unresolved.push(*pick_id),
            Effect::ResolvePick { pick_id, .. } => {
                let at = unresolved
                    .iter()
                    .position(|revealed| revealed == pick_id)
                    .ok_or(CardDefError::PickNotFound(def_id, *pick_id))?;
                unresolved.remove(at);
            }
            Effect::Draw { .. }
            | Effect::Damage { .. }
            | Effect::Buff { .. }
            | Effect::AddFriendlyAura { .. }
            | Effect::Summon { .. } => {}
        }
    }
    match unresolved.first() {
        Some(&pick_id) => Err(CardDefError::PickNotResolved(def_id, pick_id)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use pretty_assertions::assert_eq;

    use crate::{
        cards::binder::base,
        effect::EffectSequence,
        static_card_definition::*,
        trigger::{TriggerPattern, TriggerSubject},
    };

    #[test]
    fn the_built_in_table_is_valid() {
        assert_eq!(validate(&base::definitions()), Ok(()));
    }

    #[test]
    fn a_constant_missing_from_the_table_is_not_found() {
        let mut defs = base::definitions();
        defs.retain(|def| def.id != SPARK);

        assert_eq!(validate(&defs), Err(CardDefError::NotFound(SPARK)));
    }

    #[test]
    fn two_defs_with_one_id_are_a_duplicate() {
        let mut defs = base::definitions();
        let bolt = defs.iter().find(|def| def.id == BOLT).unwrap().clone();
        defs.push(bolt);

        assert_eq!(validate(&defs), Err(CardDefError::Duplicate(BOLT)));
    }

    #[test]
    fn a_def_left_with_the_placeholder_id_is_rejected() {
        let mut defs = base::definitions();
        defs.push(CardDef::default());

        assert_eq!(
            validate(&defs),
            Err(CardDefError::Placeholder),
            "a literal that forgot its id falls back to the placeholder"
        );
    }

    #[test]
    fn two_choices_with_one_id_on_a_card_are_a_duplicate() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.choices.push(ping.choices[0]);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::DuplicateChoice(PING, ChoiceId(0)))
        );
    }

    #[test]
    fn an_effect_naming_an_undeclared_choice_is_not_found() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.choices.clear();

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceNotFound(PING, ChoiceId(0)))
        );
    }

    fn chosen_damage_trigger() -> TriggerDef {
        TriggerDef {
            pattern: TriggerPattern::Died(TriggerSubject::Itself),
            effects: EffectSequence(vec![Effect::Damage {
                selector: CharacterSelector::Chosen(ChoiceId(0)),
                amount: 1.into(),
            }]),
        }
    }

    #[test]
    fn a_trigger_effect_naming_a_choice_is_rejected() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.triggers.push(chosen_damage_trigger());

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceInTrigger(PING, ChoiceId(0))),
            "a trigger resolves with no draft, even on a card that declares the choice"
        );
    }

    #[test]
    fn a_granted_trigger_effect_naming_a_choice_is_rejected() {
        let mut defs = base::definitions();
        let ping_choice = defs.iter().find(|def| def.id == PING).unwrap().choices[0];
        let banner = defs.iter_mut().find(|def| def.id == RALLY_BANNER).unwrap();
        banner.choices.push(ping_choice);
        let CardDefKind::Minion(minion) = &mut banner.kind else {
            panic!("Rally Banner is a minion");
        };
        minion.friendly_aura_effects = vec![ModifierEffect::GrantTrigger(chosen_damage_trigger())];

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceInTrigger(RALLY_BANNER, ChoiceId(0)))
        );
    }

    fn play_effects_of(defs: &mut [CardDef], def_id: DefId) -> &mut Vec<Effect> {
        &mut defs
            .iter_mut()
            .find(|def| def.id == def_id)
            .unwrap()
            .play_effects
            .0
    }

    #[test]
    fn a_pick_resolved_with_no_reveal_is_not_found() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, FORAGE).remove(0);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotFound(FORAGE, PickId(0)))
        );
    }

    #[test]
    fn a_pick_resolved_before_its_reveal_is_not_found() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, FORAGE).swap(0, 1);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotFound(FORAGE, PickId(0))),
            "resolving first would find no pick to move"
        );
    }

    #[test]
    fn a_pick_revealed_and_never_resolved_is_rejected() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, FORAGE).truncate(1);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(FORAGE, PickId(0))),
            "the revealed cards would be left in no zone"
        );
    }

    #[test]
    fn a_triggers_pick_must_be_resolved_too() {
        let mut defs = base::definitions();
        let hoarder = defs.iter_mut().find(|def| def.id == HOARDER).unwrap();
        hoarder.triggers[0].effects.0.truncate(1);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(HOARDER, PickId(0)))
        );
    }

    #[test]
    fn a_granted_triggers_pick_must_be_resolved_too() {
        let mut defs = base::definitions();
        let mut deathrattle =
            defs.iter().find(|def| def.id == HOARDER).unwrap().triggers[0].clone();
        deathrattle.effects.0.truncate(1);
        let banner = defs.iter_mut().find(|def| def.id == RALLY_BANNER).unwrap();
        let CardDefKind::Minion(minion) = &mut banner.kind else {
            panic!("Rally Banner is a minion");
        };
        minion.friendly_aura_effects = vec![ModifierEffect::GrantTrigger(deathrattle)];

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(RALLY_BANNER, PickId(0)))
        );
    }
}
