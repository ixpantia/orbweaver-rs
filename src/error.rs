#[derive(Debug)]
pub struct GraphHasCycle;

impl std::fmt::Display for GraphHasCycle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to topologically sort, graph has at least one cycle"
        )
    }
}

impl std::error::Error for GraphHasCycle {}

#[derive(Debug)]
pub enum GraphInteractionError {
    NodeNotExist(Box<str>),
    InternalResolve(u32),
}

impl GraphInteractionError {
    pub(crate) fn node_not_exists(id: impl AsRef<str>) -> Self {
        Self::NodeNotExist(id.as_ref().into())
    }
}

impl std::fmt::Display for GraphInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeNotExist(node_id) => {
                write!(f, "Node `{}` does not exist", node_id)
            }
            Self::InternalResolve(symbol) => {
                write!(f, "Internal symbol `{}` does not exist", symbol)
            }
        }
    }
}

impl std::error::Error for GraphInteractionError {}
