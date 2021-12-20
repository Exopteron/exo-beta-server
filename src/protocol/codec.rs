use super::{ProtocolVersion, Readable, Writeable};
use bytes::BytesMut;
use flate2::{
    bufread::{ZlibDecoder, ZlibEncoder},
    Compression,
};
use std::io::{Cursor, Read};

/// State to serialize and deserialize packets from a byte stream.
#[derive(Default)]
pub struct MinecraftCodec {

    /// A buffer of received bytes.
    received_buf: BytesMut,
    /// Auxilary buffer.
    staging_buf: Vec<u8>,
    /// Another auxilary buffer.
    compression_target: Vec<u8>,
}

impl MinecraftCodec {
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets another `MinecraftCodec` with the same compression and encryption
    /// parameters.
    pub fn clone_with_settings(&self) -> MinecraftCodec {
        MinecraftCodec {
            received_buf: BytesMut::new(),
            staging_buf: Vec::new(),
            compression_target: Vec::new(),
        }
    }

    /// Writes a packet into the provided writer.
    pub fn encode(&mut self, packet: &impl Writeable, output: &mut Vec<u8>) -> anyhow::Result<()> {
        packet.write(&mut self.staging_buf, ProtocolVersion::Vb1_8_1)?;
        self.encode_uncompressed(output)?;
        self.staging_buf.clear();

        Ok(())
    }

    fn data_uncompressed(&mut self) -> (usize, &[u8]) {
        (0, self.staging_buf.as_slice())
    }

    fn encode_uncompressed(&mut self, output: &mut Vec<u8>) -> anyhow::Result<()> {
        // TODO: we should probably be able to determine the length without writing the packet,
        // which could remove an unnecessary copy.
        output.extend_from_slice(&self.staging_buf);

        Ok(())
    }

    /// Accepts newly received bytes.
    pub fn accept(&mut self, bytes: &[u8]) {
        let start_index = self.received_buf.len();
        self.received_buf.extend(bytes);
    }

    /// Gets the next packet that was received, if any.
    pub fn next_packet<T>(&mut self) -> anyhow::Result<Option<T>>
    where
        T: Readable,
    {
        let mut cursor = Cursor::new(&self.received_buf[..]);
        let start = cursor.position();
        let packet = T::read(&mut cursor, ProtocolVersion::Vb1_8_1);
        if packet.is_err() {
            return Ok(None);
        }
        let packet = packet.unwrap();
        let end = cursor.position();
        let bytes_read = (end - start) as usize;
        self.received_buf = self.received_buf.split_off(bytes_read);
        Ok(Some(packet))
    }
}
