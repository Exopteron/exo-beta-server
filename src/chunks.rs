use crate::game::ChunkCoords;
use crate::network::packet::ServerPacket;
use flume::{Receiver, Sender};
use std::collections::HashMap;
#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub b_type: u8,
    pub b_metadata: u8,
    b_light: u8,
    b_skylight: u8,
}
impl Block {
    pub fn set_type(&mut self, block_type: u8) {
        self.b_type = block_type;
    }
    pub fn get_type(&self) -> u8 {
        self.b_type
    }
}
#[derive(Clone)]
pub struct ChunkSection {
    data: Vec<Block>,
    x: i32,
    z: i32,
    section: i8,
}
impl ChunkSection {
    pub fn new(x: i32, z: i32, section: i8) -> Self {
        Self {
            data: Vec::new(),
            x,
            z,
            section,
        }
    }
}
impl ChunkSection {
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> usize {
        (y + (z * 16) + (x * 16 * 16)) as usize
    }
    pub fn get_block(&mut self, idx: usize) -> Option<&mut Block> {
        //log::info!("Here! {}", idx);
        let len = self.data.len();
        let possible = self.data.get(idx);
        if possible.is_some() {
            return self.data.get_mut(idx);
        } else {
            for i in 0..idx + 1 {
                if let None = self.data.get_mut(i) {
                    self.data.push(Block {
                        b_type: 0,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    });
                }
            }
            return self.data.get_mut(idx);
        }
        None
    }
    pub fn to_packets_section_raw(
        &self,
        player: &mut Sender<ServerPacket>,
        has_loaded_before: &mut Vec<ChunkCoords>,
    ) -> Option<()> {
        //let mut packets = vec![];
        let chunk = self;
        //let chunk = chunk?;
        //let coords = ChunkCoords { x: chunk.x, z: chunk.x };
        let mut size_x = 0;
        let mut size_y = 0;
        let mut size_z = 0;
        let mut z_counter = 16;
        let mut x_counter = 128;
        //log::info!("Len: {:?}", chunk.data.len());
        for i in 0..chunk.data.len() + 0 {
            if size_y < 16 {
                size_y += 1;
            } else if size_z < 16 {
                if z_counter >= 16 {
                    size_z += 1;
                    // log::info!("+1 now {}", size_z);
                    z_counter = 0;
                    continue;
                }
                z_counter += 1;
            } else if size_x < 16 {
                //log::info!("Done {}", i);
                if x_counter >= 128 {
                    size_x += 1;
                    //  log::info!("X+1 now {}", size_x);
                    x_counter = 0;
                    continue;
                }
                x_counter += 1;
                //size_x += 1;
            } else {
                break;
            }
        }
        /*         let size_y = 127;
        let rest = chunk.data.len() % size_y;
        let size_z = 16;
        let size_x = 16; */
        let mut blockdata =
            Vec::with_capacity(chunk.data.len() + (chunk.data.len() as f32 * 1.5) as usize);
        let mut metadata = Vec::with_capacity(chunk.data.len());
        let mut blocklight = Vec::with_capacity(chunk.data.len());
        let mut skylight = Vec::with_capacity(chunk.data.len());
        for byte in &chunk.data {
            blockdata.push(byte.b_type);
            metadata.push(byte.b_metadata);
            blocklight.push(byte.b_light);
            skylight.push(byte.b_skylight);
        }
        blockdata.append(&mut compress_to_nibble(metadata)?);
        blockdata.append(&mut compress_to_nibble(blocklight)?);
        blockdata.append(&mut compress_to_nibble(skylight)?);
        let data = deflate::deflate_bytes_zlib(&blockdata);
        /*         let size_x = size_x - 1;
        let size_y = size_y - 1;
        let size_z = size_z - 1; */
        let size_x = (size_x - 1).max(0);
        //log::info!("size_x: {}", size_x);
        let size_y = (size_y - 1).max(0);
        let size_z = (size_z - 1).max(0);
        let mut epicy = ((chunk.section * 1) as i16) * 16;
        if chunk.section > 16 {
            //epicy -= 1;
        }
        //log::info!("EPIC Y: {:?} ON SECTION {}", epicy, self.section);
        let packet = ServerPacket::MapChunk {
            x: chunk.x * 16,
            y: epicy,
            z: chunk.z * 16,
            size_x: size_x as u8,
            size_y: size_y as u8,
            size_z: size_z as u8,
            compressed_size: data.len() as i32,
            compressed_data: data,
        };
        log::info!("Packet {:?}", packet);
        player.send(packet).ok()?;
        log::debug!("G");
        return Some(());
    }
}
#[derive(Clone)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    pub data: [Option<ChunkSection>; 8],
}
impl Chunk {
    pub fn epic_generate(x: i32, z: i32) -> Self {
        let mut blocks = Vec::new();
        for _ in 0..4096 {
            blocks.push(Block {
                b_type: 1,
                b_metadata: 0,
                b_light: 0,
                b_skylight: 0,
            });
        }
        let chunk = Chunk {
            x: x,
            z: z,
            data: [
                Some(ChunkSection {
                    data: blocks,
                    x: x,
                    z: z,
                    section: 0,
                }),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            ],
        };
        chunk
    }
}
pub struct World {
    pub chunks: HashMap<ChunkCoords, Chunk>,
}
impl World {
    pub fn check_chunk_exists(&self, coords: ChunkCoords) -> bool {
        self.chunks.get(&coords).is_some()
    }
    pub fn chunk_to_packets(
        &self,
        coords: ChunkCoords,
        player: &mut Sender<ServerPacket>,
    ) -> anyhow::Result<()> {
        let mut initialized = Vec::new();
        let chunk = self.chunks.get(&coords).ok_or(anyhow::anyhow!("Balls"))?;
        for section in &chunk.data {
            if section.is_some() {
                log::info!("Sending section");
                if !initialized.contains(&coords) {
                    player.send(ServerPacket::PreChunk {
                        x: chunk.x,
                        z: chunk.z,
                        mode: true,
                    })?;
                    initialized.push(coords);
                }
                section
                    .as_ref()
                    .unwrap()
                    .to_packets_section_raw(player, &mut Vec::new());
                //return Ok(());
            }
        }
        Ok(())
    }
    pub fn bad_to_packets(&self, player: &mut Sender<ServerPacket>) -> anyhow::Result<()> {
        let mut initialized = Vec::new();
        for (coords, chunk) in &self.chunks {
            for section in &chunk.data {
                if section.is_some() {
                    log::info!("Sending section");
                    if !initialized.contains(coords) {
                        player.send(ServerPacket::PreChunk {
                            x: chunk.x,
                            z: chunk.z,
                            mode: true,
                        })?;
                        initialized.push(*coords);
                    }
                    section
                        .as_ref()
                        .unwrap()
                        .to_packets_section_raw(player, &mut Vec::new());
                    //return Ok(());
                }
            }
        }
        Ok(())
    }
    pub fn get_block(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Block> {
        //let y = y - 1;
        let idx = Self::pos_to_index(x, y, z);
        let chunk = self
            .chunks
            .get(&ChunkCoords { x: idx.0, z: idx.1 })
            .is_some();
        if !chunk {
            self.chunks.insert(
                ChunkCoords { x: idx.0, z: idx.1 },
                Chunk::epic_generate(idx.0, idx.1)
/*                 Chunk {
                    x: idx.0,
                    z: idx.1,
                    data: [None, None, None, None, None, None, None, None],
                }, */
            );
        }
        drop(chunk);
        let chunk = self.chunks.get_mut(&ChunkCoords { x: idx.0, z: idx.1 })?;
        let section = chunk.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
            /*             panic!("TODO fix this");
            return None; */
        }
        let section = section.as_mut().unwrap();
        /*         log::info!("Balls");
        log::info!(
            "X {} Y {} Z {}",
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16)
        ); */
        //section.data.get_mut(ChunkSection::pos_to_index(x % 16, y % 16, z % 16))
        section.get_block(ChunkSection::pos_to_index(
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16),
        ))
    }
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> (i32, i32, i32) {
        //log::info!("X {} Y {} Z {}", x, y, z);
        let mut chunk_x = x >> 4;
        let mut chunk_z = z >> 4;
        let section = y / 16;
        //let section = ((((y * 1) as f64) / 16.) - 1.).max(0.).floor() as i32;
        if section < 0 {
            panic!("Ahh!");
        }
        /*         if chunk_x < 0 {
            chunk_x = -1;
        }
        if chunk_z < 0 {
            chunk_z = -1;
        } */
        /*         log::info!(
            "Chunk_x: {} chunk_z: {} section: {}",
            chunk_x,
            chunk_z,
            section
        ); */
        (chunk_x, chunk_z, section)
    }
    pub fn epic_generate() -> Self {
        let mut chunks = HashMap::new();
        let mut blocks = Vec::new();
        for i in 0..4096 {
            blocks.push(Block {
                b_type: 1,
                b_metadata: 0,
                b_light: 0,
                b_skylight: 0,
            });
        }
        chunks.insert(
            ChunkCoords { x: 0, z: 0 },
            Chunk {
                x: 0,
                z: 0,
                data: [
                    Some(ChunkSection {
                        data: blocks,
                        x: 0,
                        z: 0,
                        section: 0,
                    }),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ],
            },
        );
        Self { chunks }
    }
}
fn make_nibble_byte(mut a: u8, mut b: u8) -> Option<u8> {
    if a > 15 || b > 15 {
        return None;
    }
    a <<= 4;
    a &= 0b11110000;
    b &= 0b00001111;
    return Some(a | b);
}
fn compress_to_nibble(input: Vec<u8>) -> Option<Vec<u8>> {
    let mut output = vec![];
    for i in 0..input.len() - 1 {
        output.push(make_nibble_byte(input[i], input[i + 1])?);
    }
    return Some(output);
}
