//! Written card data into its parsed form, each code that names a card replaced by its `DefId`.

use std::collections::BTreeMap;

use crate::cards::binder::def_code::DefCode;
use crate::cards::binder::def_id::DefId;
use crate::cards::binder::store::BinderError;
use crate::cards::definition::{
    CardDef, CardKind, CardKindDef, CharacterSelector, MinionCardDef, ParsedCardDef,
    ParsedCardKindDef, ParsedCharacterSelector, ParsedMinionCardDef, ParsedTriggerDef, TriggerDef,
};
use crate::cards::modifier::{ModifierEffect, ParsedModifierEffect};
use crate::effect::{
    CharacterVerb, Effect, ParsedCharacterVerb, ParsedEffect, ParsedPlayerVerb, PlayerVerb,
};
use crate::target::TargetForm;

/// What the parse needs to know of the card a code names.
pub(super) struct Named {
    pub(super) def_id: DefId,
    pub(super) kind: CardKind,
}

type Codes = BTreeMap<DefCode, Named>;

pub(super) fn kind(kind: &CardKindDef) -> CardKind {
    match kind {
        CardKindDef::Spell(_) => CardKind::Spell,
        CardKindDef::Minion(_) => CardKind::Minion,
        CardKindDef::Hero(_) => CardKind::Hero,
    }
}

pub(super) fn card(def: CardDef, codes: &Codes) -> Result<ParsedCardDef, BinderError> {
    let CardDef {
        name,
        kind,
        mana_cost,
        modifier_effects,
        play_effects,
        triggers,
    } = def;
    Ok(ParsedCardDef {
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

fn kind_def(kind: CardKindDef, codes: &Codes) -> Result<ParsedCardKindDef, BinderError> {
    Ok(match kind {
        CardKindDef::Spell(spell) => ParsedCardKindDef::Spell(spell),
        CardKindDef::Minion(minion) => ParsedCardKindDef::Minion(minion_card(minion, codes)?),
        CardKindDef::Hero(hero) => ParsedCardKindDef::Hero(hero),
    })
}

fn minion_card(minion: MinionCardDef, codes: &Codes) -> Result<ParsedMinionCardDef, BinderError> {
    let MinionCardDef {
        attack,
        health,
        friendly_aura_effects,
        hostile_aura_effects,
    } = minion;
    Ok(ParsedMinionCardDef {
        attack,
        health,
        friendly_aura_effects: each(friendly_aura_effects, codes, modifier_effect)?,
        hostile_aura_effects: each(hostile_aura_effects, codes, modifier_effect)?,
    })
}

fn trigger(trigger: TriggerDef, codes: &Codes) -> Result<ParsedTriggerDef, BinderError> {
    let TriggerDef { pattern, effects } = trigger;
    Ok(ParsedTriggerDef {
        pattern,
        effects: each(effects, codes, effect)?,
    })
}

fn modifier_effect(
    modifier_effect: ModifierEffect,
    codes: &Codes,
) -> Result<ParsedModifierEffect, BinderError> {
    Ok(match modifier_effect {
        ModifierEffect::BuffAtk { amount } => ParsedModifierEffect::BuffAtk { amount },
        ModifierEffect::BuffHealth { amount } => ParsedModifierEffect::BuffHealth { amount },
        ModifierEffect::ReduceManaCost { amount } => {
            ParsedModifierEffect::ReduceManaCost { amount }
        }
        ModifierEffect::GrantTrigger(granted) => {
            ParsedModifierEffect::GrantTrigger(trigger(granted, codes)?)
        }
    })
}

fn effect<S: TargetForm>(effect: Effect<S>, codes: &Codes) -> Result<ParsedEffect<S>, BinderError> {
    Ok(match effect {
        Effect::Players { selector, verbs } => ParsedEffect::Players {
            selector,
            verbs: each(verbs, codes, player_verb)?,
        },
        Effect::Characters { selector, verbs } => ParsedEffect::Characters {
            selector: character_selector(selector, codes)?,
            verbs: each(verbs, codes, character_verb)?,
        },
        Effect::Cards { selector, verbs } => ParsedEffect::Cards { selector, verbs },
    })
}

fn character_selector<S: TargetForm>(
    selector: CharacterSelector<S>,
    codes: &Codes,
) -> Result<ParsedCharacterSelector<S>, BinderError> {
    Ok(match selector {
        CharacterSelector::All(filter) => ParsedCharacterSelector::All(filter),
        CharacterSelector::Itself => ParsedCharacterSelector::Itself,
        CharacterSelector::OwnerHero => ParsedCharacterSelector::OwnerHero,
        CharacterSelector::Random(filter) => ParsedCharacterSelector::Random(filter),
        CharacterSelector::Targeted(target) => ParsedCharacterSelector::Targeted(target),
        CharacterSelector::Picked { choice, others } => ParsedCharacterSelector::Picked {
            choice,
            others: each(others, codes, character_verb)?,
        },
    })
}

fn player_verb(verb: PlayerVerb, codes: &Codes) -> Result<ParsedPlayerVerb, BinderError> {
    Ok(match verb {
        PlayerVerb::Draw(amount) => ParsedPlayerVerb::Draw(amount),
        PlayerVerb::Summon(code) => ParsedPlayerVerb::Summon(minion(code, codes)?),
    })
}

fn character_verb(verb: CharacterVerb, codes: &Codes) -> Result<ParsedCharacterVerb, BinderError> {
    Ok(match verb {
        CharacterVerb::Damage(amount) => ParsedCharacterVerb::Damage(amount),
        CharacterVerb::Modify(modifier) => {
            ParsedCharacterVerb::Modify(modifier_effect(modifier, codes)?)
        }
        CharacterVerb::AddFriendlyAura(modifier) => {
            ParsedCharacterVerb::AddFriendlyAura(modifier_effect(modifier, codes)?)
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
