use enumset::EnumSet;

use crate::{
    cards::{
        definition::{
            CardDef, CardDefKind, CharacterSelector, DefId, Effect, EffectSequence, HeroCardDef,
            MinionCardDef, PlayerSelector, SpellCardDef, Trigger, TriggerPattern, TriggerSubject,
        },
        filter::{CardFilter, CardKindFilter, CharacterFilter, CharacterKindFilter, Side},
        modifier::ModifierEffect,
    },
    choice::{AmountBound, CharacterChoice, ChoiceId, EffectAmount},
    history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter},
    static_card_definition::*,
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
}

pub(crate) const PLACEHOLDER: DefId = DefId::new("placeholder");

macro_rules! def_ids {
    ($($name:ident = $id:literal),* $(,)?) => {
        pub mod static_card_definition {
            use crate::cards::definition::DefId;

            $(
                pub const $name: DefId = DefId::new($id);
            )*

            pub(super) const ALL_DEF_ID: &[DefId] = &[
            $(
                $name,
            )*
            ];
        }
    };
}

def_ids!(
    CAPTAIN = "base.captain.v0",
    BLAST = "base.blast.v0",
    RECRUIT = "base.recruit.v0",
    GIANT = "base.giant.v0",
    SPARK = "base.spark.v0",
    BOLT = "base.bolt.v0",
    FORAGE = "base.forage.v0",
    WILD_BOLT = "base.wild_bolt.v0",
    HERO = "base.hero.v0",
    SQUIRE = "base.squire.v0",
    BARRACKS = "base.barracks.v0",
    ZAP = "base.zap.v0",
    STRAY_SHOT = "base.stray_shot.v0",
    PING = "base.ping.v0",
    SHOVE = "base.shove.v0",
    TWIN_SHOT = "base.twin_shot.v0",
    CROSSFIRE = "base.crossfire.v0",
    MEDIC = "base.medic.v0",
    MOURNER = "base.mourner.v0",
    DRILLMASTER = "base.drillmaster.v0",
    RALLY_BANNER = "base.rally_banner.v0",
    BOMBER = "base.bomber.v0",
);

// TODO: loading cards from disk into the engine should be shells job
#[derive(Debug)]
pub struct CardDefLoader;

impl CardDefLoader {
    pub fn load_all(&self) -> Vec<CardDef> {
        vec![
            CardDef {
                id: SPARK,
                name: "Spark".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Heroes),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    amount: 1.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: BOLT,
                name: "Bolt".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Heroes),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    amount: 2.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: WILD_BOLT,
                name: "Wild Bolt".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Random(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Heroes),
                        side: EnumSet::all(),
                    }),
                    amount: 3.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: BLAST,
                name: "Blast".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 3,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::All(CharacterFilter::all()),
                    amount: 2.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: FORAGE,
                name: "Forage".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: EffectSequence(vec![Effect::Reveal {
                    selector: PlayerSelector::Owner,
                    amount: 2.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: RECRUIT,
                name: "Recruit".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(2, 2)),
                mana_cost: 2,
                ..Default::default()
            },
            CardDef {
                id: CAPTAIN,
                name: "Captain".to_string(),
                kind: CardDefKind::Minion(MinionCardDef {
                    health: 1,
                    attack: 1,
                    friendly_aura_effects: vec![
                        ModifierEffect::BuffAtk { amount: 1.into() },
                        ModifierEffect::BuffHealth { amount: 1.into() },
                    ],
                    hostile_aura_effects: Default::default(),
                }),
                mana_cost: 3,
                ..Default::default()
            },
            CardDef {
                id: GIANT,
                name: "Giant".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(5, 5)),
                mana_cost: 8,
                modifier_effects: vec![ModifierEffect::ReduceManaCost {
                    amount: EffectAmount::History(HistoryQuery {
                        kind: HistoryQueryKind::SpellsPlayed,
                        scope: PlayerFilter::Owner,
                        turn: TurnFilter::All,
                    }),
                }],
                ..Default::default()
            },
            CardDef {
                id: HERO,
                name: "Hero".to_string(),
                kind: CardDefKind::Hero(HeroCardDef { health: 10 }),
                ..Default::default()
            },
            CardDef {
                id: SQUIRE,
                name: "Squire".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 1,
                ..Default::default()
            },
            CardDef {
                id: BARRACKS,
                name: "Barracks".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: EffectSequence(vec![Effect::Summon {
                    selector: PlayerSelector::Owner,
                    def_id: SQUIRE,
                }]),
                ..Default::default()
            },
            CardDef {
                id: ZAP,
                name: "Zap".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: EffectSequence(vec![
                    Effect::Damage {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Heroes),
                            side: EnumSet::only(Side::Enemy),
                        }),
                        amount: 1.into(),
                    },
                    Effect::Draw {
                        selector: PlayerSelector::Owner,
                        amount: 1.into(),
                    },
                ]),
                ..Default::default()
            },
            CardDef {
                id: STRAY_SHOT,
                name: "Stray Shot".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Random(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    amount: 2.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: PING,
                name: "Ping".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Chosen(ChoiceId(0)),
                    amount: 2.into(),
                }]),
                choices: vec![CharacterChoice {
                    id: ChoiceId(0),
                    filter: CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::all(),
                    },
                    count: 1.into(),
                    bound: AmountBound::Exactly,
                    different: true,
                }],
                ..Default::default()
            },
            CardDef {
                id: SHOVE,
                name: "Shove".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: EffectSequence(vec![
                    Effect::Damage {
                        selector: CharacterSelector::Chosen(ChoiceId(0)),
                        amount: 1.into(),
                    },
                    Effect::Damage {
                        selector: CharacterSelector::Chosen(ChoiceId(1)),
                        amount: 1.into(),
                    },
                ]),
                choices: vec![
                    CharacterChoice {
                        id: ChoiceId(0),
                        filter: CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                    CharacterChoice {
                        id: ChoiceId(1),
                        filter: CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::only(Side::Friendly),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                ],
                ..Default::default()
            },
            CardDef {
                id: TWIN_SHOT,
                name: "Twin Shot".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Chosen(ChoiceId(0)),
                    amount: 1.into(),
                }]),
                choices: vec![CharacterChoice {
                    id: ChoiceId(0),
                    filter: CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::all(),
                    },
                    count: 2.into(),
                    bound: AmountBound::Exactly,
                    different: true,
                }],
                ..Default::default()
            },
            CardDef {
                id: CROSSFIRE,
                name: "Crossfire".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: EffectSequence(vec![
                    Effect::Damage {
                        selector: CharacterSelector::Chosen(ChoiceId(0)),
                        amount: 1.into(),
                    },
                    Effect::Damage {
                        selector: CharacterSelector::Chosen(ChoiceId(1)),
                        amount: 1.into(),
                    },
                ]),
                choices: vec![
                    CharacterChoice {
                        id: ChoiceId(0),
                        filter: CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                    CharacterChoice {
                        id: ChoiceId(1),
                        filter: CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Heroes),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                ],
                ..Default::default()
            },
            CardDef {
                id: MEDIC,
                name: "Medic".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 2,
                triggers: vec![Trigger {
                    pattern: TriggerPattern::Died(TriggerSubject::Itself),
                    effects: EffectSequence(vec![Effect::Buff {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::only(Side::Friendly),
                        }),
                        effect: ModifierEffect::BuffHealth { amount: 2.into() },
                    }]),
                }],
                ..Default::default()
            },
            CardDef {
                id: MOURNER,
                name: "Mourner".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(1, 3)),
                mana_cost: 3,
                triggers: vec![Trigger {
                    pattern: TriggerPattern::Died(TriggerSubject::Character(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::only(Side::Friendly),
                    })),
                    effects: EffectSequence(vec![Effect::Draw {
                        selector: PlayerSelector::Owner,
                        amount: 1.into(),
                    }]),
                }],
                ..Default::default()
            },
            CardDef {
                id: DRILLMASTER,
                name: "Drillmaster".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(2, 3)),
                mana_cost: 3,
                triggers: vec![Trigger {
                    pattern: TriggerPattern::Played(CardFilter {
                        kind: EnumSet::only(CardKindFilter::Spells),
                        side: EnumSet::only(Side::Friendly),
                    }),
                    effects: EffectSequence(vec![Effect::Summon {
                        selector: PlayerSelector::Owner,
                        def_id: SQUIRE,
                    }]),
                }],
                ..Default::default()
            },
            CardDef {
                id: RALLY_BANNER,
                name: "Rally Banner".to_string(),
                kind: CardDefKind::Minion(MinionCardDef {
                    attack: 0,
                    health: 5,
                    friendly_aura_effects: vec![ModifierEffect::GrantTrigger(Trigger {
                        pattern: TriggerPattern::Damaged(TriggerSubject::Itself),
                        effects: EffectSequence(vec![Effect::Draw {
                            selector: PlayerSelector::Owner,
                            amount: 1.into(),
                        }]),
                    })],
                    hostile_aura_effects: Default::default(),
                }),
                mana_cost: 3,
                ..Default::default()
            },
            CardDef {
                id: BOMBER,
                name: "Bomber".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 2,
                triggers: vec![Trigger {
                    pattern: TriggerPattern::Died(TriggerSubject::Itself),
                    effects: EffectSequence(vec![Effect::Damage {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::all(),
                        }),
                        amount: 1.into(),
                    }]),
                }],
                ..Default::default()
            },
        ]
    }

    /// Tests ensure statically defined cards exist
    pub fn load_and_validate(&self) -> Result<Vec<CardDef>, CardDefError> {
        let defs = self.load_all();
        validate(&defs)?;
        Ok(defs)
    }
}

fn validate(defs: &[CardDef]) -> Result<(), CardDefError> {
    validate_placeholder(defs)?;
    validate_duplicates(defs)?;
    validate_not_found(defs)?;
    validate_choice_duplicates(defs)?;
    validate_choice_not_found(defs)?;
    validate_choice_on_trigger(defs)?;
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
        Effect::Draw { .. } | Effect::Reveal { .. } | Effect::Summon { .. } => return None,
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
                match modifier_effect {
                    ModifierEffect::GrantTrigger(trigger) => {
                        if let Some(choice_id) = choice_on_trigger(trigger) {
                            return Err(CardDefError::ChoiceInTrigger(def.id, choice_id));
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

fn choice_on_trigger(trigger: &Trigger) -> Option<ChoiceId> {
    trigger.effects.as_slice().iter().find_map(chosen_id)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    fn chosen_damage_trigger() -> Trigger {
        Trigger {
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
}
