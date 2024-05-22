use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
    rc::Rc,
};

pub mod acyclic;
pub mod error;
pub(crate) mod topological_sort;
pub use acyclic::AcyclicDirectedGraph;

/// Prelude of data types and functionality.
pub mod prelude {
    pub use crate::acyclic::AcyclicDirectedGraph;
    pub use crate::error;
    pub use crate::DirectedGraph;
    pub use crate::Node;
    pub use crate::NodeId;
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
pub struct NodeId(Rc<str>);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

impl AsRef<str> for NodeId {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::borrow::Borrow<str> for NodeId {
    fn borrow(&self) -> &str {
        self.as_ref()
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

pub struct Node<Data> {
    node_id: NodeId,
    data: Data,
}

impl<Data> Node<Data> {
    #[inline(always)]
    fn new(node_id: NodeId, data: Data) -> Self {
        Node { node_id, data }
    }
    #[inline(always)]
    pub fn id(&self) -> NodeId {
        self.node_id.clone()
    }
    #[inline(always)]
    pub fn data(&self) -> &Data {
        &self.data
    }
}

#[derive(Debug)]
pub struct DirectedGraph<Data> {
    nodes: HashMap<NodeId, Data>,
    parents: HashMap<NodeId, HashSet<NodeId>>,
    children: HashMap<NodeId, HashSet<NodeId>>,
}

impl<Data: Clone> Clone for DirectedGraph<Data> {
    fn clone(&self) -> Self {
        DirectedGraph {
            nodes: HashMap::clone(&self.nodes),
            parents: HashMap::clone(&self.parents),
            children: HashMap::clone(&self.children),
        }
    }
}

impl<Data> DirectedGraph<Data> {
    pub fn new() -> Self {
        DirectedGraph {
            nodes: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
        }
    }
    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }
    pub fn add_node(
        &mut self,
        id: impl AsRef<str>,
        data: Data,
    ) -> Result<&mut Self, error::DuplicateNode> {
        let node_id: NodeId = id.as_ref().into();

        match self.nodes.insert(node_id.clone(), data) {
            Some(_) => Err(error::DuplicateNode(node_id)),
            _ => Ok(self),
        }
    }
    pub fn get_node(&self, id: impl AsRef<str>) -> Option<Node<&Data>> {
        if let Some((node_id, data)) = self.nodes.get_key_value(id.as_ref()) {
            return Some(Node::new(node_id.clone(), data));
        }
        None
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
        let from: NodeId = from.as_ref().into();
        let to: NodeId = to.as_ref().into();
        self.children
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.parents
            .entry(to.clone())
            .or_default()
            .insert(from.clone());
        Ok(self)
    }

    pub fn add_path(&mut self, path: &[impl AsRef<str>]) -> Result<&mut Self, error::AddEdgeError> {
        for edge in path.windows(2) {
            let from = unsafe { edge.get_unchecked(0) };
            let to = unsafe { edge.get_unchecked(1) };
            self.add_edge(from, to)?;
        }
        Ok(self)
    }

    pub fn edge_exists(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> bool {
        if let Some(children) = self.children.get(from.as_ref()) {
            return children.contains(to.as_ref());
        }
        false
    }

    pub fn children(&self, node: impl AsRef<str>) -> Option<&HashSet<NodeId>> {
        self.children.get(node.as_ref())
    }

    pub fn parents(&self, node: impl AsRef<str>) -> Option<&HashSet<NodeId>> {
        self.parents.get(node.as_ref())
    }

    pub fn remove_edge(&mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> &mut Self {
        if let Some(children) = self.children.get_mut(from.as_ref()) {
            children.remove(to.as_ref());
            if children.is_empty() {
                self.children.remove(from.as_ref());
            }
        }
        if let Some(parents) = self.parents.get_mut(to.as_ref()) {
            parents.remove(from.as_ref());
            if parents.is_empty() {
                self.parents.remove(to.as_ref());
            }
        }
        self
    }

    pub fn has_parents(&self, id: impl AsRef<str>) -> bool {
        if let Some(parents) = self.parents(id) {
            return !parents.is_empty();
        }
        false
    }

    pub fn has_children(&self, id: impl AsRef<str>) -> bool {
        if let Some(children) = self.children(id) {
            return !children.is_empty();
        }
        false
    }

    pub fn nodes(&self) -> impl Iterator<Item = Node<&Data>> {
        self.nodes
            .iter()
            .map(|(node_id, data)| Node::new(node_id.clone(), data))
    }

    pub fn into_dataless(&self) -> DirectedGraph<()> {
        let nodes = self
            .nodes
            .keys()
            .map(|node_id| (node_id.clone(), ()))
            .collect();
        DirectedGraph {
            nodes,
            parents: self.parents.clone(),
            children: self.children.clone(),
        }
    }
    /// Finds path using breadth-first search
    pub fn find_path(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> Option<Vec<NodeId>> {
        // Helper function for constructing the path
        fn construct_path(
            parents: &[(&NodeId, &NodeId)],
            start_id: &NodeId,
            goal_id: &NodeId,
        ) -> Vec<NodeId> {
            let mut path = Vec::new();
            let mut current_id = goal_id;
            path.push(current_id.clone());

            while current_id != start_id {
                if let Some(parent_pair) = parents.iter().find(|(node, _)| *node == current_id) {
                    current_id = &parent_pair.1;
                    path.push(current_id.clone());
                } else {
                    break; // This should not happen if the path exists
                }
            }

            path.reverse(); // Reverse to get the path from start to goal
            path
        }

        let start_id = self.get_node(&from)?.node_id;
        let goal_id = self.get_node(&to)?.node_id;

        if start_id == goal_id {
            return Some(vec![start_id]);
        }

        let mut queue = Vec::new();
        let mut visited = Vec::new();
        let mut parents = Vec::new(); // To track the path back to the start node

        // Initialize
        queue.push(&start_id);
        visited.push(&start_id);

        while let Some(current) = queue.pop() {
            if let Some(children) = self.children(current) {
                for child in children {
                    if !visited.contains(&child) {
                        visited.push(child);
                        parents.push((child, current));

                        if child == &goal_id {
                            // If goal found, construct the path from parents
                            return Some(construct_path(&parents, &start_id, &goal_id));
                        }
                        queue.push(child);
                    }
                }
            }
        }

        None // No path found
    }

    pub fn clear_edges(&mut self) -> &mut Self {
        self.parents.clear();
        self.children.clear();
        self
    }

    // With no dependencies
    pub fn no_deps(&self) -> impl Iterator<Item = Node<&Data>> {
        self.nodes()
            .filter(|node| !self.has_children(&node.node_id))
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
        assert_eq!(graph.has_children("0"), true);
        assert_eq!(graph.has_parents("999"), true);
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
