use serde::{Deserialize, Serialize};
use std::{ops::Deref, rc::Rc};

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

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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

impl From<&NodeId> for String {
    fn from(value: &NodeId) -> Self {
        value.as_ref().to_string()
    }
}

impl std::cmp::PartialEq<str> for NodeId {
    fn eq(&self, other: &str) -> bool {
        self.0.as_ref().eq(other)
    }
}

impl std::cmp::PartialEq<&str> for NodeId {
    fn eq(&self, other: &&str) -> bool {
        self.0.as_ref().eq(*other)
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

impl<Data> Node<&Data>
where
    Data: Clone,
{
    #[inline(always)]
    pub fn cloned(self) -> Node<Data> {
        Node {
            data: self.data.clone(),
            node_id: self.node_id,
        }
    }
}

impl<Data> Clone for Node<Data>
where
    Data: Clone,
{
    fn clone(&self) -> Self {
        Node {
            data: self.data.clone(),
            node_id: self.node_id.clone(),
        }
    }
}
