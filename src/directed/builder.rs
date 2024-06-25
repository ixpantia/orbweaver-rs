use std::rc::Rc;

use crate::utils::{interner::InternerBuilder, node_map::NodeMap, sym::Sym};
use rayon::prelude::*;

use super::{DirectedAcyclicGraph, DirectedGraph, GraphHasCycle};

#[derive(Clone)]
pub struct DirectedGraphBuilder {
    pub(crate) parents: Vec<Sym>,
    pub(crate) children: Vec<Sym>,
    pub(crate) interner: InternerBuilder,
}

fn find_leaves(parents: &[Sym], children: &[Sym]) -> Vec<Sym> {
    let mut leaves: Vec<_> = children
        .par_iter()
        .filter(|child| parents.binary_search(child).is_err())
        .copied()
        .collect();
    leaves.sort_unstable();
    leaves.dedup();
    leaves
}

fn find_roots(parents: &[Sym], children: &[Sym]) -> Vec<Sym> {
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
    pub(crate) fn get_or_intern(&mut self, val: impl AsRef<str>) -> Sym {
        self.interner.get_or_intern(val)
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

        let interner = Rc::new(self.interner.build());

        // Maps parents to their children
        let mut children_map = NodeMap::new(interner.len());

        for i in 0..self.parents.len() {
            let was_added = children_map
                .get_mut(self.parents[i])
                .or_init()
                .insert(self.children[i]);
            if was_added {
                n_edges += 1;
            }
        }

        // Maps children to their parents
        let mut parent_map = NodeMap::new(interner.len());

        for i in 0..self.parents.len() {
            parent_map
                .get_mut(self.children[i])
                .or_init()
                .insert(self.parents[i]);
        }

        // Every parent and child must have been initialized. If it
        // has not that means it's empty.
        for node in &nodes {
            let parent_entry = parent_map.get_mut(*node);
            let child_entry = children_map.get_mut(*node);
            if parent_entry.is_uninitialized() {
                parent_entry.into_empty();
            }
            if child_entry.is_uninitialized() {
                child_entry.into_empty();
            }
        }

        DirectedGraph {
            interner,
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
