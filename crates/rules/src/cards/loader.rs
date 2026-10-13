use enumset::EnumSet;

use crate::cards::{
    amount::{EffectAmount, HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter},
    choice::{AmountBound, CharacterChoice, ChoiceId, PickId},
    def_ids::static_card_definition::*,
    definition::{CardDef, CardDefKind, HeroCardDef, MinionCardDef, SpellCardDef},
    effect::{Effect, EffectSequence, ZoneMove},
    filter::{CardFilter, CardKindFilter, CharacterFilter, CharacterKindFilter, Side},
    modifier::ModifierEffect,
    selector::{CharacterSelector, PlayerSelector},
    trigger::{TriggerDef, TriggerPattern, TriggerSubject},
    validate::{CardDefError, validate},
};

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
                play_effects: EffectSequence(vec![
                    Effect::Reveal {
                        pick_id: PickId(0),
                        selector: PlayerSelector::Owner,
                        amount: 2.into(),
                    },
                    Effect::ResolvePick {
                        pick_id: PickId(0),
                        picked_move: ZoneMove::AddToHand,
                        other_move: ZoneMove::Bury,
                    },
                ]),
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
                triggers: vec![TriggerDef {
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
                triggers: vec![TriggerDef {
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
                triggers: vec![TriggerDef {
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
                    friendly_aura_effects: vec![ModifierEffect::GrantTrigger(TriggerDef {
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
                triggers: vec![TriggerDef {
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
            CardDef {
                id: GRAVEDIGGER,
                name: "Gravedigger".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(2, 2)),
                mana_cost: 2,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Character(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::all(),
                    })),
                    effects: EffectSequence(vec![Effect::Damage {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Heroes),
                            side: EnumSet::only(Side::Enemy),
                        }),
                        amount: 1.into(),
                    }]),
                }],
                ..Default::default()
            },
            CardDef {
                id: SWEEP,
                name: "Sweep".to_string(),
                kind: CardDefKind::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: EffectSequence(vec![
                    Effect::Damage {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKindFilter::Minions),
                            side: EnumSet::all(),
                        }),
                        amount: 1.into(),
                    },
                    Effect::Reveal {
                        pick_id: PickId(0),
                        selector: PlayerSelector::Owner,
                        amount: 2.into(),
                    },
                    Effect::ResolvePick {
                        pick_id: PickId(0),
                        picked_move: ZoneMove::AddToHand,
                        other_move: ZoneMove::Bury,
                    },
                    Effect::Draw {
                        selector: PlayerSelector::Owner,
                        amount: 1.into(),
                    },
                ]),
                ..Default::default()
            },
            CardDef {
                id: HOARDER,
                name: "Hoarder".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 1,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Itself),
                    effects: EffectSequence(vec![
                        Effect::Reveal {
                            pick_id: PickId(0),
                            selector: PlayerSelector::Owner,
                            amount: 2.into(),
                        },
                        Effect::ResolvePick {
                            pick_id: PickId(0),
                            picked_move: ZoneMove::AddToHand,
                            other_move: ZoneMove::Bury,
                        },
                    ]),
                }],
                ..Default::default()
            },
            CardDef {
                id: DRUMMER,
                name: "Drummer".to_string(),
                kind: CardDefKind::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 2,
                play_effects: EffectSequence(vec![Effect::Buff {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKindFilter::Minions),
                        side: EnumSet::only(Side::Friendly),
                    }),
                    effect: ModifierEffect::BuffHealth { amount: 1.into() },
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
