

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
pub struct DuplicateNode(String);

impl DuplicateNode {
    pub fn new(id: impl AsRef<str>) -> DuplicateNode {
        DuplicateNode(format!(
            "Unable to insert node, `{}` already exists",
            id.as_ref()
        ))
    }
}

impl std::fmt::Display for DuplicateNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for DuplicateNode {}

#[derive(Debug)]
pub enum GraphInteractionError {
    NodeNotExist,
}

impl std::fmt::Display for GraphInteractionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NodeNotExist => {
                write!(f, "Node does not exist")
            }
        }
    }
}

impl std::error::Error for GraphInteractionError {}
