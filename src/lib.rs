use lasso::Spur;
use serde::{Deserialize, Serialize};


pub mod acyclic;
pub mod directed;
pub mod error;

/// Prelude of data types and functionality.
pub mod prelude {
    pub(crate) type GraphInteractionResult<T> = Result<T, GraphInteractionError>;
    pub use crate::acyclic::DirectedAcyclicGraph;
    pub use crate::directed::DirectedGraph;
    pub use crate::error::*;
    pub use crate::Graph;
    pub use crate::Node;
    pub use crate::NodeId;
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Graph<Data> {
    Directed(directed::DirectedGraph<Data>),
    DirectedAcyclic(acyclic::DirectedAcyclicGraph<Data>),
}

impl<Data> From<acyclic::DirectedAcyclicGraph<Data>> for Graph<Data> {
    fn from(v: acyclic::DirectedAcyclicGraph<Data>) -> Self {
        Self::DirectedAcyclic(v)
    }
}

impl<Data> From<directed::DirectedGraph<Data>> for Graph<Data> {
    fn from(v: directed::DirectedGraph<Data>) -> Self {
        Self::Directed(v)
    }
}

#[allow(clippy::result_large_err)]
impl<Data> Graph<Data> {
    pub fn try_into_directed(self) -> Result<directed::DirectedGraph<Data>, Self> {
        if let Self::Directed(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    pub fn try_into_directed_acyclic(self) -> Result<acyclic::DirectedAcyclicGraph<Data>, Self> {
        if let Self::DirectedAcyclic(v) = self {
            Ok(v)
        } else {
            Err(self)
        }
    }

    /// Returns `true` if the graph is [`Directed`].
    ///
    /// [`Directed`]: Graph::Directed
    #[must_use]
    pub fn is_directed(&self) -> bool {
        matches!(self, Self::Directed(..))
    }

    /// Returns `true` if the graph is [`DirectedAcyclic`].
    ///
    /// [`DirectedAcyclic`]: Graph::DirectedAcyclic
    #[must_use]
    pub fn is_directed_acyclic(&self) -> bool {
        matches!(self, Self::DirectedAcyclic(..))
    }
}

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Hash, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct NodeId(Spur);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let internal = self.0.into_inner();
        write!(f, "NodeId({})", internal)
    }
}

impl<'a> From<&'a NodeId> for &'a Spur {
    fn from(val: &'a NodeId) -> Self {
        &val.0
    }
}

pub struct Node<Id, Data> {
    node_id: Id,
    data: Data,
}

impl<'a, Data> Node<&'a str, Data> {
    #[inline(always)]
    fn new(node_id: &'a str, data: Data) -> Self {
        Node { node_id, data }
    }
    #[inline(always)]
    pub fn id(&self) -> &'a str {
        self.node_id
    }
    #[inline(always)]
    pub fn data(&self) -> &Data {
        &self.data
    }
    #[inline(always)]
    pub fn data_mut(&mut self) -> &mut Data {
        &mut self.data
    }
}

impl<'a, Data> Node<&'a str, &Data>
where
    Data: Clone,
{
    #[inline(always)]
    pub fn cloned(self) -> Node<Box<str>, Data> {
        Node {
            data: self.data.clone(),
            node_id: self.node_id.into(),
        }
    }
}

impl<Id, Data> Clone for Node<Id, Data>
where
    Data: Clone,
    Id: Clone,
{
    fn clone(&self) -> Self {
        Node {
            data: self.data.clone(),
            node_id: self.node_id.clone(),
        }
    }
}

pub struct NodeIdSet<Iter>
where
    Iter: Iterator<Item = NodeId>,
{
    iterator: Iter,
}

impl<Iter> NodeIdSet<Iter>
where
    Iter: Iterator<Item = NodeId>,
{
    pub(crate) fn new(iter: impl IntoIterator<IntoIter = Iter>) -> Self {
        let iterator = iter.into_iter();
        Self { iterator }
    }
}

struct NodeSet<Id, Data> {
    ids: Vec<Id>,
    data: Vec<Data>,
    len: usize,
}

impl<Id, Data> NodeSet<Id, Data> {
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn from_single(node: Node<Id, Data>) -> Self {
        [node].into_iter().collect()
    }
}

impl<Id, Data> FromIterator<Node<Id, Data>> for NodeSet<Id, Data> {
    fn from_iter<T: IntoIterator<Item = Node<Id, Data>>>(iter: T) -> Self {
        let mut ids = Vec::new();
        let mut data = Vec::new();
        iter.into_iter().for_each(|node| {
            ids.push(node.node_id);
            data.push(node.data);
        });
        let len = ids.len();
        NodeSet { ids, data, len }
    }
}

impl<'a, Data> IntoIterator for NodeSet<&'a str, Data> {
    type Item = Node<&'a str, Data>;
    type IntoIter = NodeIter<&'a str, Data>;
    fn into_iter(self) -> Self::IntoIter {
        NodeIter {
            node_set: self,
            curr: 0,
        }
    }
}

struct NodeIter<Id, Data> {
    node_set: NodeSet<Id, Data>,
    curr: usize,
}

impl<'a, Data> Iterator for NodeIter<&'a str, Data> {
    type Item = Node<&'a str, Data>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.curr == self.node_set.len {
            return None;
        }
        let node_id = self.node_set.ids[self.curr];
        let data = unsafe { (&self.node_set.data[self.curr] as *const Data).read() };
        self.curr += 1;
        Some(Node { node_id, data })
    }
}
