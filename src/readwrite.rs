#[cfg(feature = "binary")]
use base64::engine::general_purpose;

#[cfg(feature = "binary")]
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

macro_rules! impl_read_write {
    // `()` indicates that the macro takes no argument.
    ($struct:ty) => {
        #[cfg(feature = "binary")]
        impl $struct {
            pub fn to_binary<W>(&self, writer: W) -> Result<(), serde_cbor::Error>
            where
                W: std::io::Write,
            {
                let writer = base64::write::EncoderWriter::new(
                    ZlibEncoder::new(writer, Compression::default()),
                    &general_purpose::STANDARD,
                );
                serde_cbor::to_writer(writer, self)
            }

            pub fn from_binary<R>(reader: R) -> Result<Self, serde_cbor::Error>
            where
                R: std::io::Read,
            {
                let reader = base64::read::DecoderReader::new(
                    ZlibDecoder::new(reader),
                    &general_purpose::STANDARD,
                );

                serde_cbor::from_reader(reader)
            }
        }
    };
}

impl_read_write!(crate::prelude::DirectedGraph);
impl_read_write!(crate::prelude::DirectedAcyclicGraph);

#[cfg(test)]
mod tests {
    use crate::prelude::{DirectedGraph, DirectedGraphBuilder};

    #[test]
    fn directed_from_and_to_binary() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("1", "2");
        builder.add_edge("2", "3");
        builder.add_edge("3", "4");
        builder.add_edge("4", "5");
        builder.add_edge("5", "6");
        builder.add_edge("6", "7");
        builder.add_edge("7", "8");
        builder.add_edge("8", "9");
        builder.add_edge("9", "10");
        builder.add_edge("10", "11");
        builder.add_edge("11", "12");
        builder.add_edge("12", "13");
        let dg = builder.build_directed();

        let mut buffer = Vec::new();
        dg.to_binary(&mut buffer).unwrap();

        let de_dg = DirectedGraph::from_binary(buffer.as_slice()).unwrap();

        assert_eq!(format!("{:?}", dg), format!("{:?}", de_dg));
    }

    #[test]
    fn directed_acyclic_from_and_to_binary() {
        let mut builder = DirectedGraphBuilder::new();
        builder.add_edge("1", "2");
        builder.add_edge("2", "3");
        builder.add_edge("3", "4");
        builder.add_edge("4", "5");
        builder.add_edge("5", "6");
        builder.add_edge("6", "7");
        builder.add_edge("7", "8");
        builder.add_edge("8", "9");
        builder.add_edge("9", "10");
        builder.add_edge("10", "11");
        builder.add_edge("11", "12");
        builder.add_edge("12", "13");
        let dg = builder.build_acyclic().unwrap();

        let mut buffer = Vec::new();
        dg.to_binary(&mut buffer).unwrap();

        let de_dg = DirectedGraph::from_binary(buffer.as_slice()).unwrap();

        assert_eq!(format!("{:?}", dg), format!("{:?}", de_dg));
    }
}
