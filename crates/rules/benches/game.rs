//! Wall-clock benchmarks for the public `Game` interface over seeded random playouts.

mod support;

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use support::{
    DeckFn, PLAYOUT_SEEDS, STAGE_SEED, decider, drafting_deck, drafting_stages, new_game,
    play_to_end, sample_deck, stages,
};

fn bench_new(c: &mut Criterion) {
    let mut group = c.benchmark_group("new");
    group.bench_function("sample", |b| {
        b.iter(|| new_game(sample_deck, black_box(STAGE_SEED)))
    });
    group.finish();
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
    let mut group = c.benchmark_group("playout");
    for (id, deck) in [
        ("sample", sample_deck as DeckFn),
        ("drafting", drafting_deck),
    ] {
        group.bench_function(id, |b| {
            b.iter(|| {
                for seed in PLAYOUT_SEEDS {
                    black_box(play_to_end(deck, black_box(seed)));
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
