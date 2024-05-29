use crate::prelude::*;
use std::ops::Deref;
mod topological_sort;
use serde::{Deserialize, Serialize};
use topological_sort::topological_sort;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DirectedAcyclicGraph<Data> {
    pub(crate) dg: Box<DirectedGraph<Data>>,
    pub(crate) topological_sort: Vec<NodeId>,
}

impl<Data> Clone for DirectedAcyclicGraph<Data>
where
    Data: Clone,
{
    fn clone(&self) -> Self {
        DirectedAcyclicGraph {
            dg: self.dg.clone(),
            topological_sort: self.topological_sort.clone(),
        }
    }
}

impl<Data> DirectedAcyclicGraph<&Data>
where
    Data: Clone,
{
    pub fn cloned(self) -> DirectedAcyclicGraph<Data> {
        let dg = Box::new(self.dg.cloned());
        let topological_sort = self.topological_sort;
        DirectedAcyclicGraph {
            dg,
            topological_sort,
        }
    }
}

impl<Data> DirectedAcyclicGraph<Data> {
    pub fn build(dg: DirectedGraph<Data>) -> Result<DirectedAcyclicGraph<Data>, GraphHasCycle> {
        let topological_sort = topological_sort::<Data>(&dg)?;
        Ok(DirectedAcyclicGraph {
            dg: Box::new(dg),
            topological_sort,
        })
    }
    pub fn into_inner(self) -> DirectedGraph<Data> {
        *self.dg
    }
    /// Finds path using topological sort
    pub fn find_path(
        &self,
        start_id: NodeId,
        goal_id: NodeId,
    ) -> GraphInteractionResult<Option<Vec<NodeId>>> {
        if start_id == goal_id {
            return Ok(Some(vec![start_id]));
        }

        let topo_order = self.topological_sort.as_slice();
        let start_id_index = topo_order
            .iter()
            .position(|&id| id == start_id)
            .expect("Node must be included in topo_order");
        let goal_id_index = topo_order
            .iter()
            .position(|&id| id == goal_id)
            .expect("Node must be included in topo_order");

        if goal_id_index > start_id_index {
            return Ok(None); // No path from start_id to goal_id in a DAG if start_id comes after goal_id in topo order
        }

        let mut path = Vec::new();
        let mut current = goal_id;
        path.push(current);

        // Explore the path using the topological order
        for &node_id in &topo_order[goal_id_index..=start_id_index] {
            if self.edge_exists(node_id, current) {
                path.push(node_id);
                current = node_id;
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
        start_id: NodeId,
        goal_id: NodeId,
    ) -> GraphInteractionResult<Vec<Vec<NodeId>>> {
        // Helper function to perform DFS
        fn dfs<Data>(
            graph: &DirectedAcyclicGraph<Data>,
            current: NodeId,
            goal_id: NodeId,
            current_path: &mut Vec<NodeId>,
            all_paths: &mut Vec<Vec<NodeId>>,
        ) {
            // Add current node to path
            current_path.push(current);

            // Check if the current node is the goal
            if current == goal_id {
                all_paths.push(current_path.clone());
            } else {
                // Continue to next nodes that can be visited from the current node
                for child in graph.children(current).expect("Node must exist") {
                    dfs(
                        graph,
                        child,
                        goal_id,
                        current_path,
                        all_paths,
                    );
                }
            }

            // Backtrack to explore another path
            current_path.pop();
        }

        let mut all_paths = Vec::new();
        let mut current_path = Vec::new();

        // Start DFS from the start node
        dfs(self, start_id, goal_id, &mut current_path, &mut all_paths);

        Ok(all_paths)
    }

    pub fn subset(&self, node_id: NodeId) -> GraphInteractionResult<DirectedAcyclicGraph<&Data>> {
        let subset_dg = self.dg.subset(node_id)?;
        Ok(DirectedAcyclicGraph::build(subset_dg).expect("A subset of a DAG has no cycles"))
    }

    pub fn update_node_data(
        &mut self,
        node_id: NodeId,
        data: Data,
    ) -> GraphInteractionResult<Data> {
        self.dg.update_node_data(node_id, data)
    }
}

impl<Data> Deref for DirectedAcyclicGraph<Data> {
    type Target = DirectedGraph<Data>;
    fn deref(&self) -> &Self::Target {
        &self.dg
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_topologically_sort() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let id_5 = graph.add_node("5", ())?;
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_2, id_3);
        let _ = graph.add_edge(id_3, id_4);
        let _ = graph.add_edge(id_4, id_5);

        assert!(topological_sort(&graph).is_ok());
        Ok(())
    }

    #[test]
    fn test_topologically_sort_non_acyclic() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let id_5 = graph.add_node("5", ())?;
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_2, id_3);
        let _ = graph.add_edge(id_3, id_4);
        let _ = graph.add_edge(id_4, id_5);
        let _ = graph.add_edge(id_5, id_1);

        assert!(topological_sort(&graph).is_err());
        Ok(())
    }

    #[test]
    fn test_find_path_simple() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_0 = graph.add_node("0", ())?;
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let _ = graph.add_edge(id_0, id_1);
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_2, id_3);
        let _ = graph.add_edge(id_3, id_4);

        let graph = DirectedAcyclicGraph::build(graph)?;

        let path = graph.find_path(id_0, id_4)?.unwrap();

        assert_eq!(path.len(), 5);
        Ok(())
    }

    #[test]
    fn test_find_path_many_paths() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_0 = graph.add_node("0", ())?;
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let _ = graph.add_edge(id_0, id_1);
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_2, id_3);
        let _ = graph.add_edge(id_3, id_4);
        let _ = graph.add_edge(id_0, id_4);

        let graph = DirectedAcyclicGraph::build(graph)?;

        let path = graph.find_path(id_0, id_4)?.unwrap();

        assert_eq!(path.len(), 5);
        Ok(())
    }

    #[test]
    fn test_find_all_paths_many_paths() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_0 = graph.add_node("0", ())?;
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let _ = graph.add_edge(id_0, id_1);
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_2, id_3);
        let _ = graph.add_edge(id_3, id_4);
        let _ = graph.add_edge(id_0, id_4);

        let graph = DirectedAcyclicGraph::build(graph)?;

        let mut paths = graph.find_all_paths(id_0, id_4)?;
        println!("{paths:?}");

        paths.sort_unstable();

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].len(), 5);
        assert_eq!(paths[1].len(), 2);
        Ok(())
    }

    #[test]
    fn test_subset_tree_acyclic() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_0 = graph.add_node("0", ())?;
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let id_5 = graph.add_node("5", ())?;
        let _ = graph.add_edge(id_0, id_1);
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_2, id_3);
        let _ = graph.add_edge(id_3, id_4);
        let _ = graph.add_edge(id_0, id_4);
        let _ = graph.add_edge(id_3, id_5);

        let graph = DirectedAcyclicGraph::build(graph)?;

        let subset_graph = graph.subset(id_1)?;

        assert_eq!(subset_graph.get_leaves(), vec![id_4, id_5]);
        Ok(())
    }
}
