use enumset::EnumSet;

use crate::cards::binder::def_id::DefId;
use crate::cards::binder::sets::base::static_card_definition::*;
use crate::cards::definition::{
    CardDef, CardKind, CardKindDef, CharacterKind, CharacterSelector, HeroCardDef, MinionCardDef,
    PlayerSelector, SpellCardDef, TriggerDef,
};
use crate::cards::filter::{CardFilter, CharacterFilter, Side};
use crate::cards::modifier::ModifierEffect;
use crate::choice::{AmountBound, Choice};
use crate::effect::{CharacterVerb, Effect, EffectAmount, PickId, PlayerVerb, ZoneMove};
use crate::history::{HistoryQuery, HistoryQueryKind, PlayerFilter, TurnFilter};
use crate::trigger::{TriggerPattern, TriggerSubject};

macro_rules! def_ids {
    ($($name:ident = $id:literal),* $(,)?) => {
        pub mod static_card_definition {
            use crate::cards::binder::def_id::DefId;

            $(
                pub const $name: DefId = DefId::new($id);
            )*

            pub(in crate::cards::binder) const ALL_DEF_ID: &[DefId] = &[
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
    GRAVEDIGGER = "base.gravedigger.v0",
    SWEEP = "base.sweep.v0",
    HOARDER = "base.hoarder.v0",
    DRUMMER = "base.drummer.v0",
    FRENZY = "base.frenzy.v0",
    GOAD = "base.goad.v0",
    AFTERSHOCK = "base.aftershock.v0",
    DUD = "base.dud.v0",
);

pub(in crate::cards::binder) fn definitions() -> Vec<(DefId, CardDef)> {
    vec![
        (
            SPARK,
            CardDef {
                name: "Spark".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Hero),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    verbs: vec![CharacterVerb::Damage(1.into())],
                }],
                ..Default::default()
            },
        ),
        (
            BOLT,
            CardDef {
                name: "Bolt".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Hero),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    verbs: vec![CharacterVerb::Damage(2.into())],
                }],
                ..Default::default()
            },
        ),
        (
            WILD_BOLT,
            CardDef {
                name: "Wild Bolt".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Random(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Hero),
                        side: EnumSet::all(),
                    }),
                    verbs: vec![CharacterVerb::Damage(3.into())],
                }],
                ..Default::default()
            },
        ),
        (
            BLAST,
            CardDef {
                name: "Blast".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 3,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::All(CharacterFilter::all()),
                    verbs: vec![CharacterVerb::Damage(2.into())],
                }],
                ..Default::default()
            },
        ),
        (
            FORAGE,
            CardDef {
                name: "Forage".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![
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
                ],
                ..Default::default()
            },
        ),
        (
            RECRUIT,
            CardDef {
                name: "Recruit".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(2, 2)),
                mana_cost: 2,
                ..Default::default()
            },
        ),
        (
            CAPTAIN,
            CardDef {
                name: "Captain".to_string(),
                kind: CardKindDef::Minion(MinionCardDef {
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
        ),
        (
            GIANT,
            CardDef {
                name: "Giant".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(5, 5)),
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
        ),
        (
            HERO,
            CardDef {
                name: "Hero".to_string(),
                kind: CardKindDef::Hero(HeroCardDef { health: 10 }),
                ..Default::default()
            },
        ),
        (
            SQUIRE,
            CardDef {
                name: "Squire".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 1,
                ..Default::default()
            },
        ),
        (
            BARRACKS,
            CardDef {
                name: "Barracks".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![Effect::Players {
                    selector: PlayerSelector::Owner,
                    verbs: vec![PlayerVerb::Summon(SQUIRE)],
                }],
                ..Default::default()
            },
        ),
        (
            ZAP,
            CardDef {
                name: "Zap".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![
                    Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Hero),
                            side: EnumSet::only(Side::Enemy),
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    },
                    Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Draw(1.into())],
                    },
                ],
                ..Default::default()
            },
        ),
        (
            STRAY_SHOT,
            CardDef {
                name: "Stray Shot".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Random(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Minion),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    verbs: vec![CharacterVerb::Damage(2.into())],
                }],
                ..Default::default()
            },
        ),
        (
            PING,
            CardDef {
                name: "Ping".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Targeted(Choice {
                        options: CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    }),
                    verbs: vec![CharacterVerb::Damage(2.into())],
                }],
                ..Default::default()
            },
        ),
        (
            SHOVE,
            CardDef {
                name: "Shove".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![
                    Effect::Characters {
                        selector: CharacterSelector::Targeted(Choice {
                            options: CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Minion),
                                side: EnumSet::all(),
                            },
                            count: 1.into(),
                            bound: AmountBound::Exactly,
                            different: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    },
                    Effect::Characters {
                        selector: CharacterSelector::Targeted(Choice {
                            options: CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Minion),
                                side: EnumSet::only(Side::Friendly),
                            },
                            count: 1.into(),
                            bound: AmountBound::Exactly,
                            different: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    },
                ],
                ..Default::default()
            },
        ),
        (
            TWIN_SHOT,
            CardDef {
                name: "Twin Shot".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Targeted(Choice {
                        options: CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                        },
                        count: 2.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    }),
                    verbs: vec![CharacterVerb::Damage(1.into())],
                }],
                ..Default::default()
            },
        ),
        (
            CROSSFIRE,
            CardDef {
                name: "Crossfire".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![
                    Effect::Characters {
                        selector: CharacterSelector::Targeted(Choice {
                            options: CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Minion),
                                side: EnumSet::all(),
                            },
                            count: 1.into(),
                            bound: AmountBound::Exactly,
                            different: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    },
                    Effect::Characters {
                        selector: CharacterSelector::Targeted(Choice {
                            options: CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Hero),
                                side: EnumSet::all(),
                            },
                            count: 1.into(),
                            bound: AmountBound::Exactly,
                            different: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    },
                ],
                ..Default::default()
            },
        ),
        (
            MEDIC,
            CardDef {
                name: "Medic".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 2,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Itself),
                    effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::only(Side::Friendly),
                        }),
                        verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffHealth {
                            amount: 2.into(),
                        })],
                    }],
                }],
                ..Default::default()
            },
        ),
        (
            MOURNER,
            CardDef {
                name: "Mourner".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(1, 3)),
                mana_cost: 3,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Character(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Minion),
                        side: EnumSet::only(Side::Friendly),
                    })),
                    effects: vec![Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Draw(1.into())],
                    }],
                }],
                ..Default::default()
            },
        ),
        (
            DRILLMASTER,
            CardDef {
                name: "Drillmaster".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(2, 3)),
                mana_cost: 3,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Played(CardFilter {
                        kind: EnumSet::only(CardKind::Spell),
                        side: EnumSet::only(Side::Friendly),
                    }),
                    effects: vec![Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Summon(SQUIRE)],
                    }],
                }],
                ..Default::default()
            },
        ),
        (
            RALLY_BANNER,
            CardDef {
                name: "Rally Banner".to_string(),
                kind: CardKindDef::Minion(MinionCardDef {
                    attack: 0,
                    health: 5,
                    friendly_aura_effects: vec![ModifierEffect::GrantTrigger(TriggerDef {
                        pattern: TriggerPattern::Damaged(TriggerSubject::Itself),
                        effects: vec![Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(1.into())],
                        }],
                    })],
                    hostile_aura_effects: Default::default(),
                }),
                mana_cost: 3,
                ..Default::default()
            },
        ),
        (
            BOMBER,
            CardDef {
                name: "Bomber".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 2,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Itself),
                    effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    }],
                }],
                ..Default::default()
            },
        ),
        (
            GRAVEDIGGER,
            CardDef {
                name: "Gravedigger".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(2, 2)),
                mana_cost: 2,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Character(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Minion),
                        side: EnumSet::all(),
                    })),
                    effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Hero),
                            side: EnumSet::only(Side::Enemy),
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    }],
                }],
                ..Default::default()
            },
        ),
        (
            SWEEP,
            CardDef {
                name: "Sweep".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![
                    Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
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
                    Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Draw(1.into())],
                    },
                ],
                ..Default::default()
            },
        ),
        (
            HOARDER,
            CardDef {
                name: "Hoarder".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 1,
                triggers: vec![TriggerDef {
                    pattern: TriggerPattern::Died(TriggerSubject::Itself),
                    effects: vec![
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
                    ],
                }],
                ..Default::default()
            },
        ),
        (
            DRUMMER,
            CardDef {
                name: "Drummer".to_string(),
                kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                mana_cost: 2,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Minion),
                        side: EnumSet::only(Side::Friendly),
                    }),
                    verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffHealth {
                        amount: 1.into(),
                    })],
                }],
                ..Default::default()
            },
        ),
        (
            FRENZY,
            CardDef {
                name: "Frenzy".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Random(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Minion),
                        side: EnumSet::only(Side::Friendly),
                    }),
                    verbs: vec![
                        CharacterVerb::Damage(1.into()),
                        CharacterVerb::Modify(ModifierEffect::BuffAtk { amount: 2.into() }),
                    ],
                }],
                ..Default::default()
            },
        ),
        (
            GOAD,
            CardDef {
                name: "Goad".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Targeted(Choice {
                        options: CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                        },
                        count: 1.into(),
                        bound: AmountBound::Exactly,
                        different: true,
                    }),
                    verbs: vec![
                        CharacterVerb::Damage(2.into()),
                        CharacterVerb::Modify(ModifierEffect::BuffAtk { amount: 2.into() }),
                    ],
                }],
                ..Default::default()
            },
        ),
        (
            AFTERSHOCK,
            CardDef {
                name: "Aftershock".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 2,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::All(CharacterFilter {
                        kind: EnumSet::only(CharacterKind::Minion),
                        side: EnumSet::only(Side::Enemy),
                    }),
                    verbs: vec![
                        CharacterVerb::Damage(2.into()),
                        CharacterVerb::Damage(1.into()),
                    ],
                }],
                ..Default::default()
            },
        ),
        (
            DUD,
            CardDef {
                name: "Dud".to_string(),
                kind: CardKindDef::Spell(SpellCardDef),
                mana_cost: 1,
                play_effects: vec![Effect::Characters {
                    selector: CharacterSelector::Random(CharacterFilter::all()),
                    verbs: Vec::new(),
                }],
                ..Default::default()
            },
        ),
    ]
}
