//! Benchmarks for the public `Game` interface over seeded random playouts.

use std::hint::black_box;
use std::ops::Range;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rules::static_card_definition::{
    BLAST, BOLT, CAPTAIN, CROSSFIRE, FORAGE, GIANT, PING, RECRUIT, SHOVE, SPARK, TWIN_SHOT,
    WILD_BOLT,
};
use rules::{Action, DefId, Game, PlayerId, Rng};

const MAX_STEPS: usize = 5_000;
const STAGE_SEED: u64 = 0;
const PLAYOUT_SEEDS: Range<u64> = 0..10;

fn sample_deck() -> Vec<DefId> {
    vec![
        SPARK, BOLT, WILD_BOLT, FORAGE, BLAST, RECRUIT, CAPTAIN, GIANT, RECRUIT, CAPTAIN, SPARK,
        FORAGE, BLAST, RECRUIT, CAPTAIN, BOLT,
    ]
}

/// `sample_deck` plus one of each card that is played through a draft.
fn drafting_deck() -> Vec<DefId> {
    let mut deck = sample_deck();
    deck.extend([PING, TWIN_SHOT, CROSSFIRE, SHOVE]);
    deck
}

fn new_game(deck: fn() -> Vec<DefId>, seed: u64) -> Game {
    Game::new(seed, [deck(), deck()])
}

/// Plays `seed`'s game with `deck` for both players to the end, picking uniformly
/// among both players' listed actions, and calls `visit` on every state that still
/// has a decision.
fn play_out(deck: fn() -> Vec<DefId>, seed: u64, mut visit: impl FnMut(&Game)) -> Game {
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

/// Across the drafting deck's playouts, the idle state that offers a `Draft` with the
/// most minions on the boards, and the open draft that offers the most `Choose`s.
fn drafting_stages() -> [(&'static str, Game); 2] {
    let mut idle: Option<(usize, Game)> = None;
    let mut open: Option<(usize, Game)> = None;
    for seed in PLAYOUT_SEEDS {
        play_out(drafting_deck, seed, |game| {
            let actions = game.legal_actions(decider(game));
            let minions = game.players().iter().map(|&p| game.board(p).len()).sum();
            let chooses = actions
                .iter()
                .filter(|a| matches!(a, Action::Choose { .. }))
                .count();
            if actions.iter().any(|a| matches!(a, Action::Draft { .. }))
                && idle.as_ref().is_none_or(|(best, _)| minions > *best)
            {
                idle = Some((minions, game.clone()));
            }
            if chooses > 0 && open.as_ref().is_none_or(|(best, _)| chooses > *best) {
                open = Some((chooses, game.clone()));
            }
        });
    }
    let idle = idle.expect("some playout offers a draft").1;
    let open = open.expect("some playout opens a draft").1;
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

fn bench_legal_actions_drafting(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_actions_drafting");
    for (stage, game) in drafting_stages() {
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
        ("", sample_deck as fn() -> Vec<DefId>),
        ("drafting_", drafting_deck),
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
    bench_legal_actions_drafting,
    bench_playout
);
criterion_main!(benches);
