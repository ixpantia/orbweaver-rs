use super::*;

#[derive(Debug)]
pub struct AcyclicDirectedGraph<Data> {
    dg: DirectedGraph<Data>,
    topological_sort: Vec<NodeId>,
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

/// L ← Empty list that will contain the sorted elements
/// S ← Set of all nodes with no incoming edge
///
/// while S is not empty do
///     remove a node n from S
///     add n to L
///     for each node m with an edge e from n to m do
///         remove edge e from the graph
///         if m has no other incoming edges then
///             insert m into S
///
/// if graph has edges then
///     return error   (graph has at least one cycle)
/// else
///     return L   (a topologically sorted order)
fn topological_sort<Data>(dg: &DirectedGraph<Data>) -> Result<Vec<NodeId>, error::GraphHasCycle> {
    let mut dg_temp = dg.into_dataless();
    let mut l = Vec::new();
    let mut s: Vec<NodeId> = dg_temp
        .get_nodes_no_incoming_edges()
        .into_iter()
        .map(|node| node.id)
        .collect();
    while let Some(n) = s.pop() {
        l.push(n.clone());
        let nodes_m = dg_temp
            .nodes()
            .map(|m| m.id)
            .filter_map(|m| Some((m.clone(), dg_temp.get_edge(&n, &m)?)))
            .collect::<Vec<_>>();
        for (m, e) in nodes_m {
            dg_temp.remove_edge(e);
            if dg_temp.get_incoming_edges(&m).is_empty() {
                s.push(m);
            }
        }
    }
    if !dg_temp.edges.is_empty() {
        return Err(error::GraphHasCycle);
    }
    Ok(l)
}

impl<Data> AcyclicDirectedGraph<Data> {
    pub fn build(
        dg: DirectedGraph<Data>,
    ) -> Result<AcyclicDirectedGraph<Data>, error::GraphHasCycle> {
        let topological_sort = topological_sort(&dg)?;
        Ok(AcyclicDirectedGraph {
            dg,
            topological_sort,
        })
    }
    pub fn into_inner(self) -> DirectedGraph<Data> {
        self.dg
    }
    /// Finds path using topological sort
    pub fn find_path(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> Option<Vec<NodeId>> {
        let start_id = self.get_node(&from)?.id;
        let goal_id = self.get_node(&to)?.id;
        if start_id == goal_id {
            return Some(vec![start_id]);
        }

        let topo_order = self.topological_sort.as_slice();
        let start_index = topo_order.iter().position(|id| id == &start_id)?;
        let goal_index = topo_order.iter().position(|id| id == &goal_id)?;

        if start_index > goal_index {
            return None; // No path from start to goal in a DAG if start comes after goal in topo order
        }

        let mut path = Vec::new();
        let mut current = start_id.clone();
        path.push(current.clone());

        // Explore the path using the topological order
        for node_id in topo_order[start_index..=goal_index].iter() {
            if self.get_edge(&current, node_id).is_some() {
                path.push(node_id.clone());
                current = node_id.clone();
                if current == goal_id {
                    return Some(path);
                }
            }
        }

        None // No path found
    }
    pub fn find_all_paths(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> Vec<Vec<NodeId>> {
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
                for edge in graph.get_outgoing_edges(&current) {
                    dfs(
                        graph,
                        edge.to.clone(),
                        goal_id.clone(),
                        current_path,
                        all_paths,
                    );
                }
            }

            // Backtrack to explore another path
            current_path.pop();
        }

        let start_id = match self.get_node(&from) {
            Some(node) => node.id,
            None => return Vec::new(), // Node not found
        };
        let goal_id = match self.get_node(&to) {
            Some(node) => node.id,
            None => return Vec::new(), // Node not found
        };

        let mut all_paths = Vec::new();
        let mut current_path = Vec::new();

        // Start DFS from the start node
        dfs(self, start_id, goal_id, &mut current_path, &mut all_paths);

        all_paths
    }

    pub fn filter(&self, predicate: impl FnMut(&Node<&Data>) -> bool) -> Self
    where
        Data: Clone,
    {
        let dg = self.dg.filter(predicate);
        let filtered_node_ids: Vec<_> = dg.nodes().map(|n| n.id).collect();
        let topological_sort: Vec<_> = self
            .topological_sort
            .iter()
            .filter(|node| filtered_node_ids.contains(node))
            .cloned()
            .collect();

        Self {
            topological_sort,
            dg,
        }
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

        assert!(topological_sort(&graph).is_ok());
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

        assert!(topological_sort(&graph).is_err());
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

        assert_eq!(path.len(), 5);
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

        assert_eq!(path.len(), 5);
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

        let paths = graph.find_all_paths("0", "4");

        assert_eq!(paths.len(), 2);
        assert_eq!(paths[0].len(), 5);
        assert_eq!(paths[1].len(), 2);
    }
}
