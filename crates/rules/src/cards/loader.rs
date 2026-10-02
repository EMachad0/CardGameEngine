use crate::cards::definition::{
    CardDef, CardDefKind, DefId, Effect, EffectSequence, MonsterCardDef, MonsterTargeteer,
    PlayerTargeteer, SpellCardDef,
};

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
#[error("Card definition not found {0:?}")]
pub struct CardDefNotFound(DefId);

pub const BLAST: DefId = DefId::new("blast");
pub const CAPTAIN: DefId = DefId::new("captain");
pub const GIANT: DefId = DefId::new("giant");
pub const RECRUIT: DefId = DefId::new("recruit");
pub const SPARK: DefId = DefId::new("spark");
pub const BOLT: DefId = DefId::new("bolt");
pub const FORAGE: DefId = DefId::new("forage");
pub const WILD_BOLT: DefId = DefId::new("wild_bolt");

// TODO: loading cards from disk into the engine should be shells job
#[derive(Debug)]
pub struct CardDefLoader;

impl CardDefLoader {
    pub fn load_all(&self) -> Vec<CardDef> {
        vec![
            CardDef {
                id: SPARK,
                name: "Spark".to_string(),
                kind: CardDefKind::Spell(SpellCardDef {}),
                mana_cost: 1,
                on_play_effect: EffectSequence(vec![Effect::DamagePlayer {
                    targeteer: PlayerTargeteer::NextPlayer,
                    damage: 1,
                }]),
            },
            CardDef {
                id: BOLT,
                name: "Bolt".to_string(),
                kind: CardDefKind::Spell(SpellCardDef {}),
                mana_cost: 2,
                on_play_effect: EffectSequence(vec![Effect::DamagePlayer {
                    targeteer: PlayerTargeteer::NextPlayer,
                    damage: 2,
                }]),
            },
            CardDef {
                id: WILD_BOLT,
                name: "Wild Bolt".to_string(),
                kind: CardDefKind::Spell(SpellCardDef {}),
                mana_cost: 1,
                on_play_effect: EffectSequence(vec![Effect::DamagePlayer {
                    targeteer: PlayerTargeteer::RandomPlayer,
                    damage: 3,
                }]),
            },
            CardDef {
                id: BLAST,
                name: "Blast".to_string(),
                kind: CardDefKind::Spell(SpellCardDef {}),
                mana_cost: 3,
                on_play_effect: EffectSequence(vec![
                    Effect::DamageMonster {
                        targeteer: MonsterTargeteer::All,
                        damage: 2,
                    },
                    Effect::DamagePlayer {
                        targeteer: PlayerTargeteer::All,
                        damage: 2,
                    },
                ]),
            },
            CardDef {
                id: FORAGE,
                name: "Forage".to_string(),
                kind: CardDefKind::Spell(SpellCardDef {}),
                mana_cost: 1,
                on_play_effect: EffectSequence(vec![Effect::RevealToPicker {
                    targeteer: PlayerTargeteer::Caster,
                    count: 2,
                }]),
            },
            CardDef {
                id: RECRUIT,
                name: "Recruit".to_string(),
                kind: CardDefKind::Monster(MonsterCardDef {
                    health: 1,
                    attack: 1,
                }),
                mana_cost: 2,
                on_play_effect: EffectSequence::default(),
            },
            CardDef {
                id: CAPTAIN,
                name: "Captain".to_string(),
                kind: CardDefKind::Monster(MonsterCardDef {
                    health: 3,
                    attack: 3,
                }),
                mana_cost: 3,
                on_play_effect: EffectSequence::default(),
            },
            CardDef {
                id: GIANT,
                name: "Giant".to_string(),
                kind: CardDefKind::Monster(MonsterCardDef {
                    health: 3,
                    attack: 3,
                }),
                mana_cost: 3,
                on_play_effect: EffectSequence::default(),
            },
        ]
    }

    pub fn load_and_validate(&self) -> Result<Vec<CardDef>, CardDefNotFound> {
        let defs = self.load_all();

        // TODO: strengthen this valition
        for def_id in [FORAGE, BOLT, WILD_BOLT] {
            if !defs.iter().any(|def| def.id == def_id) {
                return Err(CardDefNotFound(def_id));
            }
        }
        Ok(defs)
    }
}
