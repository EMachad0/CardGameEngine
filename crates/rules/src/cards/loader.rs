use enumset::EnumSet;

use crate::{
    cards::{
        definition::{
            CardDef, CardDefKind, CharacterKindFilter, CharacterSelector, CharacterSelectorFilter,
            CharacterSideFilter, DefId, Effect, EffectSequence, HeroCardDef, MinionCardDef,
            PlayerSelector, SpellCardDef,
        },
        modifier::{EffectAmount, ModifierEffect},
    },
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
}
