//! Graph benchmark

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use mindfry::arena::LineageId;
use mindfry::graph::{Bond, BondGraph};

fn bench_bond_graph_connect(c: &mut Criterion) {
    c.bench_function("bond_graph_connect_1000", |b| {
        b.iter(|| {
            let mut graph = BondGraph::with_capacity(1000, 10000);
            for i in 0..1000 {
                let src = LineageId((i % 100) as u32);
                let tgt = LineageId(((i + 1) % 100) as u32);
                graph.connect(Bond::new(src, tgt, 0.5));
            }
            black_box(graph.len())
        })
    });
}

fn bench_bond_graph_neighbors(c: &mut Criterion) {
    let mut graph = BondGraph::with_capacity(1000, 10000);

    // Create a dense graph
    for i in 0..100 {
        for j in (i + 1)..100 {
            graph.connect(Bond::new(LineageId(i), LineageId(j), 0.5));
        }
    }

    c.bench_function("bond_graph_neighbors_lookup", |b| {
        b.iter(|| {
            let neighbors: Vec<_> = graph.neighbors_with_strength(LineageId(50)).collect();
            black_box(neighbors.len())
        })
    });
}

fn bench_bond_find(c: &mut Criterion) {
    let mut graph = BondGraph::with_capacity(1000, 10000);

    for i in 0..100 {
        for j in (i + 1)..100 {
            graph.connect(Bond::new(LineageId(i), LineageId(j), 0.5));
        }
    }

    c.bench_function("bond_find_between", |b| {
        b.iter(|| black_box(graph.find_bond(LineageId(25), LineageId(75))))
    });
}

criterion_group!(
    benches,
    bench_bond_graph_connect,
    bench_bond_graph_neighbors,
    bench_bond_find,
);

criterion_main!(benches);
