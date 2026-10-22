use crate::cards::definition::CardDef;
use crate::cards::sequences::trigger_sequences;
use crate::effect::{Effect, PickId};
use crate::target::{DeclaredTarget, TargetId};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub(super) enum CardDefError {
    #[error("Target {0:?} not found")]
    TargetNotFound(TargetId),
    #[error("Pick {0:?} resolved with no reveal before it")]
    PickNotFound(PickId),
    #[error("Pick {0:?} revealed and never resolved")]
    PickNotResolved(PickId),
}

/// The card, with each target its play effects name declared. Fails on the first target the card
/// doesn't declare, then checks the card's picks.
pub(super) fn parse(def: CardDef<TargetId>) -> Result<CardDef<DeclaredTarget>, CardDefError> {
    let CardDef {
        name,
        kind,
        mana_cost,
        modifier_effects,
        play_effects,
        triggers,
        targets,
    } = def;
    let play_effects = play_effects
        .into_iter()
        .map(|effect| {
            effect.try_map_targeted(|target_id| {
                targets
                    .declared(target_id)
                    .ok_or(CardDefError::TargetNotFound(target_id))
            })
        })
        .collect::<Result<_, _>>()?;
    let def = CardDef {
        name,
        kind,
        mana_cost,
        modifier_effects,
        play_effects,
        triggers,
        targets,
    };
    validate_picks(&def)?;
    Ok(def)
}

/// Rejects a card whose effect sequences, triggers included, resolve a pick with no `Reveal`
/// before it, or reveal cards they never resolve.
fn validate_picks<T>(def: &CardDef<T>) -> Result<(), CardDefError> {
    validate_pick_order(&def.play_effects)?;
    for sequence in trigger_sequences(def) {
        validate_pick_order(sequence)?;
    }
    Ok(())
}

/// Pairs each `ResolvePick` with an earlier `Reveal` of the same pick. Revealed cards sit in no
/// zone until their pick resolves.
fn validate_pick_order<T>(effects: &[Effect<T>]) -> Result<(), CardDefError> {
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
            Effect::Draw { .. }
            | Effect::Damage { .. }
            | Effect::Buff { .. }
            | Effect::AddFriendlyAura { .. }
            | Effect::Summon { .. } => {}
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
    use crate::target::Targets;

    fn validate(def: CardDef<TargetId>) -> Result<(), CardDefError> {
        parse(def).map(drop)
    }

    fn built_in_card(def_id: DefId) -> CardDef<TargetId> {
        base::definitions()
            .into_iter()
            .find_map(|(id, def)| (id == def_id).then_some(def))
            .unwrap()
    }

    #[test]
    fn each_play_effect_holds_the_target_declared_for_its_id() {
        let shove = parse(built_in_card(SHOVE)).unwrap();

        let targeted = shove
            .play_effects
            .iter()
            .map(|effect| match effect {
                Effect::Damage {
                    selector: CharacterSelector::Targeted(target),
                    ..
                } => target.id(),
                _ => panic!("each of Shove's effects must damage a targeted character"),
            })
            .collect::<Vec<_>>();
        assert_eq!(targeted, [TargetId(0), TargetId(1)]);
    }

    #[test]
    fn an_effect_naming_an_undeclared_target_is_not_found() {
        let mut ping = built_in_card(PING);
        ping.targets = Targets::default();

        assert_eq!(
            validate(ping),
            Err(CardDefError::TargetNotFound(TargetId(0)))
        );
    }

    fn replace_friendly_auras(def: &mut CardDef<TargetId>, auras: Vec<ModifierEffect>) {
        let CardKindDef::Minion(minion) = &mut def.kind else {
            panic!("the card must be a minion");
        };
        minion.friendly_aura_effects = auras;
    }

    #[test]
    fn a_buff_in_play_effects_may_target_whom_it_grants_a_trigger_to() {
        let death_rattle = built_in_card(HOARDER).triggers[0].clone();
        let mut ping = built_in_card(PING);
        ping.play_effects.push(Effect::Buff {
            selector: CharacterSelector::Targeted(TargetId(0)),
            effect: ModifierEffect::GrantTrigger(death_rattle),
        });

        assert_eq!(
            validate(ping),
            Ok(()),
            "the buff's own selector resolves during play, where its target is named"
        );
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

        assert_eq!(validate(forage), Err(CardDefError::PickNotFound(PickId(0))));
    }

    #[test]
    fn a_pick_resolved_before_its_reveal_is_not_found() {
        let mut forage = built_in_card(FORAGE);
        forage.play_effects.swap(0, 1);

        assert_eq!(
            validate(forage),
            Err(CardDefError::PickNotFound(PickId(0))),
            "resolving first would find no pick to move"
        );
    }

    #[test]
    fn a_pick_revealed_and_never_resolved_is_rejected() {
        let mut forage = built_in_card(FORAGE);
        forage.play_effects.truncate(1);

        assert_eq!(
            validate(forage),
            Err(CardDefError::PickNotResolved(PickId(0))),
            "the revealed cards would be left in no zone"
        );
    }

    #[test]
    fn a_triggers_pick_must_be_resolved_too() {
        let mut hoarder = built_in_card(HOARDER);
        hoarder.triggers[0].effects.truncate(1);

        assert_eq!(
            validate(hoarder),
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
            validate(banner),
            Err(CardDefError::PickNotResolved(PickId(0)))
        );
    }

    #[test]
    fn a_pick_in_a_trigger_granted_by_a_buff_must_be_resolved_too() {
        let mut drummer = built_in_card(DRUMMER);
        drummer.play_effects.push(Effect::Buff {
            selector: CharacterSelector::Itself,
            effect: ModifierEffect::GrantTrigger(hoarder_trigger_with_an_unresolved_pick()),
        });

        assert_eq!(
            validate(drummer),
            Err(CardDefError::PickNotResolved(PickId(0)))
        );
    }
}
