use crate::cards::{
    choice::ChoiceId,
    def_ids::static_card_definition::ALL_DEF_ID,
    definition::{CardDef, CardDefKind, DefId, PLACEHOLDER},
    effect::{Effect, PickId},
    modifier::ModifierEffect,
    selector::CharacterSelector,
    trigger::TriggerDef,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum CardDefError {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SequenceOrigin {
    Play,
    Trigger,
}

fn effect_sequences(def: &CardDef) -> Vec<(SequenceOrigin, &[Effect])> {
    let mut sequences = Vec::new();
    collect_sequence(
        &mut sequences,
        SequenceOrigin::Play,
        def.play_effects.as_slice(),
    );
    for trigger in &def.triggers {
        collect_trigger(&mut sequences, trigger);
    }
    let (friendly_auras, hostile_auras): (&[ModifierEffect], &[ModifierEffect]) = match &def.kind {
        CardDefKind::Minion(minion) => {
            (&minion.friendly_aura_effects, &minion.hostile_aura_effects)
        }
        CardDefKind::Spell(_) | CardDefKind::Hero(_) => (&[], &[]),
    };
    let modifier_effects = def
        .modifier_effects
        .iter()
        .chain(friendly_auras)
        .chain(hostile_auras);
    for modifier_effect in modifier_effects {
        collect_modifier(&mut sequences, modifier_effect);
    }
    sequences
}

fn collect_sequence<'a>(
    sequences: &mut Vec<(SequenceOrigin, &'a [Effect])>,
    origin: SequenceOrigin,
    effects: &'a [Effect],
) {
    sequences.push((origin, effects));
    for effect in effects {
        match effect {
            Effect::Buff { effect, .. } | Effect::AddFriendlyAura { effect, .. } => {
                collect_modifier(sequences, effect);
            }
            Effect::Draw { .. }
            | Effect::Reveal { .. }
            | Effect::Damage { .. }
            | Effect::Summon { .. }
            | Effect::ResolvePick { .. } => {}
        }
    }
}

fn collect_modifier<'a>(
    sequences: &mut Vec<(SequenceOrigin, &'a [Effect])>,
    modifier_effect: &'a ModifierEffect,
) {
    match modifier_effect {
        ModifierEffect::GrantTrigger(trigger) => collect_trigger(sequences, trigger),
        ModifierEffect::BuffAtk { .. }
        | ModifierEffect::BuffHealth { .. }
        | ModifierEffect::ReduceManaCost { .. } => {}
    }
}

fn collect_trigger<'a>(
    sequences: &mut Vec<(SequenceOrigin, &'a [Effect])>,
    trigger: &'a TriggerDef,
) {
    collect_sequence(
        sequences,
        SequenceOrigin::Trigger,
        trigger.effects.as_slice(),
    );
}

fn effects_from(def: &CardDef, origin: SequenceOrigin) -> impl Iterator<Item = &Effect> {
    effect_sequences(def)
        .into_iter()
        .filter(move |(sequence_origin, _)| *sequence_origin == origin)
        .flat_map(|(_, effects)| effects)
}

fn validate_choice_not_found(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for choice_id in effects_from(def, SequenceOrigin::Play).filter_map(chosen_id) {
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
        if let Some(choice_id) = effects_from(def, SequenceOrigin::Trigger).find_map(chosen_id) {
            return Err(CardDefError::ChoiceInTrigger(def.id, choice_id));
        }
    }
    Ok(())
}

fn validate_picks(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for (_, effects) in effect_sequences(def) {
            validate_pick_order(def.id, effects)?;
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

    use crate::cards::{
        def_ids::static_card_definition::{
            BOLT, DRUMMER, FORAGE, HOARDER, PING, RALLY_BANNER, SPARK,
        },
        effect::EffectSequence,
        loader::CardDefLoader,
        trigger::{TriggerPattern, TriggerSubject},
    };

    #[test]
    fn the_built_in_table_is_valid() {
        assert_eq!(validate(&CardDefLoader.load_all()), Ok(()));
    }

    #[test]
    fn a_constant_missing_from_the_table_is_not_found() {
        let mut defs = CardDefLoader.load_all();
        defs.retain(|def| def.id != SPARK);

        assert_eq!(validate(&defs), Err(CardDefError::NotFound(SPARK)));
    }

    #[test]
    fn two_defs_with_one_id_are_a_duplicate() {
        let mut defs = CardDefLoader.load_all();
        let bolt = defs.iter().find(|def| def.id == BOLT).unwrap().clone();
        defs.push(bolt);

        assert_eq!(validate(&defs), Err(CardDefError::Duplicate(BOLT)));
    }

    #[test]
    fn a_def_left_with_the_placeholder_id_is_rejected() {
        let mut defs = CardDefLoader.load_all();
        defs.push(CardDef::default());

        assert_eq!(
            validate(&defs),
            Err(CardDefError::Placeholder),
            "a literal that forgot its id falls back to the placeholder"
        );
    }

    #[test]
    fn two_choices_with_one_id_on_a_card_are_a_duplicate() {
        let mut defs = CardDefLoader.load_all();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.choices.push(ping.choices[0]);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::DuplicateChoice(PING, ChoiceId(0)))
        );
    }

    #[test]
    fn an_effect_naming_an_undeclared_choice_is_not_found() {
        let mut defs = CardDefLoader.load_all();
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
        let mut defs = CardDefLoader.load_all();
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
        let mut defs = CardDefLoader.load_all();
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

    #[test]
    fn a_trigger_granted_by_a_modifier_effect_naming_a_choice_is_rejected() {
        let mut defs = CardDefLoader.load_all();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.modifier_effects
            .push(ModifierEffect::GrantTrigger(chosen_damage_trigger()));

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceInTrigger(PING, ChoiceId(0)))
        );
    }

    #[test]
    fn a_trigger_granted_by_a_buff_naming_a_choice_is_rejected() {
        let mut defs = CardDefLoader.load_all();
        play_effects_of(&mut defs, PING).push(Effect::Buff {
            selector: CharacterSelector::OwnerHero,
            effect: ModifierEffect::GrantTrigger(chosen_damage_trigger()),
        });

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceInTrigger(PING, ChoiceId(0))),
            "the buff resolves during play, but the trigger it grants resolves with no draft"
        );
    }

    #[test]
    fn a_trigger_granted_by_an_added_aura_naming_a_choice_is_rejected() {
        let mut defs = CardDefLoader.load_all();
        play_effects_of(&mut defs, PING).push(Effect::AddFriendlyAura {
            selector: CharacterSelector::OwnerHero,
            effect: ModifierEffect::GrantTrigger(chosen_damage_trigger()),
        });

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceInTrigger(PING, ChoiceId(0)))
        );
    }

    #[test]
    fn a_trigger_granted_inside_a_granted_trigger_naming_a_choice_is_rejected() {
        let mut defs = CardDefLoader.load_all();
        let ping_choice = defs.iter().find(|def| def.id == PING).unwrap().choices[0];
        let banner = defs.iter_mut().find(|def| def.id == RALLY_BANNER).unwrap();
        banner.choices.push(ping_choice);
        let CardDefKind::Minion(minion) = &mut banner.kind else {
            panic!("Rally Banner is a minion");
        };
        minion.friendly_aura_effects = vec![ModifierEffect::GrantTrigger(TriggerDef {
            pattern: TriggerPattern::Damaged(TriggerSubject::Itself),
            effects: EffectSequence(vec![Effect::Buff {
                selector: CharacterSelector::Itself,
                effect: ModifierEffect::GrantTrigger(chosen_damage_trigger()),
            }]),
        })];

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceInTrigger(RALLY_BANNER, ChoiceId(0)))
        );
    }

    #[test]
    fn a_buff_in_play_effects_may_choose_whom_it_grants_a_trigger_to() {
        let mut defs = CardDefLoader.load_all();
        let deathrattle = defs.iter().find(|def| def.id == HOARDER).unwrap().triggers[0].clone();
        play_effects_of(&mut defs, PING).push(Effect::Buff {
            selector: CharacterSelector::Chosen(ChoiceId(0)),
            effect: ModifierEffect::GrantTrigger(deathrattle),
        });

        assert_eq!(
            validate(&defs),
            Ok(()),
            "the buff's own selector resolves during play, where the choice is drafted"
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
        let mut defs = CardDefLoader.load_all();
        play_effects_of(&mut defs, FORAGE).remove(0);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotFound(FORAGE, PickId(0)))
        );
    }

    #[test]
    fn a_pick_resolved_before_its_reveal_is_not_found() {
        let mut defs = CardDefLoader.load_all();
        play_effects_of(&mut defs, FORAGE).swap(0, 1);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotFound(FORAGE, PickId(0))),
            "resolving first would find no pick to move"
        );
    }

    #[test]
    fn a_pick_revealed_and_never_resolved_is_rejected() {
        let mut defs = CardDefLoader.load_all();
        play_effects_of(&mut defs, FORAGE).truncate(1);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(FORAGE, PickId(0))),
            "the revealed cards would be left in no zone"
        );
    }

    #[test]
    fn a_triggers_pick_must_be_resolved_too() {
        let mut defs = CardDefLoader.load_all();
        let hoarder = defs.iter_mut().find(|def| def.id == HOARDER).unwrap();
        hoarder.triggers[0].effects.0.truncate(1);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(HOARDER, PickId(0)))
        );
    }

    #[test]
    fn a_granted_triggers_pick_must_be_resolved_too() {
        let mut defs = CardDefLoader.load_all();
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

    #[test]
    fn a_pick_in_a_trigger_granted_by_a_buff_must_be_resolved_too() {
        let mut defs = CardDefLoader.load_all();
        let mut deathrattle =
            defs.iter().find(|def| def.id == HOARDER).unwrap().triggers[0].clone();
        deathrattle.effects.0.truncate(1);
        play_effects_of(&mut defs, DRUMMER).push(Effect::Buff {
            selector: CharacterSelector::Itself,
            effect: ModifierEffect::GrantTrigger(deathrattle),
        });

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(DRUMMER, PickId(0)))
        );
    }
}
