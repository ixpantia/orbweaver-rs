use orbweaver::prelude::*;

const MEDIUM_TXT: &str = include_str!("medium.txt");

fn get_medium_graph() -> DirectedAcyclicGraph {
    let mut builder = DirectedGraphBuilder::new();
    MEDIUM_TXT
        .lines()
        .filter_map(|l| l.split_once('\t'))
        .for_each(|(parent, child)| {
            builder.add_edge(parent, child);
        });

    builder.build_acyclic().expect("This should work")
}

#[test]
fn get_all_leaves_equivalent_to_leaves_under_root() {
    let dag = get_medium_graph();
    let all_leaves = dag.get_all_leaves();
    let leaves_under_root = dag
        .get_leaves_under(["1781f676dedf5767f3243db0a9738b35"])
        .unwrap();
    assert_eq!(all_leaves.len(), leaves_under_root.len());
}

#[test]
fn get_all_leaves_equivalent_to_leaves_under_root_after_subset() {
    let dag = get_medium_graph();
    let dag_s = dag.subset("1781f676dedf5767f3243db0a9738b35").unwrap();
    let leaves_under_root = dag_s
        .get_leaves_under(["1781f676dedf5767f3243db0a9738b35"])
        .unwrap();
    assert_eq!(dag_s.get_all_leaves().len(), leaves_under_root.len());
    assert_eq!(dag.get_all_leaves().len(), leaves_under_root.len());
}
