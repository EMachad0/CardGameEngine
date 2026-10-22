use std::collections::BTreeMap;

use crate::cards::binder::sets::base::PLACEHOLDER;
use crate::cards::binder::sets::base::static_card_definition::ALL_DEF_ID;
use crate::cards::definition::{
    CardDef, CardKindDef, CharacterSelector, DefId, MinionCardDef, TriggerDef,
};
use crate::cards::modifier::ModifierEffect;
use crate::cards::sequences::effect_sequences;
use crate::effect::{Effect, EffectSequence, PickId};
use crate::target::{CharacterTarget, NoTarget, TargetId};

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

/// Its targets have distinct ids, and each of its effect sequences reveals a pick before resolving
/// it and resolves every pick it reveals.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ValidCardDef {
    kind: CardKindDef<NoTarget>,
    mana_cost: u8,
    modifier_effects: Vec<ModifierEffect<NoTarget>>,
    play_effects: Vec<Effect<DeclaredTarget, NoTarget>>,
    triggers: Vec<TriggerDef<NoTarget>>,
    targets: Vec<CharacterTarget>,
}

impl ValidCardDef {
    pub(crate) fn kind(&self) -> &CardKindDef<NoTarget> {
        &self.kind
    }

    pub(crate) fn mana_cost(&self) -> u8 {
        self.mana_cost
    }

    pub(crate) fn modifier_effects(&self) -> &[ModifierEffect<NoTarget>] {
        &self.modifier_effects
    }

    pub(crate) fn play_effects(&self) -> &[Effect<DeclaredTarget, NoTarget>] {
        &self.play_effects
    }

    pub(crate) fn triggers(&self) -> &[TriggerDef<NoTarget>] {
        &self.triggers
    }

    pub(crate) fn targets(&self) -> &[CharacterTarget] {
        &self.targets
    }
}

/// A target that the card holding it declares.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct DeclaredTarget(TargetId);

impl DeclaredTarget {
    pub(crate) fn target_id(self) -> TargetId {
        self.0
    }
}

/// The definitions by id, or the first error found. Each check runs over every definition, in the
/// order given, before the next one starts, so the order of the checks decides which error a table
/// with several gets.
pub(super) fn parse(defs: &[CardDef]) -> Result<BTreeMap<DefId, ValidCardDef>, CardDefError> {
    validate_placeholder(defs)?;
    validate_duplicates(defs)?;
    validate_not_found(defs)?;
    validate_target_duplicates(defs)?;
    let play_effects = defs
        .iter()
        .map(parse_play_targets)
        .collect::<Result<Vec<_>, _>>()?;
    let valid = defs
        .iter()
        .zip(play_effects)
        .map(|(def, play_effects)| parse_card(def, play_effects))
        .collect::<Result<Vec<_>, _>>()?;
    validate_picks(defs)?;
    Ok(defs.iter().map(|def| def.id).zip(valid).collect())
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

/// The card's play effects, with the targets they name parsed. Rejects a target the card doesn't
/// declare.
fn parse_play_targets(
    def: &CardDef,
) -> Result<Vec<Effect<DeclaredTarget, TargetId>>, CardDefError> {
    let declared = |&target_id: &TargetId| match def.target(target_id) {
        Some(_) => Ok(DeclaredTarget(target_id)),
        None => Err(CardDefError::TargetNotFound(def.id, target_id)),
    };
    def.play_effects
        .as_slice()
        .iter()
        .map(|effect| parse_effect(effect, declared, |granted| Ok(granted.clone())))
        .collect()
}

/// The valid card, from its parsed play effects. Rejects trigger text that names a target, whether
/// the trigger is printed or granted, at any depth.
fn parse_card(
    def: &CardDef,
    play_effects: Vec<Effect<DeclaredTarget, TargetId>>,
) -> Result<ValidCardDef, CardDefError> {
    let CardDef {
        id,
        name: _,
        kind,
        mana_cost,
        modifier_effects,
        play_effects: _,
        triggers,
        targets,
    } = def;
    let play_effects = play_effects
        .iter()
        .map(|effect| parse_grants(*id, effect))
        .collect::<Result<Vec<_>, _>>()?;
    let triggers = triggers
        .iter()
        .map(|trigger| parse_trigger(*id, trigger))
        .collect::<Result<Vec<_>, _>>()?;
    let modifier_effects = parse_modifiers(*id, modifier_effects)?;
    let kind = parse_kind(*id, kind)?;
    Ok(ValidCardDef {
        kind,
        mana_cost: *mana_cost,
        modifier_effects,
        play_effects,
        triggers,
        targets: targets.clone(),
    })
}

/// Checks every effect's own selector before the triggers any of them grants.
fn parse_trigger(
    def_id: DefId,
    trigger: &TriggerDef,
) -> Result<TriggerDef<NoTarget>, CardDefError> {
    let TriggerDef { pattern, effects } = trigger;
    let untargeted = effects
        .as_slice()
        .iter()
        .map(|effect| {
            parse_effect(
                effect,
                |&target_id| Err(CardDefError::TargetInTrigger(def_id, target_id)),
                |granted| Ok(granted.clone()),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    let effects = untargeted
        .iter()
        .map(|effect| parse_grants(def_id, effect))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(TriggerDef {
        pattern: *pattern,
        effects: EffectSequence(effects),
    })
}

fn parse_kind(def_id: DefId, kind: &CardKindDef) -> Result<CardKindDef<NoTarget>, CardDefError> {
    Ok(match kind {
        CardKindDef::Spell(spell) => CardKindDef::Spell(spell.clone()),
        CardKindDef::Minion(MinionCardDef {
            attack,
            health,
            friendly_aura_effects,
            hostile_aura_effects,
        }) => CardKindDef::Minion(MinionCardDef {
            attack: *attack,
            health: *health,
            friendly_aura_effects: parse_modifiers(def_id, friendly_aura_effects)?,
            hostile_aura_effects: parse_modifiers(def_id, hostile_aura_effects)?,
        }),
        CardKindDef::Hero(hero) => CardKindDef::Hero(hero.clone()),
    })
}

fn parse_modifiers(
    def_id: DefId,
    modifier_effects: &[ModifierEffect],
) -> Result<Vec<ModifierEffect<NoTarget>>, CardDefError> {
    modifier_effects
        .iter()
        .map(|modifier_effect| parse_modifier(def_id, modifier_effect))
        .collect()
}

fn parse_modifier(
    def_id: DefId,
    modifier_effect: &ModifierEffect,
) -> Result<ModifierEffect<NoTarget>, CardDefError> {
    Ok(match modifier_effect {
        ModifierEffect::BuffAtk { amount } => ModifierEffect::BuffAtk { amount: *amount },
        ModifierEffect::BuffHealth { amount } => ModifierEffect::BuffHealth { amount: *amount },
        ModifierEffect::ReduceManaCost { amount } => {
            ModifierEffect::ReduceManaCost { amount: *amount }
        }
        ModifierEffect::GrantTrigger(trigger) => {
            ModifierEffect::GrantTrigger(parse_trigger(def_id, trigger)?)
        }
    })
}

/// Parses the triggers the effect grants, and keeps its own selector.
fn parse_grants<T: Copy>(
    def_id: DefId,
    effect: &Effect<T, TargetId>,
) -> Result<Effect<T, NoTarget>, CardDefError> {
    parse_effect(
        effect,
        |&own| Ok(own),
        |granted| parse_modifier(def_id, granted),
    )
}

/// Rebuilds the effect with `own` applied to its own selector's target, and `grant` to the modifier
/// it grants.
fn parse_effect<T, U, G, H>(
    effect: &Effect<T, G>,
    own: impl Fn(&T) -> Result<U, CardDefError>,
    grant: impl Fn(&ModifierEffect<G>) -> Result<ModifierEffect<H>, CardDefError>,
) -> Result<Effect<U, H>, CardDefError> {
    let selector = |selector: &CharacterSelector<T>| {
        Ok(match selector {
            CharacterSelector::All(filter) => CharacterSelector::All(*filter),
            CharacterSelector::Itself => CharacterSelector::Itself,
            CharacterSelector::OwnerHero => CharacterSelector::OwnerHero,
            CharacterSelector::Random(filter) => CharacterSelector::Random(*filter),
            CharacterSelector::Targeted(target) => CharacterSelector::Targeted(own(target)?),
        })
    };
    Ok(match effect {
        Effect::Draw { selector, amount } => Effect::Draw {
            selector: *selector,
            amount: *amount,
        },
        Effect::Reveal {
            pick_id,
            selector,
            amount,
        } => Effect::Reveal {
            pick_id: *pick_id,
            selector: *selector,
            amount: *amount,
        },
        Effect::Damage {
            selector: own_selector,
            amount,
        } => Effect::Damage {
            selector: selector(own_selector)?,
            amount: *amount,
        },
        Effect::Buff {
            selector: own_selector,
            effect,
        } => Effect::Buff {
            selector: selector(own_selector)?,
            effect: grant(effect)?,
        },
        Effect::AddFriendlyAura {
            selector: own_selector,
            effect,
        } => Effect::AddFriendlyAura {
            selector: selector(own_selector)?,
            effect: grant(effect)?,
        },
        Effect::Summon { selector, def_id } => Effect::Summon {
            selector: *selector,
            def_id: *def_id,
        },
        Effect::ResolvePick {
            pick_id,
            picked_move,
            other_move,
        } => Effect::ResolvePick {
            pick_id: *pick_id,
            picked_move: *picked_move,
            other_move: *other_move,
        },
    })
}

/// Rejects a card whose effect sequences, triggers included, resolve a pick with no `Reveal`
/// before it, or reveal cards they never resolve.
fn validate_picks(defs: &[CardDef]) -> Result<(), CardDefError> {
    for def in defs {
        for effects in effect_sequences(def) {
            validate_pick_order(def.id, effects)?;
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
    use crate::cards::definition::PlayerSelector;
    use crate::trigger::{TriggerPattern, TriggerSubject};

    fn validate(defs: &[CardDef]) -> Result<(), CardDefError> {
        parse(defs).map(drop)
    }

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

    fn trigger_on<G>(pattern: TriggerPattern) -> TriggerDef<G> {
        TriggerDef {
            pattern,
            effects: EffectSequence(vec![
                Effect::Draw {
                    selector: PlayerSelector::Owner,
                    amount: 1.into(),
                },
                Effect::Damage {
                    selector: CharacterSelector::OwnerHero,
                    amount: 2.into(),
                },
            ]),
        }
    }

    #[test]
    fn a_play_effect_keeps_the_target_it_names() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, PING).push(Effect::Buff {
            selector: CharacterSelector::Targeted(TargetId(0)),
            effect: ModifierEffect::BuffHealth { amount: 3.into() },
        });

        let parsed = parse(&defs).unwrap();

        assert_eq!(
            parsed[&PING].play_effects().last(),
            Some(&Effect::Buff {
                selector: CharacterSelector::Targeted(DeclaredTarget(TargetId(0))),
                effect: ModifierEffect::BuffHealth { amount: 3.into() },
            })
        );
    }

    #[test]
    fn printed_triggers_are_kept_as_written() {
        let died = TriggerPattern::Died(TriggerSubject::Itself);
        let damaged = TriggerPattern::Damaged(TriggerSubject::Itself);
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.triggers = vec![trigger_on(died), trigger_on(damaged)];

        let parsed = parse(&defs).unwrap();

        assert_eq!(
            parsed[&PING].triggers(),
            [trigger_on(died), trigger_on(damaged)]
        );
    }

    #[test]
    fn a_trigger_granted_by_a_play_effect_keeps_its_effects_in_order() {
        let mut defs = base::definitions();
        play_effects_of(&mut defs, DRUMMER).push(Effect::Buff {
            selector: CharacterSelector::Itself,
            effect: ModifierEffect::GrantTrigger(trigger_on(TriggerPattern::Damaged(
                TriggerSubject::Itself,
            ))),
        });

        let parsed = parse(&defs).unwrap();

        assert_eq!(
            parsed[&DRUMMER].play_effects().last(),
            Some(&Effect::Buff {
                selector: CharacterSelector::Itself,
                effect: ModifierEffect::GrantTrigger(trigger_on(TriggerPattern::Damaged(
                    TriggerSubject::Itself
                ))),
            })
        );
    }

    #[test]
    fn a_trigger_granted_by_a_modifier_effect_keeps_its_effects_in_order() {
        let mut defs = base::definitions();
        let ping = defs.iter_mut().find(|def| def.id == PING).unwrap();
        ping.modifier_effects
            .push(ModifierEffect::GrantTrigger(trigger_on(
                TriggerPattern::Damaged(TriggerSubject::Itself),
            )));

        let parsed = parse(&defs).unwrap();

        assert_eq!(
            parsed[&PING].modifier_effects(),
            [ModifierEffect::GrantTrigger(trigger_on(
                TriggerPattern::Damaged(TriggerSubject::Itself)
            ))]
        );
    }

    #[test]
    fn a_trigger_granted_by_an_aura_keeps_its_effects_in_order() {
        let mut defs = base::definitions();
        replace_rally_banner_friendly_auras(
            &mut defs,
            vec![ModifierEffect::GrantTrigger(trigger_on(
                TriggerPattern::Damaged(TriggerSubject::Itself),
            ))],
        );

        let parsed = parse(&defs).unwrap();

        assert_eq!(
            parsed[&RALLY_BANNER].kind().friendly_aura_effects(),
            [ModifierEffect::GrantTrigger(trigger_on(
                TriggerPattern::Damaged(TriggerSubject::Itself)
            ))]
        );
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
