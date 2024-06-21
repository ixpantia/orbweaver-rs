pub mod directed;
pub mod error;
pub mod readwrite;
pub(crate) mod utils;

// Prelude of data types and functionality.
pub mod prelude {
    pub(crate) type GraphInteractionResult<T> = Result<T, GraphInteractionError>;
    pub use crate::directed::acyclic::DirectedAcyclicGraph;
    pub use crate::directed::builder::DirectedGraphBuilder;
    pub use crate::directed::DirectedGraph;
    pub use crate::error::*;
}
