pub mod acyclic;
mod get_rel2_on_rel1;

use fxhash::FxBuildHasher;
use rayon::prelude::*;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
//use string_interner::backend::BucketBackend;
//use string_interner::StringInterner;
use crate::interner::{InternerBuilder, Resolver};

use self::acyclic::DirectedAcyclicGraph;
use self::get_rel2_on_rel1::get_values_on_rel_map;
use crate::prelude::*;
use std::cell::UnsafeCell;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ops::Not;
use std::rc::Rc;

#[derive(Clone)]
pub struct DirectedGraphBuilder {
    pub(crate) parents: Vec<u32>,
    pub(crate) children: Vec<u32>,
    pub(crate) interner: InternerBuilder,
}

#[derive(Default)]
pub(crate) struct InternalBufs {
    // Espacio en memoria para buffers
    pub(crate) u32x1_vec_0: UnsafeCell<Vec<u32>>,
    pub(crate) u32x1_vec_1: UnsafeCell<Vec<u32>>,
    pub(crate) u32x1_vec_2: UnsafeCell<Vec<u32>>,
    pub(crate) u32x2_vec_0: UnsafeCell<Vec<(u32, u32)>>,
    pub(crate) u32x1_queue_0: UnsafeCell<VecDeque<u32>>,
    pub(crate) u32x1_set_0: UnsafeCell<HashSet<u32, FxBuildHasher>>,
    pub(crate) usizex2_queue_0: UnsafeCell<VecDeque<(usize, usize)>>,
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DirectedGraph {
    pub(crate) interner: Rc<Resolver>,
    pub(crate) leaves: Vec<u32>,
    pub(crate) roots: Vec<u32>,
    pub(crate) nodes: Vec<u32>,
    /// Maps parents to their children
    /// Key: Parent  | Value: Children
    pub(crate) children_map: HashMap<u32, HashSet<u32, FxBuildHasher>, FxBuildHasher>,
    /// Maps children to their parents
    /// Key: Child | Value: Parents
    pub(crate) parent_map: HashMap<u32, HashSet<u32, FxBuildHasher>, FxBuildHasher>,
    pub(crate) n_edges: usize,
    #[cfg_attr(feature = "serde", serde(skip_serializing, skip_deserializing))]
    pub(crate) buf: InternalBufs,
}

fn find_leaves(parents: &[u32], children: &[u32]) -> Vec<u32> {
    let mut leaves: Vec<_> = children
        .par_iter()
        .filter(|child| parents.binary_search(child).is_err())
        .copied()
        .collect();
    leaves.sort_unstable();
    leaves.dedup();
    leaves
}

fn find_roots(parents: &[u32], children: &[u32]) -> Vec<u32> {
    let mut roots: Vec<_> = parents
        .par_iter()
        .filter(|parent| children.binary_search(parent).is_err())
        .copied()
        .collect();
    roots.sort_unstable();
    roots.dedup();
    roots
}

impl DirectedGraphBuilder {
    pub fn new() -> Self {
        DirectedGraphBuilder {
            interner: InternerBuilder::new(),
            children: Vec::new(),
            parents: Vec::new(),
        }
    }

    #[inline(always)]
    pub(crate) fn get_or_intern(&mut self, val: impl AsRef<str>) -> u32 {
        unsafe { std::mem::transmute(self.interner.get_or_intern(val)) }
    }
    pub fn add_edge(&mut self, from: impl AsRef<str>, to: impl AsRef<str>) -> &mut Self {
        let from = self.get_or_intern(&from);
        let to = self.get_or_intern(&to);
        self.parents.push(from);
        self.children.push(to);
        self
    }
    pub fn add_path(&mut self, path: impl IntoIterator<Item = impl AsRef<str>>) -> &mut Self {
        let mut path = path.into_iter().peekable();
        while let (Some(from), Some(to)) = (path.next(), path.peek()) {
            self.add_edge(from.as_ref(), to.as_ref());
        }
        self
    }

    pub fn build_directed(self) -> DirectedGraph {
        // When we build we will do some optimizations
        let mut unique_parents = self.parents.clone();
        unique_parents.sort_unstable();
        unique_parents.dedup();
        unique_parents.shrink_to_fit();

        let mut unique_children = self.children.clone();
        unique_children.sort_unstable();
        unique_children.dedup();
        unique_parents.shrink_to_fit();

        let mut nodes = Vec::new();
        nodes.extend_from_slice(&unique_parents);
        nodes.extend_from_slice(&unique_children);
        nodes.sort_unstable();
        nodes.dedup();
        nodes.shrink_to_fit();

        let leaves = find_leaves(&unique_parents, &unique_children);
        let roots = find_roots(&unique_parents, &unique_children);

        let mut n_edges = 0;

        // Maps parents to their children
        let mut children_map = HashMap::<u32, HashSet<_, _>, _>::default();

        for i in 0..self.parents.len() {
            let was_added = children_map
                .entry(self.parents[i])
                .or_default()
                .insert(self.children[i]);
            if was_added {
                n_edges += 1;
            }
        }

        // Maps children to their parents
        let mut parent_map = HashMap::<u32, HashSet<_, _>, _>::default();

        for i in 0..self.parents.len() {
            parent_map
                .entry(self.children[i])
                .or_default()
                .insert(self.parents[i]);
        }

        DirectedGraph {
            interner: Rc::new(self.interner.build()),
            leaves,
            roots,
            nodes,
            children_map,
            parent_map,
            n_edges,
            buf: Default::default(),
        }
    }

    pub fn build_acyclic(self) -> Result<DirectedAcyclicGraph, GraphHasCycle> {
        DirectedAcyclicGraph::build(self.build_directed())
    }
}

impl Default for DirectedGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DirectedGraph {
    fn clone(&self) -> Self {
        let interner = self.interner.clone();
        let leaves = self.leaves.clone();
        let roots = self.roots.clone();
        let nodes = self.nodes.clone();
        let children_map = self.children_map.clone();
        let parent_map = self.parent_map.clone();
        let n_edges = self.n_edges;

        DirectedGraph {
            interner,
            n_edges,
            parent_map,
            children_map,
            nodes,
            roots,
            leaves,
            buf: Default::default(),
        }
    }
}

impl std::fmt::Debug for DirectedGraph {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        let n_nodes = self.children_map.len();
        let n_edges = self.n_edges;
        let n_roots = self.roots.len();
        let n_leaves = self.leaves.len();
        writeln!(f, "# of nodes: {n_nodes}")?;
        writeln!(f, "# of edges: {n_edges}")?;
        writeln!(f, "# of roots: {n_roots}")?;
        writeln!(f, "# of leaves: {n_leaves}")?;
        writeln!(f)?;
        writeln!(f, "|   Parent   |    Child   |")?;
        writeln!(f, "| ---------- | ---------- |")?;
        let mut n_printed = 0;
        'outer: for (parent, children) in self.children_map.iter() {
            for child in children {
                n_printed += 1;
                writeln!(f, "| {:0>10} | {:0>10} |", parent, child)?;
                if n_printed == 10 {
                    break 'outer;
                }
            }
        }

        if n_nodes > 10 {
            writeln!(f, "Omitted {} nodes", n_nodes - 10)?;
        }

        Ok(())
    }
}

macro_rules! impl_buf {
    ($name:ident, $ret:ty) => {
        #[inline(always)]
        #[allow(clippy::mut_from_ref)]
        pub(crate) unsafe fn $name(&self) -> &mut $ret {
            let ptr = unsafe { &mut *self.buf.$name.get() };
            ptr.clear();
            ptr
        }
    };
}

impl DirectedGraph {
    impl_buf!(u32x1_vec_0, Vec<u32>);
    impl_buf!(u32x1_vec_1, Vec<u32>);
    impl_buf!(u32x1_vec_2, Vec<u32>);
    impl_buf!(u32x2_vec_0, Vec<(u32, u32)>);
    impl_buf!(u32x1_queue_0, VecDeque<u32>);
    impl_buf!(u32x1_set_0, HashSet<u32, FxBuildHasher>);
    impl_buf!(usizex2_queue_0, VecDeque<(usize, usize)>);

    #[inline(always)]
    pub(crate) fn resolve(&self, val: u32) -> &str {
        unsafe { self.interner.resolve_unchecked(val) }
    }

    #[inline(always)]
    pub(crate) fn resolve_mul_slice(&self, nodes: &[u32]) -> Vec<&str> {
        unsafe { self.interner.resolve_many_unchecked_from_slice(nodes) }
    }

    #[inline(always)]
    pub(crate) fn get_internal(&self, val: impl AsRef<str>) -> GraphInteractionResult<u32> {
        self.interner
            .get(val.as_ref())
            .map(|v| v.get())
            .ok_or_else(|| GraphInteractionError::node_not_exists(val))
    }

    #[inline(always)]
    pub(crate) fn get_internal_mul(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
        buf: &mut Vec<u32>,
    ) -> GraphInteractionResult<()> {
        for node in nodes {
            buf.push(self.get_internal(node)?);
        }
        Ok(())
    }

    pub(crate) fn edge_exists(&self, from: u32, to: u32) -> bool {
        self.children_map
            .get(&from)
            .map(|children| children.contains(&to))
            .unwrap_or(false)
    }

    #[inline]
    pub(crate) fn children_u32(&self, ids: &[u32], out: &mut Vec<u32>) {
        // Gets the children for the given parent
        get_values_on_rel_map(ids, &self.children_map, out)
    }

    /// This function returns the children of a given set of
    /// nodes.
    ///
    /// NOTE: The returned `Vec` may include deuplicates.
    /// For performance reasons, the responsability of
    /// dedupping is on the user since it adds
    /// significant overhead to the operation.
    pub fn children(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<&str>> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let res = unsafe { self.u32x1_vec_1() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.children_u32(nodes_buf, res);
        Ok(self.resolve_mul_slice(res))
    }

    #[inline]
    pub(crate) fn parents_u32(&self, ids: &[u32], out: &mut Vec<u32>) {
        // Gets the parents for the given children
        get_values_on_rel_map(ids, &self.parent_map, out)
    }

    /// This function returns the parents of a given set of
    /// nodes.
    ///
    /// NOTE: The returned `Vec` may include deuplicates.
    /// For performance reasons, the responsability of
    /// dedupping is on the user since it adds
    /// significant overhead to the operation.
    pub fn parents(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<&str>> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let res = unsafe { self.u32x1_vec_1() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.parents_u32(nodes_buf, res);
        Ok(self.resolve_mul_slice(res))
    }

    pub fn has_parents(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<bool>> {
        nodes
            .into_iter()
            .map(|x| {
                self.get_internal(x).map(|id| {
                    // If we can find it on the children map
                    // that means that it has parents
                    self.parent_map.contains_key(&id)
                })
            })
            .collect()
    }

    pub fn has_children(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<bool>> {
        nodes
            .into_iter()
            .map(|x| {
                self.get_internal(x)
                    .map(|id| self.children_map.contains_key(&id))
            })
            .collect()
    }

    pub fn find_path(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Vec<&str>> {
        // Helper function for constructing the path
        fn construct_path(
            parents: &[(u32, u32)],
            start_id: u32,
            goal_id: u32,
            path: &mut Vec<u32>,
        ) {
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
        }

        let from = self.get_internal(from)?;
        let to = self.get_internal(to)?;

        if from == to {
            return Ok(vec![self.resolve(from)]);
        }

        let queue = unsafe { self.u32x1_queue_0() };
        let visited = unsafe { self.u32x1_set_0() };
        let path_buf = unsafe { self.u32x1_vec_0() };
        let parents = unsafe { self.u32x2_vec_0() }; // To track the path back to the start node

        // Initialize
        queue.push_back(from);
        visited.insert(from);

        'outer: while let Some(current) = queue.pop_front() {
            if let Some(children) = self.children_map.get(&current) {
                for &child in children {
                    if visited.insert(child) {
                        parents.push((child, current));
                        if child == to {
                            // Construct the path and place it in `path_buf`
                            construct_path(parents, from, to, path_buf);
                            break 'outer;
                        }
                        queue.push_back(child);
                    }
                }
            }
        }

        Ok(self.resolve_mul_slice(path_buf))
    }

    /// Finds all paths on a DG using BFS
    pub fn find_all_paths(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<Vec<Vec<&str>>> {
        const PATH_DELIM: u32 = 0;

        let from = self.get_internal(from)?;
        let to = self.get_internal(to)?;

        let path_buf = unsafe { self.u32x1_vec_0() };
        let children = unsafe { self.u32x1_vec_1() };
        let all_paths = unsafe { self.u32x1_vec_2() };
        let queue = unsafe { self.usizex2_queue_0() };

        path_buf.push(from);
        queue.push_back((0, 0));

        while let Some((starti, endi)) = queue.pop_front() {
            let last = path_buf[endi];

            if last == to {
                all_paths.extend_from_slice(&path_buf[starti..=endi]);
                all_paths.push(PATH_DELIM);
            } else {
                self.children_u32(&[last], children);
                for child in children.drain(..) {
                    if !path_buf[starti..=endi].contains(&child) {
                        let start = path_buf.len();
                        path_buf.extend_from_within(starti..=endi);
                        path_buf.push(child);
                        let end = path_buf.len() - 1;
                        queue.push_back((start, end));
                    }
                }
            }
        }

        Ok(all_paths
            .split(|&n| n == PATH_DELIM)
            .filter(|p| !p.is_empty())
            .map(|path| self.resolve_mul_slice(path))
            .collect())
    }

    pub fn least_common_parents(
        &self,
        selected: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<&str>> {
        // Declare used buffers
        let selected_buf = unsafe { self.u32x1_vec_0() };
        let selected_buf_set = unsafe { self.u32x1_set_0() };
        let parents = unsafe { self.u32x1_vec_1() };
        let least_common_parents = unsafe { self.u32x1_vec_2() };

        self.get_internal_mul(selected, selected_buf)?;
        selected_buf_set.extend(selected_buf.iter().copied());

        selected_buf.iter().for_each(|&child| {
            self.parents_u32(&[child], parents);
            let parent_not_in_selection = parents
                .drain(..)
                .any(|parent| selected_buf_set.contains(&parent))
                .not();
            if parent_not_in_selection {
                least_common_parents.push(child);
            }
        });

        least_common_parents.sort_unstable();
        least_common_parents.dedup();

        Ok(self.resolve_mul_slice(least_common_parents))
    }

    pub fn get_all_leaves(&self) -> Vec<&str> {
        self.resolve_mul_slice(&self.leaves)
    }

    fn get_leaves_under_u32(
        &self,
        to_visit: &mut Vec<u32>,
        leaves: &mut Vec<u32>,
        visited: &mut HashSet<u32, FxBuildHasher>,
    ) {
        while let Some(node) = to_visit.pop() {
            // If it was already present we continue
            if !visited.insert(node) {
                continue;
            }

            match self.children_map.get(&node) {
                Some(children) => children.iter().for_each(|child| to_visit.push(*child)),
                None if self.leaves.binary_search(&node).is_ok() => leaves.push(node),
                None => (),
            }
        }
    }

    /// This is a very expensive operation.
    pub fn get_leaves_under(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<&str>> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let leaves = unsafe { self.u32x1_vec_1() };
        let visited = unsafe { self.u32x1_set_0() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.get_leaves_under_u32(nodes_buf, leaves, visited);
        Ok(self.resolve_mul_slice(leaves))
    }

    pub fn get_all_roots(&self) -> Vec<&str> {
        self.resolve_mul_slice(&self.roots)
    }

    fn get_roots_over_u32(
        &self,
        to_visit: &mut Vec<u32>,
        roots: &mut Vec<u32>,
        visited: &mut HashSet<u32, FxBuildHasher>,
    ) {
        while let Some(node) = to_visit.pop() {
            // If it was already present we continue
            if !visited.insert(node) {
                continue;
            }

            match self.parent_map.get(&node) {
                Some(parents) => parents.iter().for_each(|parent| to_visit.push(*parent)),
                None if self.roots.binary_search(&node).is_ok() => roots.push(node),
                None => (),
            }
        }
    }

    pub fn get_roots_over(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<Vec<&str>> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let roots = unsafe { self.u32x1_vec_1() };
        let visited = unsafe { self.u32x1_set_0() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.get_roots_over_u32(nodes_buf, roots, visited);
        Ok(self.resolve_mul_slice(roots))
    }

    fn subset_u32(&self, node: u32) -> DirectedGraph {
        let mut new_dg = DirectedGraph {
            interner: self.interner.clone(),
            // When subsetting the graph the only root will be
            // the node we select. This is because we are selecting
            // it and all their dependants.
            roots: vec![node],
            leaves: Vec::new(),
            children_map: HashMap::default(),
            parent_map: HashMap::default(),
            nodes: Vec::new(),
            n_edges: 0,
            buf: Default::default(),
        };

        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        // Since the main value is a NonZeroU32
        // we can use a 0 as None :D minor
        // optimization
        queue.push_back((0, node));

        while let Some((parent, node)) = queue.pop_front() {
            // If we have a parent we add the relationship
            if parent != 0 {
                new_dg.children_map.entry(parent).or_default().insert(node);
                new_dg.parent_map.entry(node).or_default().insert(parent);
                new_dg.n_edges += 1;
            }

            // If we have already visited this node
            // we return :)
            if !visited.insert(node) {
                continue;
            }

            // If this node has children then
            // we recurse, else we insert it
            // as a leaf
            match self.children_map.get(&node) {
                None => new_dg.leaves.push(node),
                Some(children) => children.iter().for_each(|&child| {
                    queue.push_back((node, child));
                }),
            }
        }

        let nodes = new_dg.parent_map.keys().copied().collect();
        new_dg.nodes = nodes;
        new_dg.nodes.push(node);

        // Re order values
        new_dg.nodes.sort_unstable();
        new_dg.nodes.dedup();

        new_dg.leaves.sort_unstable();
        new_dg.leaves.dedup();

        // There should only be one root in a subset
        assert_eq!(new_dg.roots.len(), 1);

        new_dg
    }

    /// Returns a new tree that is the subset of of all children under a
    /// node.
    pub fn subset(&self, node: impl AsRef<str>) -> GraphInteractionResult<DirectedGraph> {
        self.get_internal(node).map(|node| self.subset_u32(node))
    }

    pub fn nodes(&self) -> Vec<&str> {
        self.resolve_mul_slice(&self.nodes)
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sort(mut vec: Vec<&str>) -> Vec<&str> {
        vec.sort_unstable();
        vec
    }

    #[test]
    fn dg_builder_add_edge() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("hello", "world");
        assert_eq!(builder.parents, [1], "Parent is not equal");
        assert_eq!(builder.children, [2], "Children is not equal");
    }

    #[test]
    fn dg_builder_add_path() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_path(["hello", "world", "again"]);
        assert_eq!(builder.parents, [1, 2], "Parent is not equal");
        assert_eq!(builder.children, [2, 3], "Children is not equal");
    }

    #[test]
    fn dg_get_children() {
        let mut builder = DirectedGraphBuilder::new();
        // We put more than 8 children to
        // test if SIMD actually workd
        builder.add_edge("hello", "0");
        builder.add_edge("hello", "1");
        builder.add_edge("hello", "2");
        builder.add_edge("hello", "3");
        builder.add_edge("hello", "4");
        builder.add_edge("other", "5");
        builder.add_edge("other", "6");
        builder.add_edge("other", "7");
        builder.add_edge("other", "8");
        builder.add_edge("other", "9");
        builder.add_edge("other", "10");
        let dg = builder.build_directed();
        assert_eq!(
            sort(dg.children(["hello"]).unwrap()),
            ["0", "1", "2", "3", "4"],
            "Parent is not equal"
        );
        assert_eq!(
            sort(dg.children(["hello", "other"]).unwrap()),
            vec!["0", "1", "10", "2", "3", "4", "5", "6", "7", "8", "9"],
            "Parent is not equal"
        );
    }

    #[test]
    fn dg_get_parents() {
        let mut builder = DirectedGraphBuilder::new();
        // We put more than 8 children to
        // test if SIMD actually workd
        builder.add_edge("hello", "0");
        builder.add_edge("hello", "1");
        builder.add_edge("hello", "2");
        builder.add_edge("hello", "3");
        builder.add_edge("hello", "A");
        builder.add_edge("other", "A");
        builder.add_edge("other", "6");
        builder.add_edge("other", "7");
        builder.add_edge("other", "8");
        builder.add_edge("other", "9");
        builder.add_edge("other", "10");
        let dg = builder.build_directed();
        assert_eq!(dg.parents(["A"]).unwrap(), vec!["hello", "other"],);
        assert_eq!(
            dg.parents(["A", "0"]).unwrap(),
            vec!["hello", "other", "hello"],
        );
    }

    #[test]
    fn dg_has_parents() {
        let mut builder = DirectedGraphBuilder::new();
        // We put more than 8 children to
        // test if SIMD actually workd
        builder.add_edge("hello", "0");
        builder.add_edge("hello", "1");
        builder.add_edge("hello", "2");
        builder.add_edge("hello", "3");
        builder.add_edge("hello", "A");
        builder.add_edge("other", "A");
        builder.add_edge("other", "6");
        builder.add_edge("other", "7");
        builder.add_edge("other", "8");
        builder.add_edge("other", "9");
        builder.add_edge("other", "10");
        let dg = builder.build_directed();
        assert_eq!(
            dg.has_parents(["A", "0", "hello", "10"]).unwrap(),
            [true, true, false, true]
        );
    }

    #[test]
    fn dg_has_children() {
        let mut builder = DirectedGraphBuilder::new();
        // We put more than 8 children to
        // test if SIMD actually workd
        builder.add_edge("hello", "0");
        builder.add_edge("hello", "1");
        builder.add_edge("hello", "2");
        builder.add_edge("hello", "3");
        builder.add_edge("hello", "A");
        builder.add_edge("other", "A");
        builder.add_edge("other", "6");
        builder.add_edge("other", "7");
        builder.add_edge("other", "8");
        builder.add_edge("other", "9");
        builder.add_edge("other", "10");
        let dg = builder.build_directed();
        assert_eq!(
            dg.has_children(["hello", "other", "9", "0"]).unwrap(),
            [true, true, false, false]
        );
    }

    #[test]
    fn dg_find_path() {
        let mut builder = DirectedGraphBuilder::new();
        // We put more than 8 children to
        // test if SIMD actually workd
        builder.add_path(["A", "B", "C", "D"]);
        let dg = builder.clone().build_directed();
        assert_eq!(dg.find_path("A", "D").unwrap(), ["A", "B", "C", "D"]);

        builder.add_path(["A", "H", "D"]);
        let dg = builder.clone().build_directed();
        assert_eq!(dg.find_path("A", "D").unwrap(), ["A", "H", "D"]);
        assert_eq!(dg.children(["A"]).unwrap(), ["H", "B"]);
    }

    #[test]
    fn dg_find_least_common_parents() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("A", "B");
        builder.add_edge("A", "C");
        builder.add_edge("C", "D");
        let dg = builder.clone().build_directed();
        assert_eq!(dg.least_common_parents(["B", "D"]).unwrap(), ["B", "D"]);
        assert_eq!(dg.least_common_parents(["B", "C"]).unwrap(), ["B", "C"]);
        assert_eq!(
            dg.least_common_parents(["B", "C", "D"]).unwrap(),
            ["B", "C"]
        );
        assert_eq!(
            dg.least_common_parents(["A", "B", "C", "D"]).unwrap(),
            ["A"]
        );

        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("A", "B");
        builder.add_edge("B", "C");
        builder.add_edge("C", "D");
        builder.add_edge("C", "E");
        builder.add_edge("F", "D");
        let dg = builder.clone().build_directed();
        assert_eq!(
            dg.least_common_parents(["A", "B", "C", "D", "E", "F"])
                .unwrap(),
            ["A", "F"]
        );
    }

    #[test]
    fn dg_get_all_leaves() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("A", "B");
        builder.add_edge("A", "C");
        builder.add_edge("C", "D");
        builder.add_edge("C", "H");
        builder.add_edge("0", "1");
        let dg = builder.clone().build_directed();
        assert_eq!(dg.get_all_leaves(), ["B", "D", "H", "1"]);
    }

    #[test]
    fn dg_get_leaves_under() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("A", "B");
        builder.add_edge("A", "C");
        builder.add_edge("C", "D");
        builder.add_edge("C", "H");
        builder.add_edge("0", "1");
        let dg = builder.clone().build_directed();
        assert_eq!(
            dg.get_leaves_under(["A", "0"]).unwrap(),
            ["1", "H", "D", "B"]
        );
        assert_eq!(dg.get_leaves_under(["A"]).unwrap(), ["H", "D", "B"]);
        assert_eq!(dg.get_leaves_under(["C"]).unwrap(), ["H", "D"]);
    }

    #[test]
    fn dg_get_roots_over() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("A", "B");
        builder.add_edge("A", "C");
        builder.add_edge("C", "D");
        builder.add_edge("C", "H");
        builder.add_edge("0", "1");
        let dg = builder.clone().build_directed();
        assert_eq!(dg.get_roots_over(["A", "0"]).unwrap(), ["0", "A"]);
        assert_eq!(dg.get_roots_over(["H"]).unwrap(), ["A"]);
        assert_eq!(dg.get_roots_over(["H", "C", "1"]).unwrap(), ["0", "A"]);
    }

    #[test]
    fn dg_subset() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("A", "B");
        builder.add_edge("A", "C");
        builder.add_edge("C", "D");
        builder.add_edge("C", "H");
        builder.add_edge("0", "1");
        let dg = builder.clone().build_directed();
        let dg2 = dg.subset("A").unwrap();
        // This should not include children on "0" since
        // the subset has no concept of those relationships
        assert_eq!(dg2.get_roots_over(["A"]).unwrap(), ["A"]);
        assert_eq!(dg2.get_roots_over(["H"]).unwrap(), ["A"]);
        assert_eq!(dg2.get_roots_over(["H", "C", "1"]).unwrap(), ["A"]);
        assert_eq!(dg2.nodes(), ["A", "B", "C", "D", "H"]);
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
        let dg = builder.build_directed();

        let actual = format!("{:?}", dg);

        assert_eq!(
            actual,
            "# of nodes: 12\n# of edges: 12\n# of roots: 1\n# of leaves: 1\n\n|   Parent   |    Child   |\n| ---------- | ---------- |\n| 0000000010 | 0000000011 |\n| 0000000007 | 0000000008 |\n| 0000000004 | 0000000005 |\n| 0000000001 | 0000000002 |\n| 0000000011 | 0000000012 |\n| 0000000008 | 0000000009 |\n| 0000000005 | 0000000006 |\n| 0000000002 | 0000000003 |\n| 0000000012 | 0000000013 |\n| 0000000009 | 0000000010 |\nOmitted 2 nodes\n"
        )
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
}
