use enumset::EnumSet;

use crate::{
    cards::{
        definition::{
            CardDef, CardDefKind, CharacterKindFilter, CharacterSelector, CharacterSelectorFilter,
            CharacterSideFilter, DefId, Effect, EffectSequence, HeroCardDef, MinionCardDef,
            PlayerSelector, SpellCardDef,
        },
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::All(CharacterSelectorFilter {
                        kind: EnumSet::only(CharacterKindFilter::Heroes),
                        side: EnumSet::only(CharacterSideFilter::Enemy),
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::All(CharacterSelectorFilter {
                        kind: EnumSet::only(CharacterKindFilter::Heroes),
                        side: EnumSet::only(CharacterSideFilter::Enemy),
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Random(CharacterSelectorFilter {
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::All(CharacterSelectorFilter::all()),
                    amount: 2.into(),
                }]),
                ..Default::default()
            },
            CardDef {
                id: FORAGE,
                name: "Forage".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 1,
                on_play_effect: EffectSequence(vec![Effect::Reveal {
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
                on_play_effect: EffectSequence(vec![Effect::Summon {
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
                on_play_effect: EffectSequence(vec![
                    Effect::Damage {
                        selector: CharacterSelector::All(CharacterSelectorFilter {
                            kind: EnumSet::only(CharacterKindFilter::Heroes),
                            side: EnumSet::only(CharacterSideFilter::Enemy),
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Random(CharacterSelectorFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::only(CharacterSideFilter::Enemy),
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Chosen(ChoiceId(0)),
                    amount: 2.into(),
                }]),
                choices: vec![CharacterChoice {
                    id: ChoiceId(0),
                    filter: CharacterSelectorFilter {
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
                on_play_effect: EffectSequence(vec![
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
                        filter: CharacterSelectorFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                    CharacterChoice {
                        id: ChoiceId(1),
                        filter: CharacterSelectorFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::only(CharacterSideFilter::Friendly),
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
                on_play_effect: EffectSequence(vec![Effect::Damage {
                    selector: CharacterSelector::Chosen(ChoiceId(0)),
                    amount: 1.into(),
                }]),
                choices: vec![CharacterChoice {
                    id: ChoiceId(0),
                    filter: CharacterSelectorFilter {
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
                on_play_effect: EffectSequence(vec![
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
                        filter: CharacterSelectorFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    },
                    CharacterChoice {
                        id: ChoiceId(1),
                        filter: CharacterSelectorFilter {
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
        let effects = [
            &def.on_play_effect,
            &def.on_board_enter,
            &def.on_board_leave,
            &def.on_death,
        ]
        .into_iter()
        .flat_map(EffectSequence::as_slice);
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
        Effect::Damage { selector, .. } | Effect::AddFriendlyAura { selector, .. } => selector,
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

    #[test]
    fn an_on_death_effect_naming_an_undeclared_choice_is_not_found() {
        let mut defs = CardDefLoader.load_all();
        let recruit = defs.iter_mut().find(|def| def.id == RECRUIT).unwrap();
        recruit.on_death = EffectSequence(vec![Effect::Damage {
            selector: CharacterSelector::Chosen(ChoiceId(0)),
            amount: 1.into(),
        }]);

        assert_eq!(
            validate(&defs),
            Err(CardDefError::ChoiceNotFound(RECRUIT, ChoiceId(0)))
        );
    }
}
