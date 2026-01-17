//! Decay benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mindfry::arena::{Lineage, PsycheArena};
use mindfry::dynamics::{DecayConfig, DecayEngine};

fn bench_decay_lut_lookup(c: &mut Criterion) {
    let engine = DecayEngine::default();

    c.bench_function("decay_lut_lookup", |b| {
        b.iter(|| black_box(engine.decay_factor(0.001, 100.0)))
    });
}

fn bench_lineage_current_energy(c: &mut Criterion) {
    let lineage = Lineage::with_config(0.8, 0.5, 0.001);

    c.bench_function("lineage_current_energy", |b| {
        b.iter(|| black_box(lineage.current_energy()))
    });
}

fn bench_psyche_arena_alloc(c: &mut Criterion) {
    c.bench_function("psyche_arena_alloc_1000", |b| {
        b.iter(|| {
            let mut arena = PsycheArena::with_capacity(1000);
            for i in 0..1000 {
                arena.alloc(Lineage::new(0.5 + (i as f32) * 0.0001));
            }
            black_box(arena.len())
        })
    });
}

fn bench_decay_tick_1m(c: &mut Criterion) {
    let mut arena = PsycheArena::with_capacity(1_000_000);
    for i in 0..100_000 {
        arena.alloc(Lineage::new(0.5 + (i as f32) * 0.000001));
    }

    let mut engine = DecayEngine::new(DecayConfig {
        parallel: true,
        ..Default::default()
    });

    c.bench_function("decay_tick_100k_lineages", |b| {
        b.iter(|| black_box(engine.tick_psyche(&mut arena)))
    });
}

criterion_group!(
    benches,
    bench_decay_lut_lookup,
    bench_lineage_current_energy,
    bench_psyche_arena_alloc,
    bench_decay_tick_1m,
);

criterion_main!(benches);
