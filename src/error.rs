use crate::CURRENT_VERSION;

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
pub enum BinaryError {
    IO(std::io::Error),
    #[cfg(feature = "binary")]
    Cbor(serde_cbor::Error),
    Version([u32; 2]),
}

#[cfg(feature = "binary")]
impl From<serde_cbor::Error> for BinaryError {
    fn from(v: serde_cbor::Error) -> Self {
        Self::Cbor(v)
    }
}

impl From<std::io::Error> for BinaryError {
    fn from(v: std::io::Error) -> Self {
        Self::IO(v)
    }
}

impl std::fmt::Display for BinaryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error while reading the OW binary format: {err}"),
            #[cfg(feature = "binary")]
            Self::Cbor(err) => write!(f, "{err}"),
            Self::Version(version) => write!(
                f,
                "Tried to read a OW binary generated with version {}.{}. You are using version {}.{}",
                version[0], version[1],
                CURRENT_VERSION[0], CURRENT_VERSION[1]
            )
        }
    }
}

impl std::error::Error for BinaryError {}

#[derive(Debug)]
pub enum GraphInteractionError {
    NodeNotExist(Box<str>),
    InternalResolve(u32),
    ZeroSubsetLimit,
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
            Self::ZeroSubsetLimit => {
                write!(f, "Cannot set a `0` limit for a subset operation")
            }
        }
    }
}

impl std::error::Error for GraphInteractionError {}
