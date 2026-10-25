use crate::cards::definition::CardDef;
use crate::cards::sequences::trigger_sequences;
use crate::effect::{Effect, PickId};
use crate::target::TargetForm;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(super) enum CardDefError {
    #[error("Pick {0:?} resolved with no reveal before it")]
    PickNotFound(PickId),
    #[error("Pick {0:?} revealed and never resolved")]
    PickNotResolved(PickId),
}

/// Rejects a card whose effect sequences, triggers included, resolve a pick with no `Reveal`
/// before it, or reveal cards they never resolve.
pub(super) fn validate(def: &CardDef) -> Result<(), CardDefError> {
    validate_pick_order(&def.play_effects)?;
    for sequence in trigger_sequences(def) {
        validate_pick_order(sequence)?;
    }
    Ok(())
}

/// Pairs each `ResolvePick` with an earlier `Reveal` of the same pick. Revealed cards sit in no
/// zone until their pick resolves.
fn validate_pick_order<S: TargetForm>(effects: &[Effect<S>]) -> Result<(), CardDefError> {
    let mut unresolved = Vec::new();
    for effect in effects {
        match effect {
            Effect::Reveal { pick_id, .. } => unresolved.push(*pick_id),
            Effect::ResolvePick { pick_id, .. } => {
                let at = unresolved
                    .iter()
                    .position(|revealed| revealed == pick_id)
                    .ok_or(CardDefError::PickNotFound(*pick_id))?;
                unresolved.remove(at);
            }
            Effect::Players { .. } | Effect::Characters { .. } => {}
            Effect::Cards { .. } => {}
        }
    }
    match unresolved.first() {
        Some(&pick_id) => Err(CardDefError::PickNotResolved(pick_id)),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::cards::binder::def_id::DefId;
    use crate::cards::binder::sets::base;
    use crate::cards::binder::sets::base::static_card_definition::*;
    use crate::cards::definition::{CardKindDef, CharacterSelector, TriggerDef};
    use crate::cards::modifier::ModifierEffect;
    use crate::effect::CharacterVerb;

    fn built_in_card(def_id: DefId) -> CardDef {
        base::definitions()
            .into_iter()
            .find_map(|(id, def)| (id == def_id).then_some(def))
            .unwrap()
    }

    fn replace_friendly_auras(def: &mut CardDef, auras: Vec<ModifierEffect>) {
        let CardKindDef::Minion(minion) = &mut def.kind else {
            panic!("the card must be a minion");
        };
        minion.friendly_aura_effects = auras;
    }

    fn hoarder_trigger_with_an_unresolved_pick() -> TriggerDef {
        let mut trigger = built_in_card(HOARDER).triggers[0].clone();
        trigger.effects.truncate(1);
        trigger
    }

    #[test]
    fn a_pick_resolved_with_no_reveal_is_not_found() {
        let mut forage = built_in_card(FORAGE);
        forage.play_effects.remove(0);

        assert_eq!(
            validate(&forage),
            Err(CardDefError::PickNotFound(PickId(0)))
        );
    }

    #[test]
    fn a_pick_resolved_before_its_reveal_is_not_found() {
        let mut forage = built_in_card(FORAGE);
        forage.play_effects.swap(0, 1);

        assert_eq!(
            validate(&forage),
            Err(CardDefError::PickNotFound(PickId(0))),
            "resolving first would find no pick to move"
        );
    }

    #[test]
    fn a_pick_revealed_and_never_resolved_is_rejected() {
        let mut forage = built_in_card(FORAGE);
        forage.play_effects.truncate(1);

        assert_eq!(
            validate(&forage),
            Err(CardDefError::PickNotResolved(PickId(0))),
            "the revealed cards would be left in no zone"
        );
    }

    #[test]
    fn a_triggers_pick_must_be_resolved_too() {
        let mut hoarder = built_in_card(HOARDER);
        hoarder.triggers[0].effects.truncate(1);

        assert_eq!(
            validate(&hoarder),
            Err(CardDefError::PickNotResolved(PickId(0)))
        );
    }

    #[test]
    fn a_granted_triggers_pick_must_be_resolved_too() {
        let mut banner = built_in_card(RALLY_BANNER);
        replace_friendly_auras(
            &mut banner,
            vec![ModifierEffect::GrantTrigger(
                hoarder_trigger_with_an_unresolved_pick(),
            )],
        );

        assert_eq!(
            validate(&banner),
            Err(CardDefError::PickNotResolved(PickId(0)))
        );
    }

    #[test]
    fn a_pick_in_a_trigger_granted_by_modify_must_be_resolved_too() {
        let mut drummer = built_in_card(DRUMMER);
        drummer.play_effects.push(Effect::Characters {
            selector: CharacterSelector::Itself,
            verbs: vec![CharacterVerb::Modify(ModifierEffect::GrantTrigger(
                hoarder_trigger_with_an_unresolved_pick(),
            ))],
        });

        assert_eq!(
            validate(&drummer),
            Err(CardDefError::PickNotResolved(PickId(0)))
        );
    }
}
