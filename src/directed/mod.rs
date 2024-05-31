use serde::{Deserialize, Serialize};

use crate::prelude::*;
use std::collections::{HashMap, HashSet};
use std::ops::Not;

#[derive(Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DirectedGraph<Data> {
    pub(crate) nodes: HashMap<NodeId, Data>,
    pub(crate) parents: HashMap<NodeId, HashSet<NodeId>>,
    pub(crate) children: HashMap<NodeId, HashSet<NodeId>>,
    pub(crate) n_edges: usize,
}
struct DGVisitor<Data>(std::marker::PhantomData<Data>);

impl<Data> DGVisitor<Data> {
    fn new() -> Self {
        DGVisitor(std::marker::PhantomData)
    }
}

impl<'de, Data> serde::de::Visitor<'de> for DGVisitor<Data>
where
    Data: Deserialize<'de>,
{
    type Value = DirectedGraph<Data>;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("struct DirectedGraph")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        // Deserialize Vec<Rc<str>>
        let nodes: HashMap<NodeId, Data> = match map.next_entry()? {
            Some(("nodes", nodes_map)) => nodes_map,
            _ => {
                return Err(serde::de::Error::custom(
                    "The `nodes` field must always be the first",
                ))
            }
        };

        fn get_node_id<Err, Data>(map: &HashMap<NodeId, Data>, key: &str) -> Result<NodeId, Err>
        where
            Err: serde::de::Error,
        {
            map.get_key_value(key)
                .map(|(node_id, _)| node_id.clone())
                .ok_or_else(|| {
                    serde::de::Error::custom(format!("NodeId {} not found in nodes map", key))
                })
        }

        let mut parents = HashMap::new();
        let mut children = HashMap::new();
        let mut n_edges = None;

        while let Some(key) = map.next_key::<&str>()? {
            match key {
                "parents" => {
                    let parents_vec: HashMap<&str, Vec<&str>> = map.next_value()?;
                    for (k, v) in parents_vec {
                        let k = get_node_id(&nodes, k)?;
                        let v = v
                            .into_iter()
                            .map(|v| get_node_id(&nodes, v))
                            .collect::<Result<HashSet<NodeId>, _>>()?;
                        parents.insert(k, v);
                    }
                }
                "children" => {
                    let children_vec: HashMap<&str, Vec<&str>> = map.next_value()?;
                    for (k, v) in children_vec {
                        let k = get_node_id(&nodes, k)?;
                        let v = v
                            .into_iter()
                            .map(|v| get_node_id(&nodes, v))
                            .collect::<Result<HashSet<NodeId>, _>>()?;
                        children.insert(k, v);
                    }
                }
                "n_edges" => {
                    n_edges = Some(map.next_value()?);
                }
                _ => return Err(serde::de::Error::custom("Unknown field")),
            }
        }

        let n_edges =
            n_edges.ok_or_else(|| serde::de::Error::custom("field `n_edges` no included"))?;

        Ok(DirectedGraph {
            nodes,
            parents,
            children,
            n_edges,
        })
    }
}

impl<'de, Data> Deserialize<'de> for DirectedGraph<Data>
where
    Data: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(DGVisitor::<Data>::new())
    }
}

impl<Data: Clone> Clone for DirectedGraph<Data> {
    fn clone(&self) -> Self {
        DirectedGraph {
            nodes: HashMap::clone(&self.nodes),
            parents: HashMap::clone(&self.parents),
            children: HashMap::clone(&self.children),
            n_edges: self.n_edges,
        }
    }
}

impl<Data> DirectedGraph<&Data>
where
    Data: Clone,
{
    pub fn cloned(self) -> DirectedGraph<Data> {
        let nodes: HashMap<NodeId, Data> = self
            .nodes
            .iter()
            .map(|(key, &value)| (key.clone(), value.clone()))
            .collect();
        let parents = self.parents;
        let children = self.children;
        let n_edges = self.n_edges;
        DirectedGraph {
            nodes,
            parents,
            children,
            n_edges,
        }
    }
}

impl<Data> DirectedGraph<Data> {
    pub fn new() -> Self {
        DirectedGraph {
            nodes: HashMap::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
            n_edges: 0,
        }
    }

    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn add_node<'a>(
        &mut self,
        id: &'a str,
        data: Data,
    ) -> Result<&mut Self, DuplicateNode<'a>> {
        if self.nodes.contains_key(id) {
            return Err(DuplicateNode(id));
        }

        let node_id: NodeId = id.into();

        match self.nodes.insert(node_id.clone(), data) {
            Some(_) => Err(DuplicateNode(id)),
            _ => {
                self.children.insert(node_id.clone(), HashSet::new());
                self.parents.insert(node_id.clone(), HashSet::new());
                Ok(self)
            }
        }
    }

    pub fn update_node_data(
        &mut self,
        id: impl AsRef<str>,
        mut data: Data,
    ) -> GraphInteractionResult<Data> {
        match self.nodes.get_mut(id.as_ref()) {
            Some(node_data) => {
                std::mem::swap(&mut data, node_data);
                Ok(data)
            }
            None => Err(GraphInteractionError::node_not_exists(id)),
        }
    }

    pub fn get_node(&self, id: impl AsRef<str>) -> GraphInteractionResult<Node<&Data>> {
        if let Some((node_id, data)) = self.nodes.get_key_value(id.as_ref()) {
            return Ok(Node::new(node_id.clone(), data));
        }
        Err(GraphInteractionError::node_not_exists(id))
    }

    pub fn get_nodes(
        &self,
        ids: impl Iterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<Node<&Data>>> {
        ids.map(|node_id| self.get_node(node_id)).collect()
    }
    pub fn get_node_id(&self, id: impl AsRef<str>) -> GraphInteractionResult<NodeId> {
        if let Some((node_id, _)) = self.nodes.get_key_value(id.as_ref()) {
            return Ok(node_id.clone());
        }
        Err(GraphInteractionError::node_not_exists(id))
    }
    pub fn add_edge(
        &mut self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<&mut Self> {
        let from = self.get_node_id(&from)?;
        let to = self.get_node_id(&to)?;
        self.children
            .entry(from.clone())
            .or_default()
            .insert(to.clone());
        self.parents
            .entry(to.clone())
            .or_default()
            .insert(from.clone());
        self.n_edges += 1;
        Ok(self)
    }

    pub fn add_path(&mut self, path: &[impl AsRef<str>]) -> GraphInteractionResult<&mut Self> {
        if path.len() > 0 {
            for edge in path.windows(2) {
                let from = unsafe { edge.get_unchecked(0) };
                let to = unsafe { edge.get_unchecked(1) };
                self.add_edge(from, to)?;
            }
        }
        Ok(self)
    }

    pub fn edge_exists(&self, from: impl AsRef<str>, to: impl AsRef<str>) -> bool {
        if let Some(children) = self.children.get(from.as_ref()) {
            return children.contains(to.as_ref());
        }
        false
    }

    pub fn children(&self, node: impl AsRef<str>) -> GraphInteractionResult<&HashSet<NodeId>> {
        match self.children.get(node.as_ref()) {
            None => Err(GraphInteractionError::node_not_exists(node)),
            Some(children) => Ok(children),
        }
    }

    pub fn parents(&self, node: impl AsRef<str>) -> GraphInteractionResult<&HashSet<NodeId>> {
        match self.parents.get(node.as_ref()) {
            None => Err(GraphInteractionError::node_not_exists(node)),
            Some(parents) => Ok(parents),
        }
    }

    pub fn remove_edge(&mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> &mut Self {
        let children = self.children.get_mut(from.as_ref());
        let parents = self.parents.get_mut(to.as_ref());

        if let (Some(children), Some(parents)) = (children, parents) {
            if children.remove(to.as_ref()) && parents.remove(from.as_ref()) {
                self.n_edges -= 1;
            }
        }

        self
    }

    pub fn remove_node(&mut self, node_id: impl AsRef<str>) -> &mut Self {
        // Remove all edges that point to this
        if let Some(parents) = self.parents.get(node_id.as_ref()) {
            // We remove this node from the children list
            for parent in parents {
                if let Some(children) = self.children.get_mut(parent) {
                    children.remove(node_id.as_ref());
                }
            }
            self.parents.remove(node_id.as_ref());
        }

        // Remove all edges from this node to other nodes
        if let Some(children) = self.children.get(node_id.as_ref()) {
            // We remove this node from other node's parent list
            for child in children {
                if let Some(parents) = self.parents.get_mut(child) {
                    parents.remove(node_id.as_ref());
                }
            }
            self.children.remove(node_id.as_ref());
        }

        self.nodes.remove(node_id.as_ref());

        self
    }

    pub fn has_parents(&self, id: impl AsRef<str>) -> GraphInteractionResult<bool> {
        Ok(!self.parents(id)?.is_empty())
    }

    pub fn has_children(&self, id: impl AsRef<str>) -> GraphInteractionResult<bool> {
        Ok(!self.children(id)?.is_empty())
    }

    pub fn nodes(&self) -> impl Iterator<Item = Node<&Data>> {
        self.nodes
            .iter()
            .map(|(node_id, data)| Node::new(node_id.clone(), data))
    }

    pub fn node_ids(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.nodes.keys().cloned()
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
            n_edges: self.n_edges,
        }
    }
    /// Finds path using breadth-first search
    pub fn find_path(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Option<Vec<NodeId>>> {
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
            return Ok(Some(vec![start_id]));
        }

        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        let mut parents = Vec::new(); // To track the path back to the start node

        // Initialize
        queue.push(&start_id);
        visited.insert(&start_id);

        while let Some(current) = queue.pop() {
            for child in self.children(current)? {
                if !visited.contains(&child) {
                    visited.insert(child);
                    parents.push((child, current));

                    if child == &goal_id {
                        // If goal found, construct the path from parents
                        return Ok(Some(construct_path(&parents, &start_id, &goal_id)));
                    }
                    queue.push(child);
                }
            }
        }

        Ok(None)
    }

    pub fn clear_edges(&mut self) -> &mut Self {
        self.parents.clear();
        self.children.clear();
        self.n_edges = 0;
        self
    }

    pub fn least_common_parents(
        &self,
        selected: &[impl AsRef<str>],
    ) -> GraphInteractionResult<Vec<NodeId>> {
        let selected: HashSet<NodeId> = selected
            .iter()
            .map(|node| self.get_node_id(node.as_ref()))
            .collect::<GraphInteractionResult<_>>()?;
        let mut least_common_parent = selected
            .iter()
            .filter(|&node_id| {
                match self.parents.get(node_id.as_ref()) {
                    // We return true because if the node has no parent then
                    // it is part of the set of least common
                    // parents
                    None => true,
                    Some(parents) => parents
                        .iter()
                        .any(|parent| selected.contains(parent.as_ref()))
                        .not(),
                }
            })
            .cloned()
            .collect::<Vec<_>>();

        least_common_parent.sort_unstable();

        Ok(least_common_parent)
    }

    /// With no dependencies
    pub fn get_leaves(&self) -> Vec<NodeId> {
        let mut leaves = self
            .node_ids()
            .filter(|node_id| {
                !self
                    .has_children(node_id)
                    .expect("Using nodes from the graph directly")
            })
            .collect::<Vec<_>>();

        leaves.sort_unstable();

        leaves
    }

    /// Get leaves under a node or group of nodes
    pub fn get_leaves_under(
        &self,
        nodes: &[impl AsRef<str>],
    ) -> GraphInteractionResult<Vec<NodeId>> {
        let mut leaves = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = nodes
            .iter()
            .map(|node| self.get_node_id(node.as_ref()))
            .collect::<GraphInteractionResult<Vec<_>>>()?;

        while let Some(node) = to_visit.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            if self.has_children(&node)?.not() {
                leaves.push(node);
                continue;
            }
            self.children(&node)?
                .iter()
                .for_each(|child| to_visit.push(child.clone()));
        }

        Ok(leaves)
    }

    pub fn get_roots(&self) -> Vec<NodeId> {
        let mut leaves = self
            .node_ids()
            .filter(|node_id| {
                !self
                    .has_parents(node_id)
                    .expect("Using nodes from the graph directly")
            })
            .collect::<Vec<_>>();

        leaves.sort_unstable();

        leaves
    }

    pub fn get_roots_over(&self, nodes: &[impl AsRef<str>]) -> GraphInteractionResult<Vec<NodeId>> {
        let mut roots = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = nodes
            .iter()
            .map(|node| self.get_node_id(node.as_ref()))
            .collect::<GraphInteractionResult<Vec<_>>>()?;

        while let Some(node) = to_visit.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node.clone());
            if self.has_parents(&node)?.not() {
                roots.push(node);
                continue;
            }
            self.parents(&node)?
                .iter()
                .for_each(|parent| to_visit.push(parent.clone()));
        }

        Ok(roots)
    }

    /// Private, do not use outside
    fn subset_recursive<'a, 'b>(
        &'a self,
        parent: Option<&str>,
        node_id: impl AsRef<str>,
        new_dg: &'b mut DirectedGraph<&'a Data>,
        visited: &'b mut HashSet<NodeId>,
    ) -> GraphInteractionResult<()>
    where
        'a: 'b,
    {
        let node_id = node_id.as_ref();

        let node = self.get_node(node_id)?;

        if visited.contains(node_id) {
            if let Some(parent) = parent {
                new_dg.add_edge(parent, node_id)?;
            }
            return Ok(());
        }

        visited.insert(node.id());

        new_dg
            .add_node(node.id().as_ref(), node.data())
            .expect("Nodes cannot de duplicated");

        for child in self.children(node_id)?.iter() {
            self.subset_recursive(Some(node_id), child, new_dg, visited)?;
        }

        if let Some(parent) = parent {
            new_dg.add_edge(parent, node_id)?;
        }

        Ok(())
    }

    /// Returns a new tree that is the subset of of all children under a
    /// node.
    pub fn subset(&self, node_id: impl AsRef<str>) -> GraphInteractionResult<DirectedGraph<&Data>> {
        let mut new_dg = DirectedGraph::new();
        let mut visited = HashSet::new();

        self.subset_recursive(None, node_id.as_ref(), &mut new_dg, &mut visited)?;

        Ok(new_dg)
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
        assert!(graph.get_node("0").is_err());
    }

    #[test]
    fn test_get_node_existent() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("999", ());
        assert!(graph.get_node("999").is_ok());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("999", ());
        assert!(graph.add_edge("0", "999").is_ok());
        assert!(graph.has_children("0").unwrap());
        assert!(graph.has_parents("999").unwrap());
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

        assert_eq!(path.unwrap().len(), 5);
    }

    #[test]
    fn test_find_path_no_path() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("3", "4");

        let path = graph.find_path("0", "4").unwrap();

        assert_eq!(path, None);
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

        assert_eq!(path.unwrap().len(), 2);
    }

    #[test]
    fn test_least_common_parents() {
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

        assert_eq!(graph.least_common_parents(&["0", "1"]).unwrap(), vec!["0"]);
        assert_eq!(
            graph.least_common_parents(&["0", "1", "2"]).unwrap(),
            vec!["0"]
        );
        assert_eq!(
            graph.least_common_parents(&["0", "1", "2", "4"]).unwrap(),
            vec!["0"]
        );

        assert_eq!(
            graph.least_common_parents(&["0", "1", "3"]).unwrap(),
            vec!["0", "3"]
        );
    }

    #[test]
    fn test_remove_edge() {
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

        assert_eq!(
            graph.least_common_parents(&["0", "1", "2"]).unwrap(),
            vec!["0"]
        );

        graph.remove_edge("1", "2");

        assert_eq!(
            graph.least_common_parents(&["0", "1", "2"]).unwrap(),
            vec!["0", "2"]
        );
    }

    #[test]
    fn test_remove_node() {
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

        assert_eq!(
            graph.find_path("0", "3").unwrap().unwrap(),
            vec!["0", "1", "2", "3"]
        );

        graph.remove_node("2");

        assert_eq!(graph.find_path("0", "3").unwrap(), None);

        let _ = graph.add_node("2", ());

        assert_eq!(graph.find_path("0", "3").unwrap(), None);
    }

    #[test]
    fn test_get_leaves() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_node("5", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("0", "4");
        let _ = graph.add_edge("3", "5");

        assert_eq!(graph.get_leaves(), vec!["4", "5"]);
    }

    #[test]
    fn test_subset_tree_no_cycles() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_node("5", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("0", "4");
        let _ = graph.add_edge("3", "5");

        let subset_graph = graph.subset("1").unwrap();

        assert_eq!(subset_graph.get_leaves(), vec!["4", "5"]);
    }

    #[test]
    fn test_serde_json() {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("1", ());
        let _ = graph.add_node("2", ());
        let _ = graph.add_node("3", ());
        let _ = graph.add_node("4", ());
        let _ = graph.add_node("5", ());
        let _ = graph.add_edge("0", "1");
        let _ = graph.add_edge("1", "2");
        let _ = graph.add_edge("2", "3");
        let _ = graph.add_edge("3", "4");
        let _ = graph.add_edge("0", "4");
        let _ = graph.add_edge("3", "5");

        let json = serde_json::to_string_pretty(&graph).unwrap();

        println!("{}", json);

        let graph2 = serde_json::from_str(&json).unwrap();

        assert_eq!(graph, graph2);
    }
}
