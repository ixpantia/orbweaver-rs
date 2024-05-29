use lasso::Spur;
use serde::{Deserialize, Serialize};

pub mod acyclic;
pub mod directed;
pub mod error;
pub mod nodeset;

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
pub struct NodeId(pub i32);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let internal = self.0;
        write!(f, "NodeId({})", internal)
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
