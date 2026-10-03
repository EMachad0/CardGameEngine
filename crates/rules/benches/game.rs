//! Benchmarks for the public `Game` interface over seeded random playouts.

use std::hint::black_box;
use std::ops::Range;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use rules::cards::{BLAST, BOLT, CAPTAIN, FORAGE, GIANT, RECRUIT, SPARK, WILD_BOLT};
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

fn new_game(seed: u64) -> Game {
    Game::new(seed, [sample_deck(), sample_deck()])
}

/// Plays `seed`'s game to the end, picking uniformly among both players' listed
/// actions, and calls `visit` on every state that still has a decision.
fn play_out(seed: u64, mut visit: impl FnMut(&Game)) -> Game {
    let mut game = new_game(seed);
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
    play_out(STAGE_SEED, |game| states.push(game.clone()));
    [
        ("early", states[0].clone()),
        ("mid", states[states.len() / 2].clone()),
        ("late", states[states.len() - 1].clone()),
    ]
}

fn decider(game: &Game) -> PlayerId {
    game.players()
        .into_iter()
        .find(|&p| !game.legal_actions(p).is_empty())
        .expect("every stage has a decision")
}

fn bench_new(c: &mut Criterion) {
    c.bench_function("new", |b| b.iter(|| new_game(black_box(STAGE_SEED))));
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

fn bench_playout(c: &mut Criterion) {
    let seeds = PLAYOUT_SEEDS;
    let id = format!("seeds_{}_to_{}", seeds.start, seeds.end - 1);
    c.benchmark_group("playout").bench_function(id, |b| {
        b.iter(|| {
            for seed in seeds.clone() {
                black_box(play_out(black_box(seed), |_| {}));
            }
        })
    });
}

criterion_group!(
    benches,
    bench_new,
    bench_clone,
    bench_eq,
    bench_legal_actions,
    bench_playout
);
criterion_main!(benches);
