use crate::prelude::*;

pub fn topological_sort<Data>(dg: &DirectedGraph<Data>) -> Result<Vec<NodeId>, GraphHasCycle> {
    let mut dg = dg.into_dataless();
    let mut res = Vec::new();
    let mut no_deps = dg.get_leaves();

    while let Some(node) = no_deps.pop() {
        res.push(node.clone());

        if let Ok(parents) = dg.parents(&node).cloned() {
            for parent in parents {
                dg.remove_edge(&parent, &node);
                if !dg.has_children(&parent).unwrap() {
                    no_deps.push(parent.clone());
                }
            }
        }
    }

    if dg.n_edges != 0 {
        return Err(GraphHasCycle);
    }

    Ok(res)
}
