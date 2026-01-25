//! Stability Layer Benchmarks
//!
//! Benchmarks for crash recovery, warmup tracking, and exhaustion monitoring.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mindfry::stability::{
    ExhaustionLevel, ExhaustionMonitor, RecoveryAnalyzer, RecoveryState, ShutdownMarker,
    WarmupTracker,
};

// ═══════════════════════════════════════════════════════════════
// RECOVERY BENCHMARKS
// ═══════════════════════════════════════════════════════════════

fn bench_recovery_analyzer_new(c: &mut Criterion) {
    let marker = Some(ShutdownMarker::graceful());

    c.bench_function("recovery_analyzer_new", |b| {
        b.iter(|| black_box(RecoveryAnalyzer::new(marker.clone())))
    });
}

fn bench_recovery_analyzer_analyze(c: &mut Criterion) {
    let marker = Some(ShutdownMarker::graceful());
    let analyzer = RecoveryAnalyzer::new(marker);

    c.bench_function("recovery_analyzer_analyze", |b| {
        b.iter(|| black_box(analyzer.analyze()))
    });
}

fn bench_recovery_state_intensity(c: &mut Criterion) {
    c.bench_function("recovery_state_intensity", |b| {
        b.iter(|| {
            black_box(RecoveryState::Normal.intensity());
            black_box(RecoveryState::Shock.intensity());
            black_box(RecoveryState::Coma.intensity());
        })
    });
}

// ═══════════════════════════════════════════════════════════════
// WARMUP BENCHMARKS
// ═══════════════════════════════════════════════════════════════

fn bench_warmup_tracker_new(c: &mut Criterion) {
    c.bench_function("warmup_tracker_new", |b| {
        b.iter(|| black_box(WarmupTracker::new()))
    });
}

fn bench_warmup_tracker_is_ready(c: &mut Criterion) {
    let tracker = WarmupTracker::new();

    c.bench_function("warmup_tracker_is_ready", |b| {
        b.iter(|| black_box(tracker.is_ready()))
    });
}

fn bench_warmup_tracker_state(c: &mut Criterion) {
    let tracker = WarmupTracker::new();

    c.bench_function("warmup_tracker_state", |b| {
        b.iter(|| black_box(tracker.state()))
    });
}

fn bench_warmup_tracker_clone(c: &mut Criterion) {
    let tracker = WarmupTracker::new();

    c.bench_function("warmup_tracker_clone", |b| {
        b.iter(|| black_box(tracker.clone()))
    });
}

// ═══════════════════════════════════════════════════════════════
// EXHAUSTION BENCHMARKS
// ═══════════════════════════════════════════════════════════════

fn bench_exhaustion_level_from_energy(c: &mut Criterion) {
    c.bench_function("exhaustion_level_from_energy", |b| {
        b.iter(|| {
            black_box(ExhaustionLevel::from_energy(0.9));
            black_box(ExhaustionLevel::from_energy(0.5));
            black_box(ExhaustionLevel::from_energy(0.1));
        })
    });
}

fn bench_exhaustion_allows_writes(c: &mut Criterion) {
    c.bench_function("exhaustion_allows_writes", |b| {
        b.iter(|| {
            black_box(ExhaustionLevel::Normal.allows_writes());
            black_box(ExhaustionLevel::Elevated.allows_writes());
            black_box(ExhaustionLevel::Exhausted.allows_writes());
            black_box(ExhaustionLevel::Emergency.allows_writes());
        })
    });
}

fn bench_exhaustion_monitor_default(c: &mut Criterion) {
    c.bench_function("exhaustion_monitor_new", |b| {
        b.iter(|| black_box(ExhaustionMonitor::default()))
    });
}

criterion_group!(
    benches,
    // Recovery
    bench_recovery_analyzer_new,
    bench_recovery_analyzer_analyze,
    bench_recovery_state_intensity,
    // Warmup
    bench_warmup_tracker_new,
    bench_warmup_tracker_is_ready,
    bench_warmup_tracker_state,
    bench_warmup_tracker_clone,
    // Exhaustion
    bench_exhaustion_level_from_energy,
    bench_exhaustion_allows_writes,
    bench_exhaustion_monitor_default,
);

criterion_main!(benches);
