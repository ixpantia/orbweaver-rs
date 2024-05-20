use std::{ops::Deref, rc::Rc};

pub mod acyclic;
pub mod error;

pub use acyclic::AcyclicDirectedGraph;

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug)]
pub struct NodeId(Rc<str>);

impl AsRef<str> for NodeId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl From<NodeId> for String {
    fn from(value: NodeId) -> Self {
        value.as_ref().to_string()
    }
}

impl std::cmp::PartialEq<str> for NodeId {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref().eq(other)
    }
}

impl std::cmp::PartialOrd<str> for NodeId {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        self.0.as_ref().partial_cmp(other)
    }
}

impl From<&str> for NodeId {
    fn from(value: &str) -> Self {
        NodeId(Rc::from(value))
    }
}

impl Clone for NodeId {
    fn clone(&self) -> Self {
        NodeId(Rc::clone(&self.0))
    }
}

impl Deref for NodeId {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref()
    }
}

#[derive(Debug)]
pub struct Node<Data> {
    id: NodeId,
    data: Data,
}

impl<Data> Node<Data> {
    pub fn id(&self) -> &str {
        self.id.as_ref()
    }
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl<Data: Clone> Clone for Node<Data> {
    fn clone(&self) -> Self {
        Node {
            id: NodeId::clone(&self.id),
            data: Data::clone(&self.data),
        }
    }
}

impl<'a, Data> Node<&'a Data>
where
    Data: Clone,
{
    fn cloned(self) -> Node<Data> {
        Node {
            id: self.id,
            data: Data::clone(self.data),
        }
    }
}

impl<Data> Node<Data> {
    pub fn as_ref(&self) -> Node<&Data> {
        Node {
            id: NodeId::clone(&self.id),
            data: &self.data,
        }
    }
    pub fn new(id: impl AsRef<str>, data: Data) -> Self {
        Node {
            id: id.as_ref().into(),
            data,
        }
    }
}

impl<Data> std::cmp::Eq for Node<Data> {}

impl<Data> PartialEq for Node<Data> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl<Data> PartialOrd for Node<Data> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<Data> Ord for Node<Data> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    from: NodeId,
    to: NodeId,
}

impl Edge {
    fn new(from: impl AsRef<str>, to: impl AsRef<str>) -> Edge {
        Edge {
            from: from.as_ref().into(),
            to: to.as_ref().into(),
        }
    }
}

#[derive(Debug)]
pub struct DirectedGraph<Data> {
    nodes: Vec<Node<Data>>,
    edges: Vec<Edge>,
}

impl<Data: Clone> Clone for DirectedGraph<Data> {
    fn clone(&self) -> Self {
        DirectedGraph {
            nodes: Vec::clone(&self.nodes),
            edges: Vec::clone(&self.edges),
        }
    }
}

impl<Data> DirectedGraph<Data> {
    pub fn new() -> Self {
        DirectedGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }
    pub fn add_node(
        &mut self,
        id: impl AsRef<str>,
        data: Data,
    ) -> Result<&mut Self, error::DuplicateNode> {
        let node_id: NodeId = id.as_ref().into();
        let new_node = Node {
            id: node_id.clone(),
            data,
        };
        match self.nodes.binary_search(&new_node) {
            Ok(_) => Err(error::DuplicateNode(node_id)),
            Err(index) => {
                self.nodes.insert(index, new_node);
                Ok(self)
            }
        }
    }
    pub fn get_node(&self, id: impl AsRef<str>) -> Option<Node<&Data>> {
        let index = self
            .nodes
            .binary_search_by(|node| {
                node.id
                    .partial_cmp(id.as_ref())
                    .expect("&str and NodeId are should always be comparable")
            })
            .ok()?;
        // This is 100% safe since we got the index in the previous
        // step. Not a huge performance win, but why not?
        unsafe { Some(self.nodes.get_unchecked(index).as_ref()) }
    }
    pub fn add_edge(
        &mut self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> Result<&mut Self, error::AddEdgeError> {
        let _from_node = self
            .get_node(&from)
            .ok_or_else(|| error::AddEdgeError::node_not_exists(&from))?;
        let _to_node = self
            .get_node(&to)
            .ok_or_else(|| error::AddEdgeError::node_not_exists(&to))?;
        let edge = Edge::new(from, to);
        match self.edges.binary_search(&edge) {
            Err(index) => {
                self.edges.insert(index, edge);
                Ok(self)
            }
            Ok(_) => Err(error::AddEdgeError::DuplicateEdge(edge)),
        }
    }

    pub fn add_path(&mut self, path: &[impl AsRef<str>]) -> Result<&mut Self, error::AddEdgeError> {
        for edge in path.windows(2) {
            let from = unsafe { edge.get_unchecked(0) };
            let to = unsafe { edge.get_unchecked(1) };
            match self.add_edge(from, to) {
                Ok(_) => continue,
                Err(e) => match e {
                    error::AddEdgeError::NodeNotExist(_) => return Err(e),
                    error::AddEdgeError::DuplicateEdge(_) => continue,
                },
            }
        }
        Ok(self)
    }

    pub fn get_edge(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> Option<Edge> {
        self.edges()
            .find(|edge| edge.from.eq(from.as_ref()) && edge.to.eq(to.as_ref()))
    }

    pub fn get_incoming_edges(&self, node: impl AsRef<str>) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(move |edge| edge.to.eq(node.as_ref()))
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn filter(&self, predicate: impl FnMut(&Node<&Data>) -> bool) -> Self
    where
        Data: Clone,
    {
        let nodes: Vec<_> = self.nodes().filter(predicate).map(Node::cloned).collect();
        let node_ids: Vec<_> = nodes.iter().map(|node| node.id.clone()).collect();
        let edges = self
            .edges()
            .filter(|Edge { to, from }| node_ids.contains(to) && node_ids.contains(from))
            .collect();
        Self { nodes, edges }
    }

    pub fn get_outgoing_edges(&self, node: impl AsRef<str>) -> Vec<Edge> {
        self.edges
            .iter()
            .filter(move |edge| edge.from.eq(node.as_ref()))
            .cloned()
            .collect::<Vec<_>>()
    }

    pub fn children(&self, node: impl AsRef<str>) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(move |edge| edge.from.eq(node.as_ref()))
            .map(|edge| edge.to.clone())
            .collect::<Vec<_>>()
    }

    pub fn parents(&self, node: impl AsRef<str>) -> Vec<NodeId> {
        self.edges
            .iter()
            .filter(move |edge| edge.to.eq(node.as_ref()))
            .map(|edge| edge.from.clone())
            .collect::<Vec<_>>()
    }

    pub fn get_nodes_no_incoming_edges(&self) -> Vec<Node<&Data>> {
        let incoming_edges: Vec<NodeId> = self.edges.iter().map(|edge| &edge.to).cloned().collect();
        self.nodes
            .iter()
            .filter(move |node| !incoming_edges.contains(&node.id))
            .map(Node::as_ref)
            .collect()
    }
    pub fn remove_edge(&mut self, edge: Edge) -> &mut Self {
        if let Ok(index) = self.edges.binary_search(&edge) {
            self.edges.remove(index);
        }
        self
    }
    pub fn has_incoming_edge(&self, id: impl AsRef<str>) -> bool {
        self.edges
            .binary_search_by(|edge| {
                edge.to
                    .partial_cmp(id.as_ref())
                    .expect("&str and NodeId are should always be comparable")
            })
            .is_ok()
    }
    pub fn nodes(&self) -> impl Iterator<Item = Node<&Data>> {
        self.nodes.iter().map(Node::as_ref)
    }
    pub fn edges(&self) -> impl Iterator<Item = Edge> + '_ {
        self.edges.iter().cloned()
    }
    pub fn into_dataless(&self) -> DirectedGraph<()> {
        let nodes = self
            .nodes
            .iter()
            .map(|node| Node::new(&node.id, ()))
            .collect();
        let edges = self.edges.clone();
        DirectedGraph { nodes, edges }
    }
    /// Finds path using breadth-first search
    pub fn find_path(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> Option<Vec<NodeId>> {
        // Helper function for constructing the path
        fn construct_path(
            parents: &[(NodeId, NodeId)],
            start_id: &NodeId,
            goal_id: &NodeId,
        ) -> Vec<NodeId> {
            let mut path = Vec::new();
            let mut current_id = goal_id;
            path.push(current_id.clone());

            while current_id != start_id {
                if let Some(parent_pair) = parents.iter().find(|(node, _)| node == current_id) {
                    current_id = &parent_pair.1;
                    path.push(current_id.clone());
                } else {
                    break; // This should not happen if the path exists
                }
            }

            path.reverse(); // Reverse to get the path from start to goal
            path
        }

        let start_id = self.get_node(&from)?.id;
        let goal_id = self.get_node(&to)?.id;

        if start_id == goal_id {
            return Some(vec![start_id]);
        }

        let mut queue = Vec::new();
        let mut visited = Vec::new();
        let mut parents = Vec::new(); // To track the path back to the start node

        // Initialize
        queue.push(start_id.clone());
        visited.push(start_id.clone());

        while let Some(current) = queue.pop() {
            for edge in self.edges.iter().filter(|e| e.from == current) {
                if !visited.contains(&edge.to) {
                    visited.push(edge.to.clone());
                    parents.push((edge.to.clone(), current.clone()));

                    if edge.to == goal_id {
                        // If goal found, construct the path from parents
                        return Some(construct_path(&parents, &start_id, &goal_id));
                    }
                    queue.push(edge.to.clone());
                }
            }
        }

        None // No path found
    }

    pub fn clear_edges(&mut self) -> &mut Self {
        self.edges.clear();
        self
    }
}

impl<Data> Default for DirectedGraph<Data> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_a_new_directed_graph() {
        let _ = DirectedGraph::<()>::new();
    }

    #[test]
    fn test_add_nodes_to_a_graph() -> Result<(), Box<dyn std::error::Error>> {
        let mut graph = DirectedGraph::<()>::new();
        graph
            .add_node("0", ())?
            .add_node("1", ())?
            .add_node("2", ())?;
        Ok(())
    }

    #[test]
    fn test_add_repeated_node_errors() {
        let mut graph = DirectedGraph::<()>::new();
        graph
            .add_node("0", ())
            .expect("First node can't be repeated");
        assert!(graph.add_node("0", ()).is_err());
    }

    #[test]
    fn test_get_node_non_existent() {
        let graph = DirectedGraph::<()>::new();
        assert!(graph.get_node("0").is_none());
    }

    #[test]
    fn test_get_node_existent() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("999", ());
        assert!(graph.get_node("999").is_some());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("999", ());
        assert!(graph.add_edge("0", "999").is_ok());
        assert_eq!(graph.edges[0], Edge::new("0", "999"));
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

        let path = graph.find_path("0", "4").unwrap();

        assert_eq!(path.len(), 2);
    }
}
