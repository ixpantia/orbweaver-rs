use criterion::{black_box, criterion_group, criterion_main, Criterion};
use orbweaver::directed::{acyclic::DirectedAcyclicGraph, DirectedGraphBuilder};
use std::io::{prelude::*};

const MEDIUM_TXT_PATH: &str = "assets/medium.txt";

fn get_medium_graph() -> DirectedAcyclicGraph {
    use std::io::BufRead;
    let mut builder = DirectedGraphBuilder::new();
    std::io::BufReader::new(
        std::fs::File::open(MEDIUM_TXT_PATH).expect("Unable to read medium.txt"),
    )
    .lines()
    .map_while(Result::ok)
    .for_each(|l| {
        if let Some((parent, child)) = l.split_once('\t') {
            builder.add_edge(parent, child);
        }
    });

    builder.build_acyclic().expect("This should work")
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let graph_dag = get_medium_graph();
    let graph_dg = graph_dag.clone().into_inner();

    println!("Done building the graph!");

    c.bench_function("dg_get_parents", |b| {
        b.iter(|| graph_dg.parents(black_box(["1f6a329de1d9c26602fe1ee8ce81ca98"])))
    });

    c.bench_function("dg_get_children", |b| {
        b.iter(|| graph_dg.children(black_box(["1f6a329de1d9c26602fe1ee8ce81ca98"])))
    });

    c.bench_function("dg_get_parents_many", |b| {
        b.iter(|| {
            graph_dg.parents(black_box([
                "542ca75c803830d5e4f0c020fc52622a",
                "82c278b0565082eecb00d74c2bd51ff0",
            ]))
        })
    });

    c.bench_function("dg_get_children_many", |b| {
        b.iter(|| {
            graph_dg.children(black_box([
                "542ca75c803830d5e4f0c020fc52622a",
                "82c278b0565082eecb00d74c2bd51ff0",
            ]))
        })
    });

    c.bench_function("dg_has_children", |b| {
        b.iter(|| {
            graph_dg.has_children(black_box([
                "542ca75c803830d5e4f0c020fc52622a",
                "82c278b0565082eecb00d74c2bd51ff0",
            ]))
        })
    });

    c.bench_function("dg_has_parents", |b| {
        b.iter(|| {
            graph_dg.has_parents(black_box([
                "542ca75c803830d5e4f0c020fc52622a",
                "82c278b0565082eecb00d74c2bd51ff0",
            ]))
        })
    });

    c.bench_function("dg_get_all_leaves", |b| {
        b.iter(|| graph_dg.get_all_leaves())
    });

    c.bench_function("dg_get_leaves_under", |b| {
        b.iter(|| graph_dg.get_leaves_under(black_box(["1781f676dedf5767f3243db0a9738b35"])))
    });

    c.bench_function("dg_get_all_roots", |b| b.iter(|| graph_dg.get_all_roots()));

    c.bench_function("dg_get_roots_over", |b| {
        b.iter(|| graph_dg.get_roots_over(black_box(["eb85851afd251bd7c7eaf725d0d19360"])))
    });

    c.bench_function("dg_subset_graph_dg", |b| {
        b.iter(|| graph_dg.subset(black_box("1781f676dedf5767f3243db0a9738b35")))
    });

    c.bench_function("dg_find_path", |b| {
        b.iter(|| {
            graph_dg.find_path(
                black_box("1781f676dedf5767f3243db0a9738b35"),
                black_box("eb85851afd251bd7c7eaf725d0d19360"),
            )
        })
    });

    c.bench_function("dag_find_path", |b| {
        b.iter(|| {
            graph_dag.find_path(
                black_box("1781f676dedf5767f3243db0a9738b35"),
                black_box("eb85851afd251bd7c7eaf725d0d19360"),
            )
        })
    });

    c.bench_function("dag_find_all_paths", |b| {
        b.iter(|| {
            graph_dag.find_all_paths(
                black_box("1781f676dedf5767f3243db0a9738b35"),
                black_box("eb85851afd251bd7c7eaf725d0d19360"),
            )
        })
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(100);
    targets = criterion_benchmark
}

criterion_main!(benches);
