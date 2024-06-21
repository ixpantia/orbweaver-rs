use crate::{directed::DirectedGraph, prelude::*, utils::sym::Sym};
use std::ops::Deref;
mod topological_sort;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use topological_sort::topological_sort;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DirectedAcyclicGraph {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub(crate) dg: Box<DirectedGraph>,
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
        }
    }
}

impl DirectedAcyclicGraph {
    pub(crate) fn build(dg: DirectedGraph) -> Result<DirectedAcyclicGraph, GraphHasCycle> {
        topological_sort(&dg)?;
        Ok(DirectedAcyclicGraph { dg: Box::new(dg) })
    }

    pub fn into_inner(self) -> DirectedGraph {
        *self.dg
    }

    /// Finds all paths on a DAG using DFS
    pub fn find_all_paths(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Vec<Vec<&str>>> {
        // Helper function to perform DFS
        #[inline]
        fn dfs(
            graph: &DirectedAcyclicGraph,
            current: Sym,
            goal_id: Sym,
            current_path: &mut Vec<Sym>,
            all_paths: &mut Vec<Sym>,
            children_buffer: &mut Vec<Sym>,
        ) {
            // Add current node to path
            current_path.push(current);

            // Check if the current node is the goal
            if current == goal_id {
                all_paths.extend_from_slice(current_path);
                all_paths.push(Sym::RESERVED);
            } else {
                let children_start_index_local = children_buffer.len();
                graph.children_u32(&[current], children_buffer);
                (children_start_index_local..children_buffer.len()).for_each(|_| {
                    match children_buffer.pop() {
                        Some(child) => dfs(
                            graph,
                            child,
                            goal_id,
                            current_path,
                            all_paths,
                            children_buffer,
                        ),
                        None => unsafe { std::hint::unreachable_unchecked() },
                    };
                });
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
            .split(|&n| n.is_reserved())
            .filter(|p| !p.is_empty())
            .map(|path| self.resolve_mul_slice(path))
            .collect())
    }

    pub fn subset(&self, node: impl AsRef<str>) -> GraphInteractionResult<DirectedAcyclicGraph> {
        let dg = Box::new(self.dg.subset(node)?);
        Ok(DirectedAcyclicGraph { dg })
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

        assert_eq!(path.len(), 2);
        assert_eq!(path, ["0", "4"]);
    }

    #[test]
    fn test_find_path_no_paths() {
        let mut builder = DirectedGraphBuilder::new();
        let _ = builder.add_edge("0", "1");
        let _ = builder.add_edge("1", "2");
        let _ = builder.add_edge("2", "3");
        let _ = builder.add_edge("3", "4");
        let _ = builder.add_edge("0", "4");
        let _ = builder.add_edge("999", "111");

        let graph = builder.build_acyclic().unwrap();

        let path = graph.find_path("0", "999").unwrap();

        assert_eq!(path.len(), 0);
        assert_eq!(path, Vec::<&str>::new());
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
                vec!["0", "999", "4"],
                vec!["0", "111", "222", "333", "444", "4"],
                vec!["0", "1", "2", "3", "4"],
                vec!["0", "4"],
            ]
        );
    }
}
