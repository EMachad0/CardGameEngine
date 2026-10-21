use crate::cards::binder::sets::base::PLACEHOLDER;
use crate::cards::binder::sets::base::static_card_definition::ALL_DEF_ID;
use crate::cards::definition::{CardDef, CharacterSelector, DefId};
use crate::cards::sequences::{SequenceOrigin, effect_sequences};
use crate::effect::{Effect, PickId};
use crate::target::TargetId;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(crate) enum CardDefError {
    #[error("Card definition not found {0:?}")]
    NotFound(DefId),
    #[error("Duplicate {0:?}")]
    Duplicate(DefId),
    #[error("Placeholder card def present")]
    Placeholder,
    #[error("Duplicate target {1:?} on {0:?}")]
    DuplicateTarget(DefId, TargetId),
    #[error("Target {1:?} not found on {0:?}")]
    TargetNotFound(DefId, TargetId),
    #[error("Target {1:?} used by a trigger on {0:?}")]
    TargetInTrigger(DefId, TargetId),
    #[error("Pick {1:?} resolved with no reveal before it on {0:?}")]
    PickNotFound(DefId, PickId),
    #[error("Pick {1:?} revealed and never resolved on {0:?}")]
    PickNotResolved(DefId, PickId),
}

pub(super) fn validate(defs: &[CardDef]) -> Result<(), CardDefError> {
    validate_placeholder(defs)?;
    validate_duplicates(defs)?;
    validate_not_found(defs)?;
    validate_target_duplicates(defs)?;
    validate_target_not_found(defs)?;
    validate_target_on_trigger(defs)?;
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

fn validate_target_duplicates(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for (index, target) in def.targets.iter().enumerate() {
            if def.targets[..index].iter().any(|seen| seen.id == target.id) {
                return Err(CardDefError::DuplicateTarget(def.id, target.id));
            }
        }
    }
    Ok(())
}

/// Rejects a play effect that names a target the card doesn't declare.
fn validate_target_not_found(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        let effects = effect_sequences(def)
            .into_iter()
            .filter(|sequence| resolves_with_targeting(sequence.origin))
            .flat_map(|sequence| sequence.effects);
        for target_id in effects.filter_map(targeted_id) {
            if def.target(target_id).is_none() {
                return Err(CardDefError::TargetNotFound(def.id, target_id));
            }
        }
    }
    Ok(())
}

fn targeted_id(effect: &Effect) -> Option<TargetId> {
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
        CharacterSelector::Targeted(target_id) => Some(*target_id),
        CharacterSelector::All(_)
        | CharacterSelector::Itself
        | CharacterSelector::OwnerHero
        | CharacterSelector::Random(_) => None,
    }
}

/// Rejects a trigger effect that names a target, whether the trigger is printed or granted. A
/// trigger resolves with no targeting to read the target from.
fn validate_target_on_trigger(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        let mut effects = effect_sequences(def)
            .into_iter()
            .filter(|sequence| !resolves_with_targeting(sequence.origin))
            .flat_map(|sequence| sequence.effects);
        if let Some(target_id) = effects.find_map(targeted_id) {
            return Err(CardDefError::TargetInTrigger(def.id, target_id));
        }
    }
    Ok(())
}

/// Whether a sequence resolves with what the card's targeting targeted.
fn resolves_with_targeting(origin: SequenceOrigin) -> bool {
    match origin {
        SequenceOrigin::Play => true,
        SequenceOrigin::Trigger => false,
    }
}

/// Rejects a card whose effect sequences, triggers included, resolve a pick with no `Reveal`
/// before it, or reveal cards they never resolve.
fn validate_picks(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for sequence in effect_sequences(def) {
            validate_pick_order(def.id, sequence.effects)?;
        }
    }
    Ok(())
}

/// Pairs each `ResolvePick` with an earlier `Reveal` of the same pick. Revealed cards sit in no
/// zone until their pick resolves.
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
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::sets::base;
    use crate::cards::binder::sets::base::static_card_definition::*;
    use crate::cards::definition::{CardKindDef, TriggerDef};
    use crate::cards::modifier::ModifierEffect;
    use crate::effect::EffectSequence;
    use crate::trigger::{TriggerPattern, TriggerSubject};

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
    fn two_targets_with_one_id_on_a_card_are_a_duplicate() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.targets.push(ping.targets[0]);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::DuplicateTarget(PING, TargetId(0)))
        );
    }

    #[test]
    fn an_effect_naming_an_undeclared_target_is_not_found() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.targets.clear();

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetNotFound(PING, TargetId(0)))
        );
    }

    fn targeted_damage_trigger() -> TriggerDef {
        TriggerDef {
            pattern: TriggerPattern::Died(TriggerSubject::Itself),
            effects: EffectSequence(vec![Effect::Damage {
                selector: CharacterSelector::Targeted(TargetId(0)),
                amount: 1.into(),
            }]),
        }
    }

    fn replace_rally_banner_friendly_auras(defs: &mut [CardDef], auras: Vec<ModifierEffect>) {
        let banner = defs.iter_mut().find(|def| def.id == RALLY_BANNER).unwrap();
        let CardKindDef::Minion(minion) = &mut banner.kind else {
            panic!("Rally Banner is a minion");
        };
        minion.friendly_aura_effects = auras;
    }

    #[test]
    fn a_trigger_effect_naming_a_target_is_rejected() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.triggers.push(targeted_damage_trigger());

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetInTrigger(PING, TargetId(0))),
            "a trigger resolves with no targeting, even on a card that declares the target"
        );
    }

    #[test]
    fn a_granted_trigger_effect_naming_a_target_is_rejected() {
        let mut defs = base::definitions();
        let ping_target = defs.iter().find(|def| def.id == PING).unwrap().targets[0];
        let banner = defs.iter_mut().find(|def| def.id == RALLY_BANNER).unwrap();
        banner.targets.push(ping_target);
        replace_rally_banner_friendly_auras(
            &mut defs,
            vec![ModifierEffect::GrantTrigger(targeted_damage_trigger())],
        );

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetInTrigger(RALLY_BANNER, TargetId(0)))
        );
    }

    #[test]
    fn a_trigger_granted_by_a_modifier_effect_naming_a_target_is_rejected() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.modifier_effects
            .push(ModifierEffect::GrantTrigger(targeted_damage_trigger()));

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetInTrigger(PING, TargetId(0)))
        );
    }

    #[test]
    fn a_trigger_granted_by_a_buff_naming_a_target_is_rejected() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, PING).push(Effect::Buff {
            selector: CharacterSelector::OwnerHero,
            effect: ModifierEffect::GrantTrigger(targeted_damage_trigger()),
        });

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetInTrigger(PING, TargetId(0))),
            "the buff resolves during play, but the trigger it grants resolves with no targeting"
        );
    }

    #[test]
    fn a_trigger_granted_by_an_added_aura_naming_a_target_is_rejected() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, PING).push(Effect::AddFriendlyAura {
            selector: CharacterSelector::OwnerHero,
            effect: ModifierEffect::GrantTrigger(targeted_damage_trigger()),
        });

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetInTrigger(PING, TargetId(0)))
        );
    }

    #[test]
    fn a_trigger_granted_inside_a_granted_trigger_naming_a_target_is_rejected() {
        let mut defs = base::definitions();
        let ping_target = defs.iter().find(|def| def.id == PING).unwrap().targets[0];
        let banner = defs.iter_mut().find(|def| def.id == RALLY_BANNER).unwrap();
        banner.targets.push(ping_target);
        replace_rally_banner_friendly_auras(
            &mut defs,
            vec![ModifierEffect::GrantTrigger(TriggerDef {
                pattern: TriggerPattern::Damaged(TriggerSubject::Itself),
                effects: EffectSequence(vec![Effect::Buff {
                    selector: CharacterSelector::Itself,
                    effect: ModifierEffect::GrantTrigger(targeted_damage_trigger()),
                }]),
            })],
        );

        assert_eq!(
            validate(&defs),
            Err(CardDefError::TargetInTrigger(RALLY_BANNER, TargetId(0)))
        );
    }

    #[test]
    fn a_buff_in_play_effects_may_target_whom_it_grants_a_trigger_to() {
        let mut defs = base::definitions();
        let deathrattle = defs.iter().find(|def| def.id == HOARDER).unwrap().triggers[0].clone();
        play_effects_of(&mut defs, PING).push(Effect::Buff {
            selector: CharacterSelector::Targeted(TargetId(0)),
            effect: ModifierEffect::GrantTrigger(deathrattle),
        });

        assert_eq!(
            validate(&defs),
            Ok(()),
            "the buff's own selector resolves during play, where its target is named"
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

    fn hoarder_trigger_with_an_unresolved_pick(defs: &[CardDef]) -> TriggerDef {
        let mut trigger = defs.iter().find(|def| def.id == HOARDER).unwrap().triggers[0].clone();
        trigger.effects.0.truncate(1);
        trigger
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
        let deathrattle = hoarder_trigger_with_an_unresolved_pick(&defs);
        replace_rally_banner_friendly_auras(
            &mut defs,
            vec![ModifierEffect::GrantTrigger(deathrattle)],
        );

        assert_eq!(
            validate(&defs),
            Err(CardDefError::PickNotResolved(RALLY_BANNER, PickId(0)))
        );
    }

    #[test]
    fn a_pick_in_a_trigger_granted_by_a_buff_must_be_resolved_too() {
        let mut defs = base::definitions();
        let deathrattle = hoarder_trigger_with_an_unresolved_pick(&defs);
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
