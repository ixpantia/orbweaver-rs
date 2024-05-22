use crate::NodeId;

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
pub struct DuplicateNode(pub NodeId);

impl std::fmt::Display for DuplicateNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Unable to insert node, `{}` already exists",
            self.0.as_ref()
        )
    }
}

impl std::error::Error for DuplicateNode {}

#[derive(Debug)]
pub enum AddEdgeError {
    NodeNotExist(NodeId),
}

impl AddEdgeError {
    pub(crate) fn node_not_exists(id: impl AsRef<str>) -> Self {
        Self::NodeNotExist(NodeId::from(id.as_ref()))
    }
}

impl std::fmt::Display for AddEdgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeNotExist(node_id) => {
                write!(
                    f,
                    "Unable to add edge, the node `{}` does not exists",
                    node_id.as_ref()
                )
            }
        }
    }
}

impl std::error::Error for AddEdgeError {}
