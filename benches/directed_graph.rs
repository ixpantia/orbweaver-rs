use criterion::{black_box, criterion_group, criterion_main, Criterion};
use orbweaver::directed::{DirectedGraph, DirectedGraphBuilder};
use std::io::{prelude::*, BufReader};

pub fn download() -> DirectedGraph {
    let url = "https://snap.stanford.edu/data/web-BerkStan.txt.gz";
    let response = ureq::get(url).call().unwrap().into_reader();
    let decoder = BufReader::new(flate2::read::GzDecoder::new(response));
    let mut builder = DirectedGraphBuilder::new();
    decoder
        .lines()
        .map_while(Result::ok)
        .skip_while(|l| l.starts_with('#'))
        .for_each(|l| {
            if let Some((p, c)) = l.split_once('\t') {
                builder.add_edge(p, c);
            }
        });
    builder.build_directed()
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let graph = download();

    println!("Done building the graph!");

    c.bench_function("dg_get_parents", |b| {
        b.iter(|| graph.parents(black_box(["11"])))
    });

    c.bench_function("dg_get_children", |b| {
        b.iter(|| graph.children(black_box(["11"])))
    });

    c.bench_function("dg_get_parents_many", |b| {
        b.iter(|| graph.parents(black_box(["11", "1"])))
    });

    c.bench_function("dg_get_children_many", |b| {
        b.iter(|| graph.children([black_box("11"), black_box("1")]))
    });

    c.bench_function("dg_has_children", |b| {
        b.iter(|| graph.has_children([black_box("11"), black_box("1")]))
    });

    c.bench_function("dg_has_parents", |b| {
        b.iter(|| graph.has_parents([black_box("11"), black_box("1")]))
    });

    c.bench_function("dg_find_path", |b| {
        b.iter(|| graph.find_path(black_box("1"), black_box("11")))
    });

    c.bench_function("dg_get_all_leaves", |b| b.iter(|| graph.get_all_leaves()));

    c.bench_function("dg_get_leaves_under", |b| {
        b.iter(|| graph.get_leaves_under(black_box(["1"])))
    });

    c.bench_function("dg_get_all_roots", |b| b.iter(|| graph.get_all_roots()));

    c.bench_function("dg_get_roots_over", |b| {
        b.iter(|| graph.get_roots_over(black_box(["11"])))
    });

    c.bench_function("dg_subset_graph", |b| {
        b.iter(|| graph.subset(black_box("11")))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}

criterion_main!(benches);
