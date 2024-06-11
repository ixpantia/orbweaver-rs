#[cfg(feature = "binary")]
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};

#[cfg(feature = "binary")]
struct Base64Safe<W: std::io::Write>(W);

#[cfg(feature = "binary")]
impl<W: std::io::Write> std::io::Write for Base64Safe<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf)
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        if let Err(e) = self.0.write_all(buf) {
            match e.kind() {
                std::io::ErrorKind::WriteZero => return Ok(()),
                _ => return Err(e),
            }
        }
        Ok(())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

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

    #[test]
    fn directed_from_and_to_binary() {
        let mut builder = DirectedGraphBuilder::new();
        for i in 0..10000 {
            builder.add_edge(i.to_string(), (i + 1).to_string());
        }
        let dg = builder.build_directed();

        let mut buffer = Vec::new();
        dg.to_binary(&mut buffer).unwrap();

        let de_dg = DirectedGraph::from_binary(buffer.as_slice()).unwrap();

        assert_eq!(format!("{:?}", dg), format!("{:?}", de_dg));
    }

    #[test]
    fn directed_acyclic_from_and_to_binary() {
        let mut builder = DirectedGraphBuilder::new();
        for i in 0..10000 {
            builder.add_edge(i.to_string(), (i + 1).to_string());
        }
        let dg = builder.build_acyclic().unwrap();

        let mut buffer = Vec::new();
        dg.to_binary(&mut buffer).unwrap();

        let de_dg = DirectedGraph::from_binary(buffer.as_slice()).unwrap();

        assert_eq!(format!("{:?}", dg), format!("{:?}", de_dg));
    }
}
