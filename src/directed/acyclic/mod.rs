use crate::{directed::DirectedGraph, prelude::*};
use std::ops::Deref;
mod topological_sort;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use topological_sort::topological_sort;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DirectedAcyclicGraph {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub(crate) dg: Box<DirectedGraph>,
    pub(crate) topological_sort: Vec<u32>,
}

impl std::fmt::Debug for DirectedAcyclicGraph {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.dg.fmt(f)
    }
}

impl Clone for DirectedAcyclicGraph {
    fn clone(&self) -> Self {
        DirectedAcyclicGraph {
            dg: self.dg.clone(),
            topological_sort: self.topological_sort.clone(),
        }
    }
}

impl DirectedAcyclicGraph {
    pub(crate) fn build(dg: DirectedGraph) -> Result<DirectedAcyclicGraph, GraphHasCycle> {
        let topological_sort = topological_sort(&dg)?;
        Ok(DirectedAcyclicGraph {
            dg: Box::new(dg),
            topological_sort,
        })
    }

    pub fn into_inner(self) -> DirectedGraph {
        *self.dg
    }

    /// Finds path using topological sort
    pub fn find_path(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Vec<&str>> {
        let from = self.get_internal(from)?;
        let to = self.get_internal(to)?;

        if from == to {
            return Ok(vec![self.resolve(from)]);
        }

        let topo_order = self.topological_sort.as_slice();
        let start_index = topo_order
            .iter()
            .position(|id| id == &from)
            .expect("Node must be included in topo_order");
        let goal_index = topo_order
            .iter()
            .position(|id| id == &to)
            .expect("Node must be included in topo_order");

        if goal_index > start_index {
            return Ok(vec![]); // No path from start to goal in a DAG if start comes after goal in topo order
        }

        let path = unsafe { self.dg.u32x1_vec_0() };
        let mut current = to;
        path.push(current);

        // Explore the path using the topological order
        for &node_id in &topo_order[goal_index..=start_index] {
            if self.edge_exists(node_id, current) {
                path.push(node_id);
                current = node_id;
                if current == from {
                    path.reverse();
                    return Ok(self.resolve_mul(path.drain(..)));
                }
            }
        }

        Ok(vec![])
    }

    pub fn find_all_paths(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Vec<Vec<&str>>> {
        // Helper function to perform DFS
        #[inline]
        fn dfs(
            graph: &DirectedAcyclicGraph,
            current: u32,
            goal_id: u32,
            current_path: &mut Vec<u32>,
            all_paths: &mut Vec<u32>,
            children_buffer: &mut Vec<u32>,
        ) {
            // Add current node to path
            current_path.push(current);

            // Check if the current node is the goal
            if current == goal_id {
                all_paths.extend_from_slice(current_path);
                all_paths.push(0);
            } else {
                let children_start_index_local = children_buffer.len();
                // Continue to next nodes that can be visited from the current node
                graph.children_u32(&[current], children_buffer);
                while let Some(child) = children_buffer.pop() {
                    dfs(
                        graph,
                        child,
                        goal_id,
                        current_path,
                        all_paths,
                        children_buffer,
                    );
                    // The use of this buffer is to stop additional
                    // uneeded allocations
                    if children_buffer.len() == children_start_index_local {
                        break;
                    }
                }
            }

            // Backtrack to explore another path
            current_path.pop();
        }

        let from = self.get_internal(from)?;
        let to = self.get_internal(to)?;

        let current_path = unsafe { self.dg.u32x1_vec_0() };
        let children = unsafe { self.dg.u32x1_vec_1() };
        let all_paths = unsafe { self.dg.u32x1_vec_2() };

        // Start DFS from the start node
        dfs(self, from, to, current_path, all_paths, children);

        Ok(all_paths
            .split(|&n| n == 0)
            .filter(|path| !path.is_empty())
            .map(|path| self.resolve_mul(path.iter().copied()))
            .collect())
    }

    pub fn subset(&self, node: impl AsRef<str>) -> GraphInteractionResult<DirectedAcyclicGraph> {
        let subset_dg = self.dg.subset(node)?;
        Ok(DirectedAcyclicGraph::build(subset_dg).expect("A subset of a DAG has no cycles"))
    }
}

impl Deref for DirectedAcyclicGraph {
    type Target = DirectedGraph;
    fn deref(&self) -> &Self::Target {
        &self.dg
    }
}

#[cfg(test)]
mod tests {
    use crate::directed::DirectedGraphBuilder;

    #[test]
    fn test_find_path_simple() {
        let mut builder = DirectedGraphBuilder::new();
        let _ = builder.add_edge("0", "1");
        let _ = builder.add_edge("1", "2");
        let _ = builder.add_edge("2", "3");
        let _ = builder.add_edge("3", "4");
        let graph = builder.build_acyclic().unwrap();

        let path = graph.find_path("0", "4").unwrap();

        assert_eq!(path.len(), 5);
        assert_eq!(path, ["0", "1", "2", "3", "4"]);
    }

    #[test]
    fn test_find_path_many_paths() {
        let mut builder = DirectedGraphBuilder::new();
        let _ = builder.add_edge("0", "1");
        let _ = builder.add_edge("1", "2");
        let _ = builder.add_edge("2", "3");
        let _ = builder.add_edge("3", "4");
        let _ = builder.add_edge("0", "4");

        let graph = builder.build_acyclic().unwrap();

        let path = graph.find_path("0", "4").unwrap();

        assert_eq!(path.len(), 5);
        assert_eq!(path, ["0", "1", "2", "3", "4"]);
    }

    #[test]
    fn test_find_all_paths_many_paths() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_path(["0", "111", "222", "333", "444", "4"]);
        builder.add_path(["0", "999", "4"]);
        builder.add_path(["0", "1", "2", "3", "4"]);
        builder.add_path(["0", "4"]);
        let graph = builder.build_acyclic().unwrap();

        let paths = graph.find_all_paths("0", "4").unwrap();

        assert_eq!(
            paths,
            vec![
                vec!["0", "4"],
                vec!["0", "999", "4"],
                vec!["0", "111", "222", "333", "444", "4"],
                vec!["0", "1", "2", "3", "4"],
            ]
        );
    }

    #[test]
    fn test_debug() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("1", "2");
        builder.add_edge("2", "3");
        builder.add_edge("3", "4");
        builder.add_edge("4", "5");
        builder.add_edge("5", "6");
        builder.add_edge("6", "7");
        builder.add_edge("7", "8");
        builder.add_edge("8", "9");
        builder.add_edge("9", "10");
        builder.add_edge("10", "11");
        builder.add_edge("11", "12");
        builder.add_edge("12", "13");
        let dg = builder.build_acyclic().unwrap();

        let actual = format!("{:?}", dg);

        assert_eq!(
            actual,
            "# of nodes: 12\n# of edges: 12\n# of roots: 1\n# of leaves: 1\n\n|   Parent   |    Child   |\n| ---------- | ---------- |\n| 0000000010 | 0000000011 |\n| 0000000007 | 0000000008 |\n| 0000000004 | 0000000005 |\n| 0000000001 | 0000000002 |\n| 0000000011 | 0000000012 |\n| 0000000008 | 0000000009 |\n| 0000000005 | 0000000006 |\n| 0000000002 | 0000000003 |\n| 0000000012 | 0000000013 |\n| 0000000009 | 0000000010 |\nOmitted 2 nodes\n"
        )
    }
}
