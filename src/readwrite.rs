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
                let writer = ZlibEncoder::new(writer, Compression::default());
                serde_cbor::to_writer(writer, self)
            }

            pub fn from_binary<R>(reader: R) -> Result<Self, serde_cbor::Error>
            where
                R: std::io::Read,
            {
                let reader = ZlibDecoder::new(reader);

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

    #[cfg(feature = "binary")]
    #[test]
    fn directed_from_and_to_binary() {
        let mut builder = DirectedGraphBuilder::new();
        for i in 0..100 {
            builder.add_edge(i.to_string(), (i + 1).to_string());
        }
        let dg = builder.build_directed();

        let mut buffer = Vec::new();
        dg.to_binary(&mut buffer).unwrap();

        let de_dg = DirectedGraph::from_binary(buffer.as_slice()).unwrap();

        assert_eq!(format!("{:?}", dg), format!("{:?}", de_dg));
    }

    #[cfg(feature = "binary")]
    #[test]
    fn directed_acyclic_from_and_to_binary() {
        let mut builder = DirectedGraphBuilder::new();
        for i in 0..100 {
            builder.add_edge(i.to_string(), (i + 1).to_string());
        }
        let dg = builder.build_acyclic().unwrap();

        let mut buffer = Vec::new();
        dg.to_binary(&mut buffer).unwrap();

        let de_dg = DirectedGraph::from_binary(buffer.as_slice()).unwrap();

        assert_eq!(format!("{:?}", dg), format!("{:?}", de_dg));
        assert_eq!(dg.nodes(), de_dg.nodes());
        assert_eq!(dg.get_all_leaves(), de_dg.get_all_leaves());
        assert_eq!(dg.get_all_roots(), de_dg.get_all_roots());
    }
}
