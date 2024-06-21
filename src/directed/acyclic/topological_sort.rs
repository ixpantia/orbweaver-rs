use crate::{
    prelude::*,
    utils::{node_map::LazySet, sym::Sym},
};

pub fn topological_sort(dg: &DirectedGraph) -> Result<Vec<Sym>, GraphHasCycle> {
    let mut dg = dg.clone();
    let mut res = Vec::new();
    let mut no_deps = dg.leaves.clone();
    let mut parents = Vec::new();

    while let Some(node) = no_deps.pop() {
        res.push(node);

        dg.parents_u32(&[node], &mut parents);
        for parent in parents.drain(..) {
            // We need to manually remove this edge
            if let LazySet::Initialized(children) = dg.children_map.get_mut(parent) {
                children.remove(&node);
            }
            if let LazySet::Initialized(node_parents) = dg.parent_map.get_mut(node) {
                if node_parents.remove(&parent) {
                    dg.n_edges -= 1;
                }
            }

            // Check if the parent still has children.
            // If it does not we add it to the `no_deps`
            // vector.
            match dg.children_map.get(parent) {
                // If it matches and it has items them we do nothing
                // under any other circumstance we add it to
                // no deps
                LazySet::Initialized(children) if !children.is_empty() => {}
                _ => no_deps.push(parent),
            }
        }
    }

    if dg.n_edges != 0 {
        return Err(GraphHasCycle);
    }

    Ok(res)
}

#[cfg(test)]
mod tests {

    use crate::directed::DirectedGraphBuilder;

    use super::*;

    #[test]
    fn test_topologically_sort() {
        let mut builder = DirectedGraphBuilder::new();

        let _ = builder.add_edge("1", "2");
        let _ = builder.add_edge("2", "3");
        let _ = builder.add_edge("3", "4");
        let _ = builder.add_edge("4", "5");

        let graph = builder.build_directed();

        assert!(topological_sort(&graph).is_ok());
    }

    #[test]
    fn test_topologically_sort_paths() {
        let mut builder = DirectedGraphBuilder::new();

        builder.add_path(["Hello", "World"]);
        builder.add_path(["Hello", "Everybody", "Andrés"]);
        builder.add_path(["Hello", "Everybody", "Andrea"]);

        let graph = builder.build_directed();

        assert!(topological_sort(&graph).is_ok());
    }

    #[test]
    fn test_topologically_sort_non_acyclic() {
        let mut builder = DirectedGraphBuilder::new();
        let _ = builder.add_edge("1", "2");
        let _ = builder.add_edge("2", "3");
        let _ = builder.add_edge("3", "4");
        let _ = builder.add_edge("4", "5");
        let _ = builder.add_edge("5", "1");
        let graph = builder.build_directed();

        assert!(topological_sort(&graph).is_err());
    }
}
