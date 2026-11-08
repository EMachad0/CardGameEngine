//! Written card data into its parsed form, each code that names a card replaced by its `DefId`.

use std::collections::BTreeMap;

use crate::cards::binder::def_code::DefCode;
use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::BinderError;
use crate::cards::kind::CardKind;
use crate::cards::modifier::ModifierEffect;
use crate::cards::parsed::{CardDef, CardKindDef, CharacterSelector, MinionCardDef, TriggerDef};
use crate::cards::written;
use crate::effect::{CharacterVerb, Effect, PlayerVerb};
use crate::target::TargetForm;

/// What the parse needs to know of the card a code names.
pub(super) struct Named {
    pub(super) def_id: DefId,
    pub(super) kind: CardKind,
}

type Codes = BTreeMap<DefCode, Named>;

pub(super) fn kind(kind: &written::CardKindDef) -> CardKind {
    match kind {
        written::CardKindDef::Spell(_) => CardKind::Spell,
        written::CardKindDef::Minion(_) => CardKind::Minion,
        written::CardKindDef::Hero(_) => CardKind::Hero,
    }
}

pub(super) fn card(def: written::CardDef, codes: &Codes) -> Result<CardDef, BinderError> {
    let written::CardDef {
        name,
        kind,
        mana_cost,
        modifier_effects,
        play_effects,
        triggers,
    } = def;
    Ok(CardDef {
        name,
        kind: kind_def(kind, codes)?,
        mana_cost,
        modifier_effects: each(modifier_effects, codes, modifier_effect)?,
        play_effects: each(play_effects, codes, effect)?,
        triggers: each(triggers, codes, trigger)?,
    })
}

fn each<W, P>(
    written: Vec<W>,
    codes: &Codes,
    parse: impl Fn(W, &Codes) -> Result<P, BinderError>,
) -> Result<Vec<P>, BinderError> {
    written.into_iter().map(|w| parse(w, codes)).collect()
}

fn kind_def(kind: written::CardKindDef, codes: &Codes) -> Result<CardKindDef, BinderError> {
    Ok(match kind {
        written::CardKindDef::Spell(spell) => CardKindDef::Spell(spell),
        written::CardKindDef::Minion(minion) => CardKindDef::Minion(minion_card(minion, codes)?),
        written::CardKindDef::Hero(hero) => CardKindDef::Hero(hero),
    })
}

fn minion_card(
    minion: written::MinionCardDef,
    codes: &Codes,
) -> Result<MinionCardDef, BinderError> {
    let written::MinionCardDef {
        attack,
        health,
        friendly_aura_effects,
        hostile_aura_effects,
    } = minion;
    Ok(MinionCardDef {
        attack,
        health,
        friendly_aura_effects: each(friendly_aura_effects, codes, modifier_effect)?,
        hostile_aura_effects: each(hostile_aura_effects, codes, modifier_effect)?,
    })
}

fn trigger(trigger: written::TriggerDef, codes: &Codes) -> Result<TriggerDef, BinderError> {
    let written::TriggerDef { pattern, effects } = trigger;
    Ok(TriggerDef {
        pattern,
        effects: each(effects, codes, effect)?,
    })
}

fn modifier_effect(
    modifier_effect: written::ModifierEffect,
    codes: &Codes,
) -> Result<ModifierEffect, BinderError> {
    Ok(match modifier_effect {
        written::ModifierEffect::BuffAtk { amount } => ModifierEffect::BuffAtk { amount },
        written::ModifierEffect::BuffHealth { amount } => ModifierEffect::BuffHealth { amount },
        written::ModifierEffect::ReduceManaCost { amount } => {
            ModifierEffect::ReduceManaCost { amount }
        }
        written::ModifierEffect::GrantTrigger(granted) => {
            ModifierEffect::GrantTrigger(trigger(granted, codes)?)
        }
    })
}

fn effect<S: TargetForm>(
    effect: written::Effect<S>,
    codes: &Codes,
) -> Result<Effect<S>, BinderError> {
    Ok(match effect {
        written::Effect::Players { selector, verbs } => Effect::Players {
            selector,
            verbs: each(verbs, codes, player_verb)?,
        },
        written::Effect::Characters { selector, verbs } => Effect::Characters {
            selector: character_selector(selector, codes)?,
            verbs: each(verbs, codes, character_verb)?,
        },
        written::Effect::Cards { selector, verbs } => Effect::Cards { selector, verbs },
    })
}

fn character_selector<S: TargetForm>(
    selector: written::CharacterSelector<S>,
    codes: &Codes,
) -> Result<CharacterSelector<S>, BinderError> {
    Ok(match selector {
        written::CharacterSelector::All(filter) => CharacterSelector::All(filter),
        written::CharacterSelector::Itself => CharacterSelector::Itself,
        written::CharacterSelector::OwnerHero => CharacterSelector::OwnerHero,
        written::CharacterSelector::Random(filter) => CharacterSelector::Random(filter),
        written::CharacterSelector::Targeted(target) => CharacterSelector::Targeted(target),
        written::CharacterSelector::Picked { choice, others } => CharacterSelector::Picked {
            choice,
            others: each(others, codes, character_verb)?,
        },
    })
}

fn player_verb(verb: written::PlayerVerb, codes: &Codes) -> Result<PlayerVerb, BinderError> {
    Ok(match verb {
        written::PlayerVerb::Draw(amount) => PlayerVerb::Draw(amount),
        written::PlayerVerb::Summon(code) => PlayerVerb::Summon(minion(code, codes)?),
    })
}

fn character_verb(
    verb: written::CharacterVerb,
    codes: &Codes,
) -> Result<CharacterVerb, BinderError> {
    Ok(match verb {
        written::CharacterVerb::Damage(amount) => CharacterVerb::Damage(amount),
        written::CharacterVerb::Modify(modifier) => {
            CharacterVerb::Modify(modifier_effect(modifier, codes)?)
        }
        written::CharacterVerb::AddFriendlyAura(modifier) => {
            CharacterVerb::AddFriendlyAura(modifier_effect(modifier, codes)?)
        }
    })
}

fn minion(code: DefCode, codes: &Codes) -> Result<DefId, BinderError> {
    let named = codes.get(&code).ok_or(BinderError::NotFound(code))?;
    match named.kind {
        CardKind::Minion => Ok(named.def_id),
        CardKind::Spell | CardKind::Hero => Err(BinderError::NotAMinion(code)),
    }
}
