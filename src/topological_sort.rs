use crate::{error, DirectedGraph, NodeId};

pub fn topological_sort<Data>(
    dg: &DirectedGraph<Data>,
) -> Result<Vec<NodeId>, error::GraphHasCycle> {
    let mut dg = dg.into_dataless();
    let mut res = Vec::new();
    let mut no_deps = dg.no_deps().map(|node| node.node_id).collect::<Vec<_>>();

    while let Some(node) = no_deps.pop() {
        res.push(node.clone());

        if let Some(parents) = dg.parents(&node).cloned() {
            for parent in parents {
                dg.remove_edge(&parent, &node);
                if !dg.has_children(&parent) {
                    no_deps.push(parent.clone());
                }
            }
        }
    }

    if !dg.parents.is_empty() {
        return Err(error::GraphHasCycle);
    }

    Ok(res)
}
