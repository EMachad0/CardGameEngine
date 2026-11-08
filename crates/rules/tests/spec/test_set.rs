//! The test set: every card the crate tests share, under the prefix `test`.

use enumset::EnumSet;
use rules::prelude::{
    AmountBound, CardDef, CardDestinationZone, CardFilter, CardKind, CardKindDef, CardPickOptions,
    CardPickZone, CardSelector, CardTargetOptions, CardTargetZone, CardVerb, CharacterFilter,
    CharacterKind, CharacterSelector, CharacterVerb, Choice, DeckWindow, DefCode, Effect,
    EffectAmount, HeroCardDef, HistoryQuery, HistoryQueryKind, MinionCardDef, ModifierEffect,
    PlayerFilter, PlayerSelector, PlayerVerb, Set, SetCard, Side, SpellCardDef, TriggerDef,
    TriggerPattern, TriggerSubject, TurnFilter,
};

const PREFIX: &str = "test";

const fn code(name: &'static str) -> DefCode {
    DefCode {
        set: PREFIX,
        name,
        version: 0,
    }
}

pub(crate) const CAPTAIN: DefCode = code("captain");
pub(crate) const BLAST: DefCode = code("blast");
pub(crate) const RECRUIT: DefCode = code("recruit");
pub(crate) const GIANT: DefCode = code("giant");
pub(crate) const SPARK: DefCode = code("spark");
pub(crate) const BOLT: DefCode = code("bolt");
pub(crate) const FORAGE: DefCode = code("forage");
pub(crate) const WILD_BOLT: DefCode = code("wild_bolt");
pub(crate) const HERO: DefCode = code("hero");
pub(crate) const SQUIRE: DefCode = code("squire");
pub(crate) const BARRACKS: DefCode = code("barracks");
pub(crate) const ZAP: DefCode = code("zap");
pub(crate) const STRAY_SHOT: DefCode = code("stray_shot");
pub(crate) const PING: DefCode = code("ping");
pub(crate) const SHOVE: DefCode = code("shove");
pub(crate) const TWIN_SHOT: DefCode = code("twin_shot");
pub(crate) const CROSSFIRE: DefCode = code("crossfire");
pub(crate) const MEDIC: DefCode = code("medic");
pub(crate) const MOURNER: DefCode = code("mourner");
pub(crate) const DRILLMASTER: DefCode = code("drillmaster");
pub(crate) const RALLY_BANNER: DefCode = code("rally_banner");
pub(crate) const BOMBER: DefCode = code("bomber");
pub(crate) const GRAVEDIGGER: DefCode = code("gravedigger");
pub(crate) const SWEEP: DefCode = code("sweep");
pub(crate) const HOARDER: DefCode = code("hoarder");
pub(crate) const DRUMMER: DefCode = code("drummer");
pub(crate) const FRENZY: DefCode = code("frenzy");
pub(crate) const GOAD: DefCode = code("goad");
pub(crate) const AFTERSHOCK: DefCode = code("aftershock");
pub(crate) const DUD: DefCode = code("dud");
pub(crate) const RECYCLE: DefCode = code("recycle");
pub(crate) const MUSTER: DefCode = code("muster");
pub(crate) const PACKRAT: DefCode = code("packrat");
pub(crate) const TRIAGE: DefCode = code("triage");
pub(crate) const ENLIST: DefCode = code("enlist");
pub(crate) const SHARPEN: DefCode = code("sharpen");
pub(crate) const ROUSE: DefCode = code("rouse");
pub(crate) const MENTOR: DefCode = code("mentor");
pub(crate) const TREMOR: DefCode = code("tremor");
pub(crate) const KNIFE_JUGGLER: DefCode = code("knife_juggler");
pub(crate) const IMAGE: DefCode = code("image");
pub(crate) const MIRROR_IMAGE: DefCode = code("mirror_image");
pub(crate) const JUGGLING_ACT: DefCode = code("juggling_act");
pub(crate) const JUGGLING_LINE: DefCode = code("juggling_line");
pub(crate) const COURIER: DefCode = code("courier");
pub(crate) const HERALD: DefCode = code("herald");
pub(crate) const RINGMASTER: DefCode = code("ringmaster");
pub(crate) const DISPATCH: DefCode = code("dispatch");
pub(crate) const TINKER: DefCode = code("tinker");

fn card(code: DefCode, def: CardDef) -> SetCard {
    SetCard {
        name: code.name,
        version: code.version,
        def,
    }
}

pub(crate) fn test_set() -> Set {
    Set {
        prefix: PREFIX,
        cards: vec![
            card(
                SPARK,
                CardDef {
                    name: "Spark".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Hero),
                            side: EnumSet::only(Side::Enemy),
                            include_itself: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    }],
                    ..Default::default()
                },
            ),
            card(
                BOLT,
                CardDef {
                    name: "Bolt".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Hero),
                            side: EnumSet::only(Side::Enemy),
                            include_itself: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(2.into())],
                    }],
                    ..Default::default()
                },
            ),
            card(
                WILD_BOLT,
                CardDef {
                    name: "Wild Bolt".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::Random(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Hero),
                            side: EnumSet::all(),
                            include_itself: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(3.into())],
                    }],
                    ..Default::default()
                },
            ),
            card(
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
            card(
                FORAGE,
                CardDef {
                    name: "Forage".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Cards {
                        selector: CardSelector::Picked {
                            choice: Choice {
                                options: CardPickOptions {
                                    zone: CardPickZone::YourDeck(DeckWindow::Top(2)),
                                    kind: EnumSet::all(),
                                },
                                count: 1.into(),
                                bound: AmountBound::Exactly,
                                different: true,
                            },
                            others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                        },
                        verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
                    }],
                    ..Default::default()
                },
            ),
            card(
                RECRUIT,
                CardDef {
                    name: "Recruit".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(2, 2)),
                    mana_cost: 2,
                    ..Default::default()
                },
            ),
            card(
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
            card(
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
            card(
                HERO,
                CardDef {
                    name: "Hero".to_string(),
                    kind: CardKindDef::Hero(HeroCardDef { health: 10 }),
                    ..Default::default()
                },
            ),
            card(
                SQUIRE,
                CardDef {
                    name: "Squire".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                    mana_cost: 1,
                    ..Default::default()
                },
            ),
            card(
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
            card(
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
                                include_itself: true,
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
            card(
                STRAY_SHOT,
                CardDef {
                    name: "Stray Shot".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::Random(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::only(Side::Enemy),
                            include_itself: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(2.into())],
                    }],
                    ..Default::default()
                },
            ),
            card(
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
                                include_itself: true,
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
            card(
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
                                    include_itself: true,
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
                                    include_itself: true,
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
            card(
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
                                include_itself: true,
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
            card(
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
                                    include_itself: true,
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
                                    include_itself: true,
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
            card(
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
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffHealth {
                                amount: 2.into(),
                            })],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                MOURNER,
                CardDef {
                    name: "Mourner".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 3)),
                    mana_cost: 3,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Died(TriggerSubject::Character(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::only(Side::Friendly),
                            include_itself: true,
                        })),
                        effects: vec![Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(1.into())],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                DRILLMASTER,
                CardDef {
                    name: "Drillmaster".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(2, 3)),
                    mana_cost: 3,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Played(CardFilter {
                            kind: EnumSet::only(CardKind::Spell),
                            side: EnumSet::only(Side::Friendly),
                            include_itself: true,
                        }),
                        effects: vec![Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Summon(SQUIRE)],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
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
            card(
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
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Damage(1.into())],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                GRAVEDIGGER,
                CardDef {
                    name: "Gravedigger".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(2, 2)),
                    mana_cost: 2,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Died(TriggerSubject::Character(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                            include_itself: true,
                        })),
                        effects: vec![Effect::Characters {
                            selector: CharacterSelector::All(CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Hero),
                                side: EnumSet::only(Side::Enemy),
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Damage(1.into())],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
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
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Damage(1.into())],
                        },
                        Effect::Cards {
                            selector: CardSelector::Picked {
                                choice: Choice {
                                    options: CardPickOptions {
                                        zone: CardPickZone::YourDeck(DeckWindow::Top(2)),
                                        kind: EnumSet::all(),
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                            },
                            verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
                        },
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(1.into())],
                        },
                    ],
                    ..Default::default()
                },
            ),
            card(
                HOARDER,
                CardDef {
                    name: "Hoarder".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                    mana_cost: 1,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Died(TriggerSubject::Itself),
                        effects: vec![Effect::Cards {
                            selector: CardSelector::Picked {
                                choice: Choice {
                                    options: CardPickOptions {
                                        zone: CardPickZone::YourDeck(DeckWindow::Top(2)),
                                        kind: EnumSet::all(),
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                            },
                            verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                DRUMMER,
                CardDef {
                    name: "Drummer".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                    mana_cost: 2,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::only(Side::Friendly),
                            include_itself: true,
                        }),
                        verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffHealth {
                            amount: 1.into(),
                        })],
                    }],
                    ..Default::default()
                },
            ),
            card(
                FRENZY,
                CardDef {
                    name: "Frenzy".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::Random(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::only(Side::Friendly),
                            include_itself: true,
                        }),
                        verbs: vec![
                            CharacterVerb::Damage(1.into()),
                            CharacterVerb::Modify(ModifierEffect::BuffAtk { amount: 2.into() }),
                        ],
                    }],
                    ..Default::default()
                },
            ),
            card(
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
                                include_itself: true,
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
            card(
                AFTERSHOCK,
                CardDef {
                    name: "Aftershock".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::only(Side::Enemy),
                            include_itself: true,
                        }),
                        verbs: vec![
                            CharacterVerb::Damage(2.into()),
                            CharacterVerb::Damage(1.into()),
                        ],
                    }],
                    ..Default::default()
                },
            ),
            card(
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
            card(
                RECYCLE,
                CardDef {
                    name: "Recycle".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![
                        Effect::Cards {
                            selector: CardSelector::Targeted(Choice {
                                options: CardTargetOptions {
                                    zone: CardTargetZone::Hand,
                                    kind: EnumSet::only(CardKind::Minion),
                                },
                                count: 1.into(),
                                bound: AmountBound::Exactly,
                                different: true,
                            }),
                            verbs: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                        },
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(EffectAmount::Static(1))],
                        },
                    ],
                    ..Default::default()
                },
            ),
            card(
                MUSTER,
                CardDef {
                    name: "Muster".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Cards {
                        selector: CardSelector::Picked {
                            choice: Choice {
                                options: CardPickOptions {
                                    zone: CardPickZone::YourDeck(DeckWindow::Top(3)),
                                    kind: EnumSet::only(CardKind::Minion),
                                },
                                count: 1.into(),
                                bound: AmountBound::Exactly,
                                different: true,
                            },
                            others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                        },
                        verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
                    }],
                    ..Default::default()
                },
            ),
            card(
                PACKRAT,
                CardDef {
                    name: "Packrat".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 4)),
                    mana_cost: 2,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Damaged(TriggerSubject::Itself),
                        effects: vec![Effect::Cards {
                            selector: CardSelector::Picked {
                                choice: Choice {
                                    options: CardPickOptions {
                                        zone: CardPickZone::YourDeck(DeckWindow::Top(2)),
                                        kind: EnumSet::all(),
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                            },
                            verbs: vec![CardVerb::Move(CardDestinationZone::Hand)],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                TRIAGE,
                CardDef {
                    name: "Triage".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![
                        Effect::Characters {
                            selector: CharacterSelector::All(CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Minion),
                                side: EnumSet::all(),
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Damage(1.into())],
                        },
                        Effect::Characters {
                            selector: CharacterSelector::Picked {
                                choice: Choice {
                                    options: CharacterFilter {
                                        kind: EnumSet::only(CharacterKind::Minion),
                                        side: EnumSet::all(),
                                        include_itself: true,
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: Vec::new(),
                            },
                            verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffHealth {
                                amount: 2.into(),
                            })],
                        },
                    ],
                    ..Default::default()
                },
            ),
            card(
                ENLIST,
                CardDef {
                    name: "Enlist".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Summon(SQUIRE)],
                        },
                        Effect::Characters {
                            selector: CharacterSelector::Picked {
                                choice: Choice {
                                    options: CharacterFilter {
                                        kind: EnumSet::only(CharacterKind::Minion),
                                        side: EnumSet::all(),
                                        include_itself: true,
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: Vec::new(),
                            },
                            verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffAtk {
                                amount: 2.into(),
                            })],
                        },
                    ],
                    ..Default::default()
                },
            ),
            card(
                SHARPEN,
                CardDef {
                    name: "Sharpen".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![
                        Effect::Characters {
                            selector: CharacterSelector::Picked {
                                choice: Choice {
                                    options: CharacterFilter {
                                        kind: EnumSet::only(CharacterKind::Minion),
                                        side: EnumSet::all(),
                                        include_itself: true,
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: Vec::new(),
                            },
                            verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffAtk {
                                amount: 2.into(),
                            })],
                        },
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(1.into())],
                        },
                    ],
                    ..Default::default()
                },
            ),
            card(
                ROUSE,
                CardDef {
                    name: "Rouse".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::Picked {
                            choice: Choice {
                                options: CharacterFilter {
                                    kind: EnumSet::only(CharacterKind::Minion),
                                    side: EnumSet::all(),
                                    include_itself: true,
                                },
                                count: 1.into(),
                                bound: AmountBound::Exactly,
                                different: true,
                            },
                            others: vec![CharacterVerb::Modify(ModifierEffect::BuffAtk {
                                amount: 1.into(),
                            })],
                        },
                        verbs: Vec::new(),
                    }],
                    ..Default::default()
                },
            ),
            card(
                MENTOR,
                CardDef {
                    name: "Mentor".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                    mana_cost: 2,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Died(TriggerSubject::Itself),
                        effects: vec![Effect::Characters {
                            selector: CharacterSelector::Picked {
                                choice: Choice {
                                    options: CharacterFilter {
                                        kind: EnumSet::only(CharacterKind::Minion),
                                        side: EnumSet::only(Side::Friendly),
                                        include_itself: true,
                                    },
                                    count: 1.into(),
                                    bound: AmountBound::Exactly,
                                    different: true,
                                },
                                others: Vec::new(),
                            },
                            verbs: vec![CharacterVerb::Modify(ModifierEffect::BuffAtk {
                                amount: 2.into(),
                            })],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                TREMOR,
                CardDef {
                    name: "Tremor".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(2, 2)),
                    mana_cost: 2,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Minion),
                            side: EnumSet::all(),
                            include_itself: false,
                        }),
                        verbs: vec![CharacterVerb::Damage(1.into())],
                    }],
                    ..Default::default()
                },
            ),
            card(
                KNIFE_JUGGLER,
                CardDef {
                    name: "Knife Juggler".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(2, 1)),
                    mana_cost: 2,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Summoned(TriggerSubject::Character(
                            CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Minion),
                                side: EnumSet::only(Side::Friendly),
                                include_itself: false,
                            },
                        )),
                        effects: vec![Effect::Characters {
                            selector: CharacterSelector::Random(CharacterFilter {
                                kind: EnumSet::all(),
                                side: EnumSet::only(Side::Enemy),
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Damage(1.into())],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                IMAGE,
                CardDef {
                    name: "Image".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(0, 2)),
                    mana_cost: 0,
                    ..Default::default()
                },
            ),
            card(
                MIRROR_IMAGE,
                CardDef {
                    name: "Mirror Image".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Summon(IMAGE), PlayerVerb::Summon(IMAGE)],
                    }],
                    ..Default::default()
                },
            ),
            card(
                JUGGLING_ACT,
                CardDef {
                    name: "Juggling Act".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![
                            PlayerVerb::Summon(KNIFE_JUGGLER),
                            PlayerVerb::Summon(KNIFE_JUGGLER),
                        ],
                    }],
                    ..Default::default()
                },
            ),
            card(
                JUGGLING_LINE,
                CardDef {
                    name: "Juggling Line".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Summon(KNIFE_JUGGLER)],
                        },
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Summon(KNIFE_JUGGLER)],
                        },
                    ],
                    ..Default::default()
                },
            ),
            card(
                COURIER,
                CardDef {
                    name: "Courier".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 1)),
                    mana_cost: 2,
                    play_effects: vec![Effect::Characters {
                        selector: CharacterSelector::All(CharacterFilter {
                            kind: EnumSet::only(CharacterKind::Hero),
                            side: EnumSet::only(Side::Enemy),
                            include_itself: true,
                        }),
                        verbs: vec![CharacterVerb::Damage(2.into())],
                    }],
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Summoned(TriggerSubject::Itself),
                        effects: vec![Effect::Characters {
                            selector: CharacterSelector::All(CharacterFilter {
                                kind: EnumSet::only(CharacterKind::Hero),
                                side: EnumSet::only(Side::Enemy),
                                include_itself: true,
                            }),
                            verbs: vec![CharacterVerb::Damage(3.into())],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                HERALD,
                CardDef {
                    name: "Herald".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(1, 2)),
                    mana_cost: 2,
                    triggers: vec![TriggerDef {
                        pattern: TriggerPattern::Played(CardFilter {
                            kind: EnumSet::only(CardKind::Minion),
                            side: EnumSet::only(Side::Friendly),
                            include_itself: false,
                        }),
                        effects: vec![Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(1.into())],
                        }],
                    }],
                    ..Default::default()
                },
            ),
            card(
                RINGMASTER,
                CardDef {
                    name: "Ringmaster".to_string(),
                    kind: CardKindDef::Minion(MinionCardDef::new(2, 2)),
                    mana_cost: 3,
                    play_effects: vec![Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Summon(KNIFE_JUGGLER)],
                    }],
                    ..Default::default()
                },
            ),
            card(
                DISPATCH,
                CardDef {
                    name: "Dispatch".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 2,
                    play_effects: vec![Effect::Players {
                        selector: PlayerSelector::Owner,
                        verbs: vec![PlayerVerb::Summon(COURIER)],
                    }],
                    ..Default::default()
                },
            ),
            card(
                TINKER,
                CardDef {
                    name: "Tinker".to_string(),
                    kind: CardKindDef::Spell(SpellCardDef),
                    mana_cost: 1,
                    play_effects: vec![
                        Effect::Cards {
                            selector: CardSelector::Targeted(Choice {
                                options: CardTargetOptions {
                                    zone: CardTargetZone::Hand,
                                    kind: EnumSet::all(),
                                },
                                count: 1.into(),
                                bound: AmountBound::Exactly,
                                different: true,
                            }),
                            verbs: vec![CardVerb::Move(CardDestinationZone::DeckBottom)],
                        },
                        Effect::Players {
                            selector: PlayerSelector::Owner,
                            verbs: vec![PlayerVerb::Draw(EffectAmount::Static(1))],
                        },
                    ],
                    ..Default::default()
                },
            ),
        ],
    }
}
