#[cfg(feature = "binary")]
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

macro_rules! impl_read_write {
    // `()` indicates that the macro takes no argument.
    ($struct:ty) => {
        #[cfg(feature = "json")]
        impl $struct {
            pub fn to_json<W>(&self, writer: W) -> Result<(), serde_json::Error>
            where
                W: std::io::Write,
            {
                serde_json::to_writer(writer, self)
            }

            pub fn to_json_pretty<W>(&self, writer: W) -> Result<(), serde_json::Error>
            where
                W: std::io::Write,
            {
                serde_json::to_writer_pretty(writer, self)
            }

            pub fn from_json<R>(reader: R) -> Result<Self, serde_json::Error>
            where
                R: std::io::Read,
            {
                serde_json::from_reader(reader)
            }
        }

        #[cfg(feature = "binary")]
        impl $struct {
            pub fn to_binary<W>(&self, writer: W) -> Result<(), serde_cbor::Error>
            where
                W: std::io::Write,
            {
                serde_cbor::to_writer(ZlibEncoder::new(writer, Compression::default()), self)
            }

            pub fn from_binary<R>(reader: R) -> Result<Self, serde_cbor::Error>
            where
                R: std::io::Read,
            {
                serde_cbor::from_reader(ZlibDecoder::new(reader))
            }
        }
    };
}

impl_read_write!(crate::prelude::DirectedGraph);
impl_read_write!(crate::prelude::DirectedAcyclicGraph);
