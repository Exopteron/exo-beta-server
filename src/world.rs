// Deprecated
use crate::game::ChunkCoords;
use crate::network::packet::ServerPacket;
use flume::{Sender, Receiver};
#[derive(Clone, Copy)]
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
pub struct Chunk {
    x: i32,
    z: i32,
    data: Vec<Block>,
}
pub struct World {
    chunks: Vec<Chunk>,
}
impl World {
    pub fn get_block(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Block> {
        let chunk_x = x / 16;
        let chunk_z = z / 16;
        for chunk in &mut self.chunks {
            if chunk.x == chunk_x && chunk.z == chunk_z {
                //*chunk.data.get_mut(0).unwrap() = Block { b_type: 2, b_metadata: 0, b_light: 0, b_skylight: 15};
                return chunk.data.get_mut(Self::pos_to_index(x, y, z));
            }
        }
        return None;
    }
    pub fn pos_to_index(mut x: i32, mut y: i32, mut z: i32) -> usize {
        x %= 16;
        y %= 128;
        z %= 16;
        if x < 0 {
            x = (-x % 16);
        }
        if z < 0 {
            z = (-z % 16);
        }
        y += 2;
        let mut offset = 1;
        for i in 0..z {
            offset += 2;
        }
        for i in 0..x {
            offset += 32;
        }
        //y -= 14 * 3;
        //z -= 2;
        ((y + z * 128 + x * 128 * 16) - offset).max(0) as usize
        //return (((y * 16 + z) * 16 + x) as usize) + 126;
        //(y * 16 * 128 + z * 16 + x) as usize
        //((y*16*127 + z*16 + x) as usize) + 1
        //((x + (z << 4) + (y << 8)) as usize) - 255
        //y -= 10;
        //(y + ((z * 127) + (x * (127 * 16)))) as usize
        //(y + z * 128 + x * 128 * 16) as usize
    }
    pub fn crappy_generate() -> Self {
        let mut world = Self { chunks: Vec::new() };
        let mut blocks = vec![];
        // blocks.push(Block { b_type: 2, b_metadata: 0, b_light: 0, b_skylight: 0});
        let mut air = vec![];
        air.push(Block {
            b_type: 0,
            b_metadata: 0,
            b_light: 0,
            b_skylight: 0,
        });
        let mut first = true;
        let mut doit = false;
        for _ in 0..256 {
            let height = 16;
            if !doit {
                blocks.append(&mut vec![
                    Block {
                        b_type: 1,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 15
                    };
                    height - 1
                ]);
            } else {
                for i in 0..10 {
                    blocks.push(Block {
                        b_type: 20,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    });
                }
                blocks.append(&mut vec![
                    Block {
                        b_type: 1,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 15
                    };
                    height - 11
                ]);
                doit = false;
            }
            if first {
                first = false;
                doit = true;
            }
            /*             blocks.push(Block { b_type: 2, b_metadata: 0, b_light: 0, b_skylight: 0}); */
            blocks.append(&mut vec![
                Block {
                    b_type: 0,
                    b_metadata: 0,
                    b_light: 0,
                    b_skylight: 15
                };
                127 - height
            ]);
        }
        //let b_type = blocks.get(Self::pos_to_index(3, 14, 10)).unwrap();
        //log::info!("Type: {:?}", b_type.b_type);
        for x in -2..2 {
            for z in -2..2 {
                if x == 0 && z == 0 {
                    world.chunks.push(Chunk {
                        x: x,
                        z: z,
                        data: blocks.clone(),
                    });
                    //world.chunks.push( Chunk { x: x + 1, z: z, data: blocks.clone()} );
                } else {
                    world.chunks.push(Chunk {
                        x: x,
                        z: z,
                        data: blocks.clone(),
                    });
                }
            }
        }
        return world;
    }
    pub fn check_chunk_exists(&self, coords: ChunkCoords) -> bool {
        for chunk in &self.chunks {
            if chunk.x == coords.x && chunk.z == coords.z {
                return true;
            }
        }
        return false;
    }
    pub fn to_packets_new(&self, player: &mut Sender<ServerPacket>, has_loaded_before: &mut Vec<ChunkCoords>) -> Option<()> {
        for index in 0..self.chunks.len() {
            self.to_packets_chunk_raw(index, player, has_loaded_before)?;
        }
        Some(())
    }
    pub fn to_packets_chunk_raw(&self, index: usize, player: &mut Sender<ServerPacket>, has_loaded_before: &mut Vec<ChunkCoords>) -> Option<()> {
        //let mut packets = vec![];
        let chunk = &self.chunks[index];
        //let chunk = chunk?;
        let coords = ChunkCoords { x: chunk.x, z: chunk.x };
        player.send(ServerPacket::PreChunk {
            x: chunk.x,
            z: chunk.z,
            mode: true,
        }).ok()?;
        let mut size_x = 0;
        let mut size_y = 0;
        let mut size_z = 0;
        for i in 0..chunk.data.len() {
            if size_y < 127 {
                size_y += 1;
            } else if size_z < 16 {
                size_z += 1;
            } else if size_x < 16 {
                size_x += 1;
            } else {
                break;
            }
        }
/*         let size_y = 127;
        let rest = chunk.data.len() % size_y;
        let size_z = 16;
        let size_x = 16; */
        let mut blockdata = Vec::with_capacity(chunk.data.len() + (chunk.data.len() as f32 * 1.5) as usize);
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
        let size_x = size_x - 1;
        let size_z = size_y - 1;
        let size_y = size_z - 1;
        let packet = ServerPacket::MapChunk {
            x: chunk.x * 16,
            y: 0,
            z: chunk.z * 16,
            size_x: size_x as u8,
            size_y: size_y as u8,
            size_z: size_z as u8,
            compressed_size: data.len() as i32,
            compressed_data: data,
        };
        player.send(packet).ok()?;
        log::debug!("G");
        return Some(());
    }
    pub fn to_packets_chunk(&self, coords: ChunkCoords, player: &mut Sender<ServerPacket>, has_loaded_before: &mut Vec<ChunkCoords>) -> Option<()> {
        //let mut packets = vec![];
        let mut chunk = None;
        for stored_chunk in &self.chunks {
            if stored_chunk.x == coords.x && stored_chunk.z == coords.z {
                chunk = Some(stored_chunk.clone());
            }
        }
        let chunk = chunk?;
        let coords = ChunkCoords { x: chunk.x, z: chunk.x };
        player.send(ServerPacket::PreChunk {
            x: chunk.x,
            z: chunk.z,
            mode: true,
        }).ok()?;
/*         let mut size_x = 0;
        let mut size_y = 0;
        let mut size_z = 0;
        for i in 0..chunk.data.len() {
            if size_y < 127 {
                size_y += 1;
            } else if size_z < 16 {
                size_z += 1;
            } else if size_x < 16 {
                size_x += 1;
            } else {
                break;
            }
        } */
        let size_y = 128;
        let rest = chunk.data.len() % size_y;
        let size_z = 16;
        let size_x = 16;
        let mut blockdata = vec![];
        let mut metadata = vec![];
        let mut blocklight = vec![];
        let mut skylight = vec![];
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
        let size_x = (size_x - 1).max(0);
        log::info!("size_x: {}", size_x);
        let size_z = (size_y - 1).max(0);
        let size_y = (size_z - 1).max(0);
        let packet = ServerPacket::MapChunk {
            x: chunk.x * 16,
            y: 0,
            z: chunk.z * 16,
            size_x: size_x as u8,
            size_y: size_y as u8,
            size_z: size_z as u8,
            compressed_size: data.len() as i32,
            compressed_data: data,
        };
        player.send(packet).ok()?;
        log::debug!("G");
        return Some(());
    }
    pub fn to_packets(&self) -> Option<Vec<ServerPacket>> {
        let mut packets = vec![];
        for chunk in &self.chunks {
            packets.push(ServerPacket::PreChunk {
                x: chunk.x,
                z: chunk.z,
                mode: true,
            });
            let mut size_x = 0;
            let mut size_y = 0;
            let mut size_z = 0;
            for i in 0..chunk.data.len() {
                if size_y < 127 {
                    size_y += 1;
                } else if size_z < 16 {
                    size_z += 1;
                } else if size_x < 16 {
                    size_x += 1;
                } else {
                    break;
                }
            }
            let mut blockdata = vec![];
            let mut metadata = vec![];
            let mut blocklight = vec![];
            let mut skylight = vec![];
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
            let size_x = size_x - 1;
            let size_z = size_y - 1;
            let size_y = size_z - 1;
            let packet = ServerPacket::MapChunk {
                x: chunk.x * 16,
                y: 0,
                z: chunk.z * 16,
                size_x,
                size_y,
                size_z,
                compressed_size: data.len() as i32,
                compressed_data: data,
            };
            packets.push(packet);
        }
        log::debug!("G");
        return Some(packets);
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
