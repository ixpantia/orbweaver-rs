use crate::prelude::*;
use std::ops::Deref;
mod topological_sort;
use serde::{Deserialize, Serialize};
use topological_sort::topological_sort;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AcyclicDirectedGraph<Data> {
    pub(crate) dg: DirectedGraph<Data>,
    pub(crate) topological_sort: Vec<NodeId>,
}

impl<Data> Clone for AcyclicDirectedGraph<Data>
where
    Data: Clone,
{
    fn clone(&self) -> Self {
        AcyclicDirectedGraph {
            dg: self.dg.clone(),
            topological_sort: self.topological_sort.clone(),
        }
    }
}

impl<Data> AcyclicDirectedGraph<Data> {
    pub fn build(dg: DirectedGraph<Data>) -> Result<AcyclicDirectedGraph<Data>, GraphHasCycle> {
        let topological_sort = topological_sort::<Data>(&dg)?;
        Ok(AcyclicDirectedGraph {
            dg,
            topological_sort,
        })
    }
    pub fn into_inner(self) -> DirectedGraph<Data> {
        self.dg
    }
    /// Finds path using topological sort
    pub fn find_path(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Option<Vec<NodeId>>> {
        let start_id = self.get_node(&from)?.node_id;
        let goal_id = self.get_node(&to)?.node_id;
        if start_id == goal_id {
            return Ok(Some(vec![start_id]));
        }

        let topo_order = self.topological_sort.as_slice();
        let start_index = topo_order
            .iter()
            .position(|id| id == &start_id)
            .expect("Node must be included in topo_order");
        let goal_index = topo_order
            .iter()
            .position(|id| id == &goal_id)
            .expect("Node must be included in topo_order");

        if goal_index > start_index {
            return Ok(None); // No path from start to goal in a DAG if start comes after goal in topo order
        }

        let mut path = Vec::new();
        let mut current = goal_id.clone();
        path.push(current.clone());

        // Explore the path using the topological order
        for node_id in &topo_order[goal_index..=start_index] {
            if self.edge_exists(node_id, &current) {
                path.push(node_id.clone());
                current = node_id.clone();
                if current == start_id {
                    path.reverse();
                    return Ok(Some(path));
                }
            }
        }

        Ok(None)
    }
    pub fn find_all_paths(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Vec<Vec<NodeId>>> {
        // Helper function to perform DFS
        fn dfs<Data>(
            graph: &AcyclicDirectedGraph<Data>,
            current: NodeId,
            goal_id: NodeId,
            current_path: &mut Vec<NodeId>,
            all_paths: &mut Vec<Vec<NodeId>>,
        ) {
            // Add current node to path
            current_path.push(current.clone());

            // Check if the current node is the goal
            if current == goal_id {
                all_paths.push(current_path.clone());
            } else {
                // Continue to next nodes that can be visited from the current node
                for child in graph.children(&current).expect("Node must exist") {
                    dfs(
                        graph,
                        child.clone(),
                        goal_id.clone(),
                        current_path,
                        all_paths,
                    );
                }
            }

            // Backtrack to explore another path
            current_path.pop();
        }

        let start_id = self.get_node_id(&from)?;
        let goal_id = self.get_node_id(&to)?;

        let mut all_paths = Vec::new();
        let mut current_path = Vec::new();

        // Start DFS from the start node
        dfs(self, start_id, goal_id, &mut current_path, &mut all_paths);

        Ok(all_paths)
    }
}

impl<Data> Deref for AcyclicDirectedGraph<Data> {
    type Target = DirectedGraph<Data>;
    fn deref(&self) -> &Self::Target {
        &self.dg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topologically_sort() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_node("5", ());
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("4", "5");

        assert!(topological_sort::<()>(&graph).is_ok());
    }

    #[test]
    fn test_topologically_sort_non_acyclic() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_node("5", ());
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("4", "5");
        let _ = graph.add_edge("5", "1");

        assert!(topological_sort::<()>(&graph).is_err());
    }

    #[test]
    fn test_find_path_simple() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");

        let graph = AcyclicDirectedGraph::build(graph).unwrap();

        let path = graph.find_path("0", "4").unwrap();

        assert_eq!(path.unwrap().len(), 5);
    }

    #[test]
    fn test_find_path_many_paths() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("0", "4");

        let graph = AcyclicDirectedGraph::build(graph).unwrap();

        let path = graph.find_path("0", "4").unwrap();

        assert_eq!(path.unwrap().len(), 5);
    }

    #[test]
    fn test_find_all_paths_many_paths() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("0", "4");

        let graph = AcyclicDirectedGraph::build(graph).unwrap();

        let mut paths = graph.find_all_paths("0", "4").unwrap();
        println!("{paths:?}");

        paths.sort_unstable();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].len(), 5);
        assert_eq!(paths[1].len(), 2);
    }
}
