use serde::{Deserialize, Serialize};

use crate::nodeset::NodeSet;
use crate::{prelude::GraphInteractionError as GIE, prelude::*};
use lasso::{Rodeo, Spur};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet};
use std::ops::Not;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DirectedGraph<Data> {
    pub(crate) nodes: crate::nodeset::NodeSet<Data>,
    pub(crate) parents: HashMap<NodeId, HashSet<NodeId>>,
    pub(crate) children: HashMap<NodeId, HashSet<NodeId>>,
    pub(crate) n_edges: usize,
}

impl<Data: Clone> Clone for DirectedGraph<Data> {
    fn clone(&self) -> Self {
        DirectedGraph {
            nodes: NodeSet::clone(&self.nodes),
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
        let nodes = self.nodes.cloned();
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
            nodes: NodeSet::new(),
            parents: HashMap::new(),
            children: HashMap::new(),
            n_edges: 0,
        }
    }

    pub fn n_nodes(&self) -> usize {
        self.nodes.len()
    }

    pub fn add_node(&mut self, id: impl AsRef<str>, data: Data) -> Result<NodeId, DuplicateNode> {
        let node_id = self.nodes.add_node(id.as_ref(), data)?;
        self.children.entry(node_id).or_default();
        self.parents.entry(node_id).or_default();
        Ok(node_id)
    }

    pub fn update_node_data(
        &mut self,
        node_id: NodeId,
        mut data: Data,
    ) -> GraphInteractionResult<Data> {
        if let Some(node_data) = self.nodes.get_data_mut(node_id) {
            std::mem::swap(node_data, &mut data);
        }

        Ok(data)
    }

    pub fn get_node(&self, node_id: NodeId) -> GraphInteractionResult<Node<&str, &Data>> {
        if let Some((node_hrid, data)) = self.nodes.get_key_value(node_id) {
            return Ok(Node::new(node_hrid, data));
        }
        Err(GIE::NodeNotExist)
    }

    pub fn id(&self, hrid: impl AsRef<str>) -> GraphInteractionResult<NodeId> {
        self.nodes.get_id(hrid.as_ref()).ok_or(GIE::NodeNotExist)
    }

    pub fn ids(
        &self,
        hrid: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<NodeId>> {
        hrid.into_iter()
            .map(|hrid| self.id(hrid.as_ref()))
            .collect()
    }

    pub fn get_nodes(
        &self,
        ids: impl Iterator<Item = NodeId>,
    ) -> GraphInteractionResult<Vec<Node<&str, &Data>>> {
        ids.map(|node_id| self.get_node(node_id)).collect()
    }

    pub fn add_edge(&mut self, from: NodeId, to: NodeId) -> GraphInteractionResult<&mut Self> {
        self.children.entry(from).or_default().insert(to);
        self.parents.entry(to).or_default().insert(from);
        self.n_edges += 1;
        Ok(self)
    }

    pub fn add_path(
        &mut self,
        path: impl IntoIterator<Item = NodeId> + ExactSizeIterator,
    ) -> GraphInteractionResult<&mut Self> {
        if path.len() > 1 {
            let mut path = path.into_iter().peekable();
            while let (Some(from), Some(&to)) = (path.next(), path.peek()) {
                self.add_edge(from, to);
            }
        }
        Ok(self)
    }

    pub fn edge_exists(&self, from: NodeId, to: NodeId) -> bool {
        if let Some(children) = self.children.get(&from) {
            return children.contains(&to);
        }
        false
    }

    pub fn children(&self, node_id: NodeId) -> GraphInteractionResult<Vec<NodeId>> {
        match self.children.get(&node_id) {
            None => Err(GIE::NodeNotExist),
            Some(children) => Ok(children.iter().copied().collect()),
        }
    }

    pub fn parents(&self, node_id: NodeId) -> GraphInteractionResult<Vec<NodeId>> {
        match self.parents.get(&node_id) {
            None => Err(GIE::NodeNotExist),
            Some(parents) => Ok(parents.iter().copied().collect()),
        }
    }

    pub fn remove_edge(&mut self, from: NodeId, to: NodeId) -> &mut Self {
        let children = self.children.get_mut(&from);
        let parents = self.parents.get_mut(&to);

        if let (Some(children), Some(parents)) = (children, parents) {
            if children.remove(&to) && parents.remove(&from) {
                self.n_edges -= 1;
            }
        }

        self
    }

    pub fn remove_node(&mut self, node_id: NodeId) -> &mut Self {
        // Remove all edges that point to this
        if let Some(parents) = self.parents.get(&node_id) {
            // We remove this node from the children list
            for parent in parents {
                if let Some(children) = self.children.get_mut(parent) {
                    children.remove(&node_id);
                }
            }
            self.parents.remove(&node_id);
        }

        // Remove all edges from this node to other nodes
        if let Some(children) = self.children.get(&node_id) {
            // We remove this node from other node's parent list
            for child in children {
                if let Some(parents) = self.parents.get_mut(child) {
                    parents.remove(&node_id);
                }
            }
            self.children.remove(&node_id);
        }

        self.nodes.remove_node(node_id);

        self
    }

    pub fn has_parents(&self, node_id: NodeId) -> GraphInteractionResult<bool> {
        Ok(!self
            .parents
            .get(&node_id)
            .ok_or(GIE::NodeNotExist)?
            .is_empty())
    }

    pub fn has_children(&self, node_id: NodeId) -> GraphInteractionResult<bool> {
        Ok(!self
            .children
            .get(&node_id)
            .ok_or(GIE::NodeNotExist)?
            .is_empty())
    }

    pub fn nodes(&self) -> Vec<NodeId> {
        self.nodes.node_ids().collect()
    }

    pub fn into_dataless(&self) -> DirectedGraph<()> {
        DirectedGraph {
            nodes: self.nodes.into_dataless(),
            parents: self.parents.clone(),
            children: self.children.clone(),
            n_edges: self.n_edges,
        }
    }
    /// Finds path using breadth-first search
    pub fn find_path(
        &self,
        from: NodeId,
        to: NodeId,
    ) -> GraphInteractionResult<Option<Vec<NodeId>>> {
        // Helper function for constructing the path
        fn construct_path(
            parents: &[(NodeId, NodeId)],
            start_id: NodeId,
            goal_id: NodeId,
        ) -> Vec<NodeId> {
            let mut path = Vec::new();
            let mut current_id = goal_id;
            path.push(current_id);

            while current_id != start_id {
                if let Some(parent_pair) = parents.iter().find(|(node, _)| *node == current_id) {
                    current_id = parent_pair.1;
                    path.push(current_id);
                } else {
                    break; // This should not happen if the path exists
                }
            }

            path.reverse(); // Reverse to get the path from start to goal
            path
        }

        let start = from;
        let goal = to;

        if start == goal {
            return Ok(Some(vec![start]));
        }

        let mut queue = Vec::new();
        let mut visited = HashSet::new();
        let mut parents = Vec::new(); // To track the path back to the start node

        // Initialize
        queue.push(start);
        visited.insert(start);

        while let Some(current) = queue.pop() {
            for &child in self.children.get(&current).ok_or(GIE::NodeNotExist)? {
                if !visited.contains(&child) {
                    visited.insert(child);
                    parents.push((child, current));

                    if child == goal {
                        // If goal found, construct the path from parents
                        return Ok(Some(construct_path(&parents, start, goal)));
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
        selected: impl IntoIterator<Item = NodeId>,
    ) -> GraphInteractionResult<Vec<NodeId>> {
        let selected: Vec<NodeId> = selected.into_iter().collect();
        let mut least_common_parent = selected
            .iter()
            .filter(|&node_id| {
                match self.parents.get(node_id) {
                    // We return true because if the node has no parent then
                    // it is part of the set of least common
                    // parents
                    None => true,
                    Some(parents) => parents.iter().any(|parent| selected.contains(parent)).not(),
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
            .nodes
            .node_ids()
            .filter(|node_id| {
                !self
                    .children
                    .get(node_id)
                    .map(|c| !c.is_empty())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        leaves.sort_unstable();

        leaves
    }

    /// Get leaves under a node or group of nodes
    pub fn get_leaves_under(
        &self,
        nodes: impl IntoIterator<Item = NodeId>,
    ) -> GraphInteractionResult<Vec<NodeId>> {
        let mut leaves = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = nodes.into_iter().collect::<Vec<_>>();

        while let Some(node) = to_visit.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            if self.has_children(node)?.not() {
                leaves.push(node);
                continue;
            }
            self.children
                .get(&node)
                .ok_or(GIE::NodeNotExist)?
                .iter()
                .for_each(|&child| to_visit.push(child));
        }

        Ok(leaves)
    }

    pub fn get_roots(&self) -> Vec<NodeId> {
        let mut roots = self
            .nodes
            .node_ids()
            .filter(|node_id| {
                !self
                    .parents
                    .get(node_id)
                    .map(|p| !p.is_empty())
                    .unwrap_or_default()
            })
            .collect::<Vec<_>>();

        roots.sort_unstable();

        roots
    }

    pub fn get_roots_over(
        &self,
        nodes: impl IntoIterator<Item = NodeId>,
    ) -> GraphInteractionResult<Vec<NodeId>> {
        let mut roots = Vec::new();
        let mut visited = HashSet::new();
        let mut to_visit = nodes.into_iter().collect::<Vec<_>>();

        while let Some(node) = to_visit.pop() {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);
            if self.has_parents(node)?.not() {
                roots.push(node);
                continue;
            }
            self.parents
                .get(&node)
                .ok_or(GIE::NodeNotExist)?
                .iter()
                .for_each(|parent| to_visit.push(*parent));
        }

        Ok(roots)
    }

    /// Private, do not use outside
    fn subset_recursive<'a, 'b>(
        &'a self,
        parent: Option<NodeId>,
        node_id: NodeId,
        new_dg: &'b mut DirectedGraph<&'a Data>,
        visited: &'b mut HashSet<NodeId>,
    ) -> GraphInteractionResult<()> {
        let node = self.get_node(node_id)?;

        if visited.contains(&node_id) {
            if let Some(parent) = parent {
                new_dg.add_edge(parent, node_id)?;
            }
            return Ok(());
        }

        visited.insert(node_id);

        new_dg
            .nodes
            .add_entry_unchecked(node.node_id, node_id, node.data);

        for child in self.children(node_id)? {
            self.subset_recursive(Some(node_id), child, new_dg, visited)?;
        }

        if let Some(parent) = parent {
            new_dg.add_edge(parent, node_id)?;
        }

        Ok(())
    }

    /// Returns a new tree that is the subset of of all children under a
    /// node.
    pub fn subset(&self, node_id: NodeId) -> GraphInteractionResult<DirectedGraph<&Data>> {
        let mut new_dg = DirectedGraph::new();
        let mut visited = HashSet::new();

        self.subset_recursive(None, node_id, &mut new_dg, &mut visited)?;

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

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_create_a_new_directed_graph() {
        let _ = DirectedGraph::<()>::new();
    }

    #[test]
    fn test_add_nodes_to_a_graph() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        graph.add_node("0", ())?;
        graph.add_node("1", ())?;
        graph.add_node("2", ())?;
        Ok(())
    }

    #[test]
    fn test_add_repeated_node_errors() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        graph.add_node("0", ())?;
        assert!(graph.add_node("0", ()).is_err());
        Ok(())
    }

    #[test]
    fn test_get_node_non_existent() -> TestResult {
        let graph = DirectedGraph::<()>::new();
        assert!(graph.id("0").is_err());
        Ok(())
    }

    #[test]
    fn test_get_node_existent() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let _ = graph.add_node("0", ());
        let _ = graph.add_node("999", ());
        assert!(graph.get_node(graph.id("999")?).is_ok());
        Ok(())
    }

    #[test]
    fn test_add_edge() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_0 = graph.add_node("0", ())?;
        let id_999 = graph.add_node("999", ())?;
        assert!(graph.add_edge(id_0, id_999).is_ok());
        assert!(graph.has_children(id_0).unwrap());
        assert!(graph.has_parents(id_999).unwrap());
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

        let path = graph.find_path(id_0, id_4).unwrap();

        assert_eq!(path.unwrap().len(), 5);
        Ok(())
    }

    #[test]
    fn test_find_path_no_path() -> TestResult {
        let mut graph = DirectedGraph::<()>::new();
        let id_0 = graph.add_node("0", ())?;
        let id_1 = graph.add_node("1", ())?;
        let id_2 = graph.add_node("2", ())?;
        let id_3 = graph.add_node("3", ())?;
        let id_4 = graph.add_node("4", ())?;
        let _ = graph.add_edge(id_0, id_1);
        let _ = graph.add_edge(id_1, id_2);
        let _ = graph.add_edge(id_3, id_4);

        let path = graph.find_path(id_0, id_4).unwrap();

        assert_eq!(path, None);
        Ok(())
    }

    #[test]
    fn test_least_common_parents() -> TestResult {
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

        assert_eq!(
            graph.least_common_parents([id_0, id_1]).unwrap(),
            vec![id_0]
        );
        assert_eq!(
            graph.least_common_parents([id_0, id_1, id_2]).unwrap(),
            vec![id_0]
        );
        assert_eq!(
            graph
                .least_common_parents([id_0, id_1, id_2, id_4])
                .unwrap(),
            vec![id_0]
        );

        assert_eq!(
            graph.least_common_parents([id_0, id_1, id_3]).unwrap(),
            vec![id_0, id_3]
        );
        Ok(())
    }

    #[test]
    fn test_remove_edge() -> TestResult {
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

        assert_eq!(graph.least_common_parents([id_0, id_1, id_2])?, vec![id_0]);

        graph.remove_edge(id_1, id_2);

        assert_eq!(
            graph.least_common_parents([id_0, id_1, id_2])?,
            vec![id_0, id_2]
        );
        Ok(())
    }

    #[test]
    fn test_remove_node() -> TestResult {
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

        assert_eq!(
            graph.find_path(id_0, id_3)?.unwrap(),
            vec![id_0, id_1, id_2, id_3]
        );

        graph.remove_node(id_2);

        assert_eq!(graph.find_path(id_0, id_3)?, None);

        let _id_2_new = graph.add_node("2", ())?;

        assert_eq!(graph.find_path(id_0, id_3)?, None);
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

        let path = graph.find_path(id_0, id_4)?.unwrap();

        assert_eq!(path.len(), 2);
        Ok(())
    }

    #[test]
    fn test_get_leaves() -> TestResult {
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

        assert_eq!(graph.get_leaves(), vec![id_4, id_5]);
        Ok(())
    }

    #[test]
    fn test_subset_tree_no_cycles() -> TestResult {
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

        let subset_graph = graph.subset(id_1)?;
        assert_eq!(subset_graph.get_leaves(), vec![id_4, id_5]);
        Ok(())
    }
}
