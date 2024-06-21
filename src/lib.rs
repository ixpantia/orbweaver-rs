pub mod directed;
pub mod error;
pub mod readwrite;
pub(crate) mod utils;

pub(crate) const CURRENT_VERSION: [u32; 2] = [0, 12];

// Prelude of data types and functionality.
pub mod prelude {
    pub(crate) type GraphInteractionResult<T> = Result<T, GraphInteractionError>;
    pub use crate::directed::acyclic::DirectedAcyclicGraph;
    pub use crate::directed::builder::DirectedGraphBuilder;
    pub use crate::directed::DirectedGraph;
    pub use crate::error::*;
}
