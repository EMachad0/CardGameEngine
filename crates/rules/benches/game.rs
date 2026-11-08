//! Benchmarks for the public `Game` interface over seeded random playouts.

use std::hint::black_box;
use std::ops::Range;
use std::sync::LazyLock;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use enumset::EnumSet;
use rules::prelude::{
    Action, AmountBound, Binder, CardDef, CardDestinationZone, CardKindDef, CardPickOptions,
    CardPickZone, CardSelector, CardVerb, CharacterFilter, CharacterKind, CharacterSelector,
    CharacterVerb, Choice, Deck, DeckWindow, DefCode, Effect, EffectAmount, Game, HeroCardDef,
    HistoryQuery, HistoryQueryKind, MinionCardDef, ModifierEffect, PlayerFilter, PlayerId, Rng,
    Set, SetCard, Side, SpellCardDef, TurnFilter,
};

const MAX_STEPS: usize = 5_000;
const STAGE_SEED: u64 = 0;
const PLAYOUT_SEEDS: Range<u64> = 0..10;

const PREFIX: &str = "bench";

const fn code(name: &'static str) -> DefCode {
    DefCode {
        set: PREFIX,
        name,
        version: 0,
    }
}

const SPARK: DefCode = code("spark");
const BOLT: DefCode = code("bolt");
const WILD_BOLT: DefCode = code("wild_bolt");
const FORAGE: DefCode = code("forage");
const BLAST: DefCode = code("blast");
const RECRUIT: DefCode = code("recruit");
const CAPTAIN: DefCode = code("captain");
const GIANT: DefCode = code("giant");
const PING: DefCode = code("ping");
const TWIN_SHOT: DefCode = code("twin_shot");
const CROSSFIRE: DefCode = code("crossfire");
const SHOVE: DefCode = code("shove");
const HERO: DefCode = code("hero");

fn card(code: DefCode, def: CardDef) -> SetCard {
    SetCard {
        name: code.name,
        version: code.version,
        def,
    }
}

/// The cards the decks use, and the hero.
fn bench_set() -> Set {
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
                HERO,
                CardDef {
                    name: "Hero".to_string(),
                    kind: CardKindDef::Hero(HeroCardDef { health: 10 }),
                    ..Default::default()
                },
            ),
        ],
    }
}

/// Built once, so the benches measure games and not the parse.
static BINDER: LazyLock<Binder> =
    LazyLock::new(|| Binder::new([bench_set()]).expect("the bench set should parse"));

fn sample_deck() -> Vec<DefCode> {
    vec![
        SPARK, BOLT, WILD_BOLT, FORAGE, BLAST, RECRUIT, CAPTAIN, GIANT, RECRUIT, CAPTAIN, SPARK,
        FORAGE, BLAST, RECRUIT, CAPTAIN, BOLT,
    ]
}

/// `sample_deck` plus one of each card that is played through targeting.
fn targeting_deck() -> Vec<DefCode> {
    let mut deck = sample_deck();
    deck.extend([PING, TWIN_SHOT, CROSSFIRE, SHOVE]);
    deck
}

fn new_game(deck: fn() -> Vec<DefCode>, seed: u64) -> Game {
    let deck = || Deck {
        hero: HERO,
        cards: deck(),
    };
    Game::new(seed, BINDER.clone(), [deck(), deck()])
        .expect("the decks should name bench set cards")
}

/// Plays `seed`'s game with `deck` for both players to the end, picking uniformly
/// among both players' listed actions, and calls `visit` on every state that still
/// has a decision.
fn play_out(deck: fn() -> Vec<DefCode>, seed: u64, mut visit: impl FnMut(&Game)) -> Game {
    let mut game = new_game(deck, seed);
    let mut picker = Rng::new(seed);
    for _ in 0..MAX_STEPS {
        let mut options: Vec<(PlayerId, Action)> = game
            .players()
            .into_iter()
            .flat_map(|p| game.legal_actions(p).into_iter().map(move |a| (p, a)))
            .collect();
        if options.is_empty() {
            return game;
        }
        visit(&game);
        let (p, a) = options.swap_remove(picker.below(options.len()));
        game.apply(p, a, &mut ())
            .expect("a listed action is accepted");
    }
    panic!("seed {seed}: no outcome within {MAX_STEPS} steps");
}

/// The first, middle and last decision states of one game.
fn stages() -> [(&'static str, Game); 3] {
    let mut states = Vec::new();
    play_out(sample_deck, STAGE_SEED, |game| states.push(game.clone()));
    [
        ("early", states[0].clone()),
        ("mid", states[states.len() / 2].clone()),
        ("late", states[states.len() - 1].clone()),
    ]
}

/// Across the targeting deck's playouts, the idle state that offers a `StartTargeting` with
/// the most minions on the boards, and the open targeting that offers the most `Target`s.
fn targeting_stages() -> [(&'static str, Game); 2] {
    let mut idle: Option<(usize, Game)> = None;
    let mut open: Option<(usize, Game)> = None;
    for seed in PLAYOUT_SEEDS {
        play_out(targeting_deck, seed, |game| {
            let actions = game.legal_actions(decider(game));
            let minions = game
                .players()
                .iter()
                .map(|&p| game.player_board(p).len())
                .sum();
            let target_actions = actions
                .iter()
                .filter(|a| matches!(a, Action::Target { .. }))
                .count();
            if actions
                .iter()
                .any(|a| matches!(a, Action::StartTargeting { .. }))
                && idle.as_ref().is_none_or(|(best, _)| minions > *best)
            {
                idle = Some((minions, game.clone()));
            }
            if target_actions > 0 && open.as_ref().is_none_or(|(best, _)| target_actions > *best) {
                open = Some((target_actions, game.clone()));
            }
        });
    }
    let idle = idle.expect("some playout offers targeting").1;
    let open = open.expect("some playout starts targeting").1;
    [("idle", idle), ("open", open)]
}

fn decider(game: &Game) -> PlayerId {
    game.players()
        .into_iter()
        .find(|&p| !game.legal_actions(p).is_empty())
        .expect("every stage has a decision")
}

fn bench_new(c: &mut Criterion) {
    c.bench_function("new", |b| {
        b.iter(|| new_game(sample_deck, black_box(STAGE_SEED)))
    });
}

fn bench_clone(c: &mut Criterion) {
    let mut group = c.benchmark_group("clone");
    for (stage, game) in stages() {
        group.bench_with_input(BenchmarkId::from_parameter(stage), &game, |b, game| {
            b.iter(|| black_box(game).clone())
        });
    }
    group.finish();
}

fn bench_eq(c: &mut Criterion) {
    let mut group = c.benchmark_group("eq");
    for (stage, game) in stages() {
        let copy = game.clone();
        group.bench_with_input(BenchmarkId::from_parameter(stage), &game, |b, game| {
            b.iter(|| black_box(game) == black_box(&copy))
        });
    }
    group.finish();
}

fn bench_legal_actions(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_actions");
    for (stage, game) in stages() {
        let p = decider(&game);
        group.bench_with_input(BenchmarkId::from_parameter(stage), &game, |b, game| {
            b.iter(|| black_box(game).legal_actions(black_box(p)))
        });
    }
    group.finish();
}

fn bench_legal_actions_targeting(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_actions_targeting");
    for (stage, game) in targeting_stages() {
        let p = decider(&game);
        group.bench_with_input(BenchmarkId::from_parameter(stage), &game, |b, game| {
            b.iter(|| black_box(game).legal_actions(black_box(p)))
        });
    }
    group.finish();
}

fn bench_playout(c: &mut Criterion) {
    let seeds = PLAYOUT_SEEDS;
    let mut group = c.benchmark_group("playout");
    for (prefix, deck) in [
        ("", sample_deck as fn() -> Vec<DefCode>),
        ("targeting_", targeting_deck),
    ] {
        let id = format!("{prefix}seeds_{}_to_{}", seeds.start, seeds.end - 1);
        group.bench_function(id, |b| {
            b.iter(|| {
                for seed in seeds.clone() {
                    black_box(play_out(deck, black_box(seed), |_| {}));
                }
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_new,
    bench_clone,
    bench_eq,
    bench_legal_actions,
    bench_legal_actions_targeting,
    bench_playout
);
criterion_main!(benches);
