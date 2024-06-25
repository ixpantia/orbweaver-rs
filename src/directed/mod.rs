pub mod acyclic;
pub mod builder;
mod debug;
mod get_rel2_on_rel1;

use self::get_rel2_on_rel1::get_values_on_rel_map;
use crate::{
    prelude::*,
    utils::{
        internal_bufs::InternalBufs,
        interner::Resolver,
        node_map::{LazySet, NodeMap},
        node_set::NodeVec,
        sym::Sym,
    },
};
use fxhash::FxHashSet;
use std::{collections::VecDeque, ops::Not, rc::Rc};

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DirectedGraph {
    pub(crate) interner: Rc<Resolver>,
    pub(crate) leaves: Vec<Sym>,
    pub(crate) roots: Vec<Sym>,
    pub(crate) nodes: Vec<Sym>,
    /// Maps parents to their children
    /// Key: Parent  | Value: Children
    pub(crate) children_map: NodeMap,
    /// Maps children to their parents
    /// Key: Child | Value: Parents
    pub(crate) parent_map: NodeMap,
    pub(crate) n_edges: usize,
    #[cfg_attr(feature = "serde", serde(skip_serializing, skip_deserializing))]
    pub(crate) buf: InternalBufs,
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
    impl_buf!(u32x1_vec_0, Vec<Sym>);
    impl_buf!(u32x1_vec_1, Vec<Sym>);
    impl_buf!(u32x1_vec_2, Vec<Sym>);
    impl_buf!(u32x2_vec_0, Vec<(Sym, Sym)>);
    impl_buf!(u32x1_queue_0, VecDeque<Sym>);
    impl_buf!(u32x1_set_0, FxHashSet<Sym>);
    impl_buf!(usizex2_queue_0, VecDeque<(usize, usize)>);

    #[inline(always)]
    pub(crate) fn resolve(&self, val: Sym) -> &str {
        unsafe { self.interner.resolve_unchecked(val) }
    }

    #[inline(always)]
    pub(crate) fn resolve_mul_slice(&self, nodes: &[Sym]) -> NodeVec {
        unsafe { self.interner.resolve_many_unchecked_from_slice(nodes) }
    }

    #[inline(always)]
    pub(crate) fn get_internal(&self, val: impl AsRef<str>) -> GraphInteractionResult<Sym> {
        self.interner
            .get(val.as_ref())
            .ok_or_else(|| GraphInteractionError::node_not_exists(val))
    }

    #[inline(always)]
    pub(crate) fn get_internal_mul(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
        buf: &mut Vec<Sym>,
    ) -> GraphInteractionResult<()> {
        for node in nodes {
            buf.push(self.get_internal(node)?);
        }
        Ok(())
    }

    #[inline]
    pub(crate) fn children_u32(&self, ids: &[Sym], out: &mut Vec<Sym>) {
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
    ) -> GraphInteractionResult<NodeVec> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let res = unsafe { self.u32x1_vec_1() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.children_u32(nodes_buf, res);
        Ok(self.resolve_mul_slice(res))
    }

    #[inline]
    pub(crate) fn parents_u32(&self, ids: &[Sym], out: &mut Vec<Sym>) {
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
    ) -> GraphInteractionResult<NodeVec> {
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
                    self.parent_map.contains_key(id)
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
                    .map(|id| self.children_map.contains_key(id))
            })
            .collect()
    }

    pub fn find_path(
        &self,
        from: impl AsRef<str>,
        to: impl AsRef<str>,
    ) -> GraphInteractionResult<NodeVec> {
        // Helper function for constructing the path
        fn construct_path(
            parents: &[(Sym, Sym)],
            start_id: Sym,
            goal_id: Sym,
            path: &mut Vec<Sym>,
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
            return Ok(self.resolve_mul_slice(&[from]));
        }

        let queue = unsafe { self.u32x1_queue_0() };
        let visited = unsafe { self.u32x1_set_0() };
        let path_buf = unsafe { self.u32x1_vec_0() };
        let parents = unsafe { self.u32x2_vec_0() }; // To track the path back to the start node

        // Initialize
        queue.push_back(from);
        visited.insert(from);

        'outer: while let Some(current) = queue.pop_front() {
            if let LazySet::Initialized(children) = self.children_map.get(current) {
                for &child in children.iter() {
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
    ) -> GraphInteractionResult<Vec<NodeVec>> {
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
                all_paths.push(Sym::RESERVED);
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
            .split(|&n| n.is_reserved())
            .filter(|p| !p.is_empty())
            .map(|path| self.resolve_mul_slice(path))
            .collect())
    }

    pub fn least_common_parents(
        &self,
        selected: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<NodeVec> {
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

    pub fn get_all_leaves(&self) -> NodeVec {
        self.resolve_mul_slice(&self.leaves)
    }

    fn get_leaves_under_u32(
        &self,
        to_visit: &mut Vec<Sym>,
        leaves: &mut Vec<Sym>,
        visited: &mut FxHashSet<Sym>,
    ) {
        while let Some(node) = to_visit.pop() {
            // If it was already present we continue
            if !visited.insert(node) {
                continue;
            }

            match self.children_map.get(node) {
                LazySet::Initialized(children) => {
                    children.iter().for_each(|child| to_visit.push(*child))
                }
                LazySet::Empty => leaves.push(node),
                _ => (),
            }
        }
    }

    /// This is a very expensive operation.
    pub fn get_leaves_under(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<NodeVec> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let leaves = unsafe { self.u32x1_vec_1() };
        let visited = unsafe { self.u32x1_set_0() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.get_leaves_under_u32(nodes_buf, leaves, visited);
        Ok(self.resolve_mul_slice(leaves))
    }

    pub fn get_all_roots(&self) -> NodeVec {
        self.resolve_mul_slice(&self.roots)
    }

    fn get_roots_over_u32(
        &self,
        to_visit: &mut Vec<Sym>,
        roots: &mut Vec<Sym>,
        visited: &mut FxHashSet<Sym>,
    ) {
        while let Some(node) = to_visit.pop() {
            // If it was already present we continue
            if !visited.insert(node) {
                continue;
            }

            match self.parent_map.get(node) {
                LazySet::Initialized(parents) => {
                    parents.iter().for_each(|parent| to_visit.push(*parent))
                }
                LazySet::Empty => roots.push(node),
                _ => (),
            }
        }
    }

    pub fn get_roots_over(
        &self,
        nodes: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> GraphInteractionResult<NodeVec> {
        let nodes_buf = unsafe { self.u32x1_vec_0() };
        let roots = unsafe { self.u32x1_vec_1() };
        let visited = unsafe { self.u32x1_set_0() };
        self.get_internal_mul(nodes, nodes_buf)?;
        self.get_roots_over_u32(nodes_buf, roots, visited);
        Ok(self.resolve_mul_slice(roots))
    }

    fn subset_u32(&self, node: Sym) -> DirectedGraph {
        // When subsetting the graph the only root will be
        // the node we select. This is because we are selecting
        // it and all their dependants.
        let roots = vec![node];
        let leaves = unsafe { self.u32x1_vec_0() };
        let visited = unsafe { self.u32x1_set_0() };
        let mut children_map = NodeMap::new(self.interner.len());
        let mut parent_map = NodeMap::new(self.interner.len());
        let mut queue = VecDeque::new();
        let mut n_edges = 0;

        parent_map.get_mut(node).into_empty();

        // Having no parent really only happens
        // on the first iteration. So we take it out of the loop
        // to optimize
        visited.insert(node);
        match self.children_map.get(node) {
            LazySet::Initialized(children) => children.iter().for_each(|&child| {
                queue.push_back((node, child));
            }),
            LazySet::Empty => {
                children_map.get_mut(node).into_empty();
                leaves.push(node);
            }
            LazySet::Uninitialized => (),
        }

        while let Some((parent, node)) = queue.pop_front() {
            // If we have a parent we add the relationship
            children_map.get_mut(parent).or_init().insert(node);
            parent_map.get_mut(node).or_init().insert(parent);
            n_edges += 1;

            // If we have already visited this node
            // we return :)
            if !visited.insert(node) {
                continue;
            }

            // If this node has children then
            // we recurse, else we insert it
            // as a leaf
            match self.children_map.get(node) {
                LazySet::Empty => {
                    leaves.push(node);
                    children_map.get_mut(node).into_empty();
                }
                LazySet::Initialized(children) => children.iter().for_each(|child| {
                    queue.push_back((node, *child));
                }),
                LazySet::Uninitialized => (),
            }
        }

        let mut nodes = parent_map.initialized_keys();
        nodes.push(node);

        // Re order values
        nodes.sort_unstable();
        nodes.dedup();

        leaves.sort_unstable();
        leaves.dedup();

        DirectedGraph {
            interner: Rc::clone(&self.interner),
            nodes,
            leaves: leaves.clone(),
            roots,
            n_edges,
            parent_map,
            children_map,
            buf: InternalBufs::default(),
        }
    }

    /// Returns a new tree that is the subset of of all children under a
    /// node.
    pub fn subset(&self, node: impl AsRef<str>) -> GraphInteractionResult<DirectedGraph> {
        self.get_internal(node).map(|node| self.subset_u32(node))
    }

    pub fn nodes(&self) -> NodeVec {
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

    #[test]
    fn dg_builder_add_edge() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("hello", "world");
        assert_eq!(builder.parents, [0], "Parent is not equal");
        assert_eq!(builder.children, [1], "Children is not equal");
    }

    #[test]
    fn dg_builder_add_path() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_path(["hello", "world", "again"]);
        assert_eq!(builder.parents, [0, 1], "Parent is not equal");
        assert_eq!(builder.children, [1, 2], "Children is not equal");
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
            dg.children(["hello"]).unwrap(),
            ["4", "1", "3", "0", "2"],
            "Parent is not equal"
        );
        assert_eq!(
            dg.children(["hello", "other"]).unwrap(),
            vec!["4", "1", "3", "0", "2", "6", "8", "5", "10", "7", "9"],
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
            ["1", "D", "H", "B"]
        );
        assert_eq!(dg.get_leaves_under(["A"]).unwrap(), ["D", "H", "B"]);
        assert_eq!(dg.get_leaves_under(["C"]).unwrap(), ["D", "H"]);
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
        builder.add_edge("0", "1");
        builder.add_edge("A", "B");
        builder.add_edge("A", "C");
        builder.add_edge("C", "D");
        builder.add_edge("C", "H");
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
