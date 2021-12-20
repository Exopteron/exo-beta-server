use std::{
    fmt::{self, Debug},
    marker::PhantomData,
    sync::Arc, io::Write,
};

use serde::{
    de,
    de::{SeqAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

use crate::{
    protocol::{ProtocolVersion, Readable, Writeable},
    world::chunk_lock::ChunkHandle,
};

#[derive(Debug, Clone)]
pub enum ChunkDataKind {
    /// Load a chunk on the client. Sends all sections + biomes.
    LoadChunk,
    /// Overwrite an existing chunk on the client. Sends
    /// only the sections in `sections`.
    OverwriteChunk { sections: Vec<usize> },
}

/// Packet to load a chunk on the client.
#[derive(Clone)]
pub struct ChunkData {
    /// The chunk to send.
    pub chunk: ChunkHandle,

    /// Whether this packet will load a chunk on
    /// the client or overwrite an existing one.
    pub kind: ChunkDataKind,
}

impl Debug for ChunkData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_struct = f.debug_struct("ChunkData");
        debug_struct.field("position", &self.chunk.read().position());
        debug_struct.field("kind", &self.kind);
        debug_struct.finish()
    }
}

impl ChunkData {
    fn should_skip_section(&self, y: usize) -> bool {
        match &self.kind {
            ChunkDataKind::LoadChunk => false,
            ChunkDataKind::OverwriteChunk { sections } => !sections.contains(&y),
        }
    }
}

impl Writeable for ChunkData {
    fn write(&self, buffer: &mut Vec<u8>, version: ProtocolVersion) -> anyhow::Result<()> {
        let chunk = self.chunk.read();
        let mut first = true;
        for (y, section) in chunk.sections().iter().enumerate().take(8) {
            if let Some(section) = section {
                if self.should_skip_section(y) {
                    continue;
                }
                if !first {
                    0x33i8.write(buffer, version)?;
                } else {
                    first = false;
                }
                let mut blockdata =
                    Vec::with_capacity(chunk.data.len() + (chunk.data.len() as f32 * 1.5) as usize);
                let mut metadata = Vec::with_capacity(chunk.data.len());
                let mut blocklight = Vec::with_capacity(chunk.data.len());
                let mut skylight = Vec::with_capacity(chunk.data.len());
                for byte in section.data() {
                    blockdata.push(byte.b_type);
                    metadata.push(byte.b_metadata);
                    blocklight.push(byte.b_light);
                    skylight.push(byte.b_skylight);
                }
                //metadata.reverse();
                blockdata.append(&mut compress_to_nibble(metadata)?);
                blockdata.append(&mut compress_to_nibble(blocklight)?);
                blockdata.append(&mut compress_to_nibble(skylight)?);
                let mut data = Vec::with_capacity(blockdata.len());
                let mut compressor =
                    flate2::write::ZlibEncoder::new(&mut data, flate2::Compression::fast());
                compressor.write_all(&blockdata)?;
                compressor.finish()?;
                chunk.position().x.write(buffer, version)?;
                (y as i16).write(buffer, version)?;
                chunk.position().z.write(buffer, version)?;
                15i8.write(buffer, version)?;
                15i8.write(buffer, version)?;
                15i8.write(buffer, version)?;
                (data.len() as i32).write(buffer, version)?;
                buffer.append(&mut data);
            }
        }
        Ok(())
    }
}

fn make_nibble_byte(mut a: u8, mut b: u8) -> anyhow::Result<u8> {
    if a > 15 || b > 15 {
        return Err(anyhow::anyhow!("Out of range"));
    }
    b <<= 4;
    b &= 0b11110000;
    a &= 0b00001111;
    return Ok(a | b);
}
fn decompress_nibble(input: u8) -> (u8, u8) {
    let b = input & 0b11110000;
    let b = b >> 4;
    let a = input & 0b00001111;
    (a, b)
}
pub fn decompress_vec(input: Vec<u8>) -> Option<Vec<u8>> {
    let mut output = vec![];
    if input.len() <= 0 {
        return None;
    }
    for i in 0..input.len() {
        let decompressed = decompress_nibble(input[i]);
        output.push(decompressed.0);
        output.push(decompressed.1);
    }
    return Some(output);
}
pub fn compress_to_nibble(input: Vec<u8>) -> anyhow::Result<Vec<u8>> {
    let mut output = vec![];
    if input.len() <= 0 {
        return Err(anyhow::anyhow!("Bad length"));
    }
    let mut i = 0;
    while i < input.len() - 1 {
        output.push(make_nibble_byte(input[i], input[i + 1])?);
        i += 2;
    }
    if input.len() % 2 == 1 {
        output.push(*input.last().unwrap())
    }
    //output.remove(output.len() - 1);
    return Ok(output);
}
fn deserialize_i64_37<'de, D>(deserializer: D) -> Result<[i64; 37], D::Error>
where
    D: Deserializer<'de>,
{
    struct MaxVisitor(PhantomData<fn() -> [i64; 37]>);

    impl<'de> Visitor<'de> for MaxVisitor {
        type Value = [i64; 37];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of 37 numbers")
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<[i64; 37], S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut res = [0; 37];
            let mut index: usize = 0;

            while let Some(value) = seq.next_element()? {
                res[index] = value;
                index += 1;
            }

            if index != 37 {
                return Err(de::Error::custom(format!(
                    "expected 37 numbers, found {}",
                    index
                )));
            }

            Ok(res)
        }
    }

    // Create the visitor and ask the deserializer to drive it. The
    // deserializer will call visitor.visit_seq() if a seq is present in
    // the input data.
    let visitor = MaxVisitor(PhantomData);
    deserializer.deserialize_seq(visitor)
}
