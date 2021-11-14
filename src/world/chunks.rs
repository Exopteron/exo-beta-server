use crate::configuration::CONFIGURATION;
use crate::ecs::entities::player::NetworkManager;
//use crate::game::items::ItemRegistry;
use crate::game::ChunkCoords;
use crate::game::RefContainer;
use crate::network::packet::ServerPacket;
use flume::{Receiver, Sender};
use libdeflater::CompressionLvl;
use nbt::decode::read_compound_tag;
use nbt::encode::write_compound_tag;
use nbt::CompoundTag;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::cell::RefMut;
use std::collections::HashMap;
use std::io::Write;
use std::sync::Arc;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Block {
    pub b_type: u8,
    pub b_metadata: u8,
    pub b_light: u8,
    pub b_skylight: u8,
}
impl std::default::Default for Block {
    fn default() -> Self {
        Self {
            b_type: 0,
            b_metadata: 0,
            b_light: 0,
            b_skylight: 0,
        }
    }
}
impl Block {
    pub fn set_type(&mut self, block_type: u8) {
        self.b_type = block_type;
    }
    pub fn get_type(&self) -> u8 {
        self.b_type
    }

    pub fn set_meta(&mut self, v: u8) {
        self.b_metadata = v;
    }
    pub fn get_meta(&self) -> u8 {
        self.b_metadata
    }

    pub fn set_light(&mut self, v: u8) {
        self.b_light = v;
    }
    pub fn get_light(&self) -> u8 {
        self.b_light
    }

    pub fn set_skylight(&mut self, v: u8) {
        self.b_skylight = v;
    }
    pub fn get_skylight(&self) -> u8 {
        self.b_skylight
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
    pub fn get_data(&mut self) -> &mut Vec<Block> {
        &mut self.data
    }
}
impl ChunkSection {
    pub fn pos_to_index(mut x: i32, mut y: i32, mut z: i32) -> usize {
        /*         x %= 16;
        y %= 16;
        z %= 16; */
        (y + (z * 16) + (x * 16 * 16)) as usize
    }
    pub fn get_block(&mut self, idx: usize) -> Option<&mut Block> {
        //log::info!("Here! {}", idx);
        let len = self.data.len();
        let possible = self.data.get(idx);
        if possible.is_some() {
            return self.data.get_mut(idx);
        } else {
            for i in 0..idx + 5 {
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
    pub fn to_packets_section_new(&self, n: &mut NetworkManager) -> Option<()> {
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
        //metadata.reverse();
        blockdata.append(&mut compress_to_nibble(metadata)?);
        blockdata.append(&mut compress_to_nibble(blocklight)?);
        blockdata.append(&mut compress_to_nibble(skylight)?);
        let start = Instant::now();
        let mut data = Vec::with_capacity(blockdata.len());
        let mut compressor =
            flate2::write::ZlibEncoder::new(&mut data, flate2::Compression::fast());
        compressor.write_all(&blockdata).ok()?;
        compressor.finish().ok()?;
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
        log::debug!("Packet {:?}", packet);
        n.write(packet);
        //log::debug!("G");
        Some(())
    }
    pub fn to_packets_section_raw(
        &self,
        player: Sender<ServerPacket>,
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
        //metadata.reverse();
        blockdata.append(&mut compress_to_nibble(metadata)?);
        blockdata.append(&mut compress_to_nibble(blocklight)?);
        blockdata.append(&mut compress_to_nibble(skylight)?);
        let start = Instant::now();
        let mut data = Vec::with_capacity(blockdata.len());
        let mut compressor =
            flate2::write::ZlibEncoder::new(&mut data, flate2::Compression::fast());
        compressor.write_all(&blockdata).ok()?;
        compressor.finish().ok()?;
        //log::info!("Compression took {}ns.", start.elapsed().as_nanos());
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
        log::debug!("Packet {:?}", packet);
        player.send(packet).ok()?;
        //log::debug!("G");
        return Some(());
    }
    pub async fn to_packets_section_async(&self, player: Sender<ServerPacket>) -> Option<()> {
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
        for _ in 0..chunk.data.len() + 0 {
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
        log::debug!("Packet {:?}", packet);
        player.send_async(packet).await.ok()?;
        //log::debug!("G");
        return Some(());
    }
}
#[derive(Clone)]
pub struct Chunk {
    pub pos: ChunkCoords,
    pub data: [Option<ChunkSection>; 8],
    pub heightmap: [[i8; 16]; 16],
}
impl Chunk {
    pub fn plain(x: i32, z: i32) -> Self {
        let mut v = Self {
            pos: ChunkCoords { x, z },
            data: [None, None, None, None, None, None, None, None],
            heightmap: [[0; 16]; 16],
        };
        v.calculate_heightmap();
        v
    }
    pub fn to_packets(&self, n: &mut NetworkManager) {
        for section in self.data.iter() {
            if let Some(s) = section {
                s.to_packets_section_new(n);
            }
        }
    }
    pub fn to_packets_async(&mut self, player: Sender<ServerPacket>) {
        use rayon::prelude::*;
        self.data.par_iter().for_each(|section| {
            if let Some(section) = section {
                section.to_packets_section_raw(player.clone(), &mut Vec::new());
            }
        });
        /*         for section in &self.data {
            tokio::task::yield_now().await;
        } */
    }
    pub fn calculate_skylight(&mut self, time: i64) -> anyhow::Result<()> {
        //log::info!("Calculating skylight for {}, {}", self.x, self.z);
        for x in 0..16 {
            for z in 0..16 {
                let y = self.heightmap[x as usize][z as usize];
                //for y in y..127 {
                self.get_block(x, y as i32, z)
                    .ok_or(anyhow::anyhow!("Block does not exist!"))?
                    .b_skylight = 15;
                //}
            }
        }
        Ok(())
    }
    pub fn calculate_heightmap(&mut self) -> anyhow::Result<()> {
        //log::info!("Calculating heightmap for {}, {}", self.x, self.z);
        for x in 0..16 {
            for z in 0..16 {
                'y_loop: for y in (0..127).rev() {
                    if let Some(block) = self.get_block(x, y, z) {
                        if block.b_type != 0 {
                            self.heightmap[x as usize][z as usize] = y as i8;
                            break 'y_loop;
                        }
                    }
                }
            }
        }
        Ok(())
    }
    pub fn to_file(&self, path: &str) -> anyhow::Result<()> {
        use nbt::encode::write_gzip_compound_tag;
        use nbt::CompoundTag;
        let mut root = CompoundTag::new();
        let mut tags = Vec::new();
        //let start = Instant::now();
        for section in self.data.iter() {
            if let Some(section) = section {
                let mut tag = CompoundTag::new();
                let mut blockdatadata = Vec::new();
                let mut metadatadata = Vec::new();
                let mut blocklightdata = Vec::new();
                let mut skylightdata = Vec::new();
                let mut metadata = Vec::with_capacity(section.data.len());
                let mut blocklight = Vec::with_capacity(section.data.len());
                let mut skylight = Vec::with_capacity(section.data.len());
                for byte in &section.data {
                    blockdatadata.push(byte.b_type as i8);
                    metadata.push(byte.b_metadata);
                    blocklight.push(byte.b_light);
                    skylight.push(byte.b_skylight);
                }
                metadatadata.append(&mut compress_to_nibble(metadata).unwrap());
                blocklightdata.append(&mut compress_to_nibble(blocklight).unwrap());
                skylightdata.append(&mut compress_to_nibble(skylight).unwrap());
                let mut newvec = Vec::new();
                for byte in metadatadata {
                    newvec.push(byte as i8);
                }
                tag.insert_i8_vec("metadata", newvec);
                tag.insert_i8_vec("blox", blockdatadata);
                tag.insert_i32("chunkx", self.pos.x);
                tag.insert_i32("chunkz", self.pos.z);
                tag.insert_i8("section", section.section);
                tags.push(tag);
            }
        }
        root.insert_compound_tag_vec("sections", tags);
        let mut file = std::fs::File::create(path)?;
        write_gzip_compound_tag(&mut file, &root)?;
        //log::info!("It took {}ms.", start.elapsed().as_millis());
        Ok(())
    }
    pub fn from_file(path: &str) -> Option<Self> {
        use nbt::decode::read_gzip_compound_tag;
        use nbt::CompoundTag;
        let mut file = std::fs::File::open(path).unwrap();
        let root = read_gzip_compound_tag(&mut file).unwrap();
        let sections = root.get_compound_tag_vec("sections").unwrap();
        let mut x = 0;
        let mut z = 0;
        let mut chunksections = Vec::new();
        for section in sections {
            let blox = section.get_i8_vec("blox").unwrap();
            let metadata = section.get_i8_vec("metadata").unwrap();
            let chunk_x = section.get_i32("chunkx").unwrap();
            let chunk_z = section.get_i32("chunkz").unwrap();
            let section = section.get_i8("section").unwrap();
            let mut newvec = Vec::with_capacity(metadata.len());
            for byte in metadata {
                newvec.push(*byte as u8);
            }
            let metadata = decompress_vec(newvec).unwrap();
            let mut data = Vec::new();
            let mut iter = 0;
            for block in blox {
                data.push(Block {
                    b_type: *block as u8,
                    b_metadata: metadata[iter],
                    b_light: 0,
                    b_skylight: 0,
                });
                iter += 1;
            }
            x = chunk_x;
            z = chunk_z;
            let section = ChunkSection {
                section,
                x: chunk_x,
                z: chunk_z,
                data,
            };
            chunksections.push(section);
        }
        let mut chunk = Chunk {
            pos: ChunkCoords { x, z },
            data: [
                Some(chunksections[0].clone()),
                Some(chunksections[1].clone()),
                Some(chunksections[2].clone()),
                Some(chunksections[3].clone()),
                Some(chunksections[4].clone()),
                Some(chunksections[5].clone()),
                Some(chunksections[6].clone()),
                Some(chunksections[7].clone()),
            ],
            heightmap: [[0; 16]; 16],
        };
        chunk.calculate_heightmap().ok()?;
        //chunk.calculate_skylight(GAME_GLOBAL.get_time()).ok()?;
        Some(chunk)
    }
    /*     pub fn calculate_skylight(&mut self, time: i64) {
        for x in 0..16 {
            for z in 0..16 {
                for y in (0..127).rev() {
                    let block = self.get_block(x, y, z).unwrap();
                    if block.get_type() == 0 {
                        continue;
                    } else {
                        block.b_skylight = 15;
                        //block.b_light = 15;
                    }
                }
            }
        }
    } */
    pub fn get_block(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Block> {
        let idx = World::pos_to_index(x, y, z)?;
        if x > 15 {
            //log::info!("Coords {} {} {}", x, y, z);
            //panic!("g");
        }
        if idx.2 > 1 {
            //log::info!("Getting section {}", idx.2);
        }
        let section = self.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(self.pos.x, self.pos.z, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        section.get_block(ChunkSection::pos_to_index(
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16),
        ))
    }
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
            pos: ChunkCoords { x, z },
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
            heightmap: [[0; 16]; 16],
        };
        chunk
    }
    pub fn fill_layer(&mut self, y: i32, block: Block) -> anyhow::Result<()> {
        let section = y / 16;
        if section < 0 {
            return Err(anyhow::anyhow!("Section below zero!"));
        }
        let sec_num = section;
        let section = self
            .data
            .get_mut(section as usize)
            .ok_or(anyhow::anyhow!("Can't get section!"))?;
        if section.is_none() {
            *section = Some(ChunkSection::new(self.pos.x, self.pos.z, sec_num as i8));
        }
        let section = section.as_mut().unwrap();
        for x in 0..16 {
            for z in 0..16 {
                if x == 0 && z == 0 {
                    log::debug!("Doing {} {} {}", x, y, z);
                }
                **section
                    .get_block(ChunkSection::pos_to_index(x, y, z - section.section as i32))
                    .as_mut()
                    .unwrap() = block;
            }
        }
        self.calculate_heightmap()?;
        //self.calculate_skylight(GAME_GLOBAL.get_time())?;
        Ok(())
    }
    pub fn fill_layer_air(&mut self, y: i32, block: Block) -> anyhow::Result<()> {
        let section = y / 16;
        if section < 0 {
            return Err(anyhow::anyhow!("Section below zero!"));
        }
        let sec_num = section;
        let section = self
            .data
            .get_mut(section as usize)
            .ok_or(anyhow::anyhow!("Can't get section!"))?;
        if section.is_none() {
            *section = Some(ChunkSection::new(self.pos.x, self.pos.z, sec_num as i8));
        }
        let section = section.as_mut().unwrap();
        for x in 0..16 {
            for z in 0..16 {
                let mut w_block =
                    section.get_block(ChunkSection::pos_to_index(x, y, z - section.section as i32));
                let w_block = w_block.as_mut().unwrap();
                if w_block.b_type == 0 {
                    if x == 0 && z == 0 {
                        log::debug!("Doing {} {} {}", x, y, z);
                    }
                    **w_block = block;
                } else {
                    if x == 0 && z == 0 {
                        log::debug!("Not doing {} {} {} it is type {}", x, y, z, w_block.b_type);
                    }
                }
            }
        }
        self.calculate_heightmap()?;
        //self.calculate_skylight(GAME_GLOBAL.get_time())?;
        Ok(())
    }
}
use crate::game::{BlockPosition, Position};
use std::collections::VecDeque;
pub struct World {
    pub chunks: HashMap<ChunkCoords, Chunk>,
    pub generator: Arc<Box<dyn WorldGenerator>>,
    pub spawn_position: Position,
    pub block_updates: VecDeque<(BlockPosition, Block)>,
    pub mcr_helper: Option<MCRegionLoader>,
}
use std::time::*;
impl World {
    pub fn init_chunk(&mut self, coords: &ChunkCoords) {
        let chunk = self.check_chunk_exists(coords);
        if !chunk {
            let coords = ChunkCoords {
                x: coords.x,
                z: coords.z,
            };
            if let Some(mcr_helper) = &mut self.mcr_helper {
                if let Some(c) = mcr_helper.get_chunk(&coords) {
                    self.chunks.insert(coords, c);
                }
            } else {
                log::info!("Generating {:?}", coords);
                self.chunks.insert(coords, Chunk::plain(coords.x, coords.z));
            }
        }
        /*         let _ = self
        .chunks
        .get_mut(coords)
        .unwrap()
        .calculate_skylight(GAME_GLOBAL.get_time()); */
    }
    pub fn to_file(&mut self, file: &str) -> anyhow::Result<()> {
        let mut mcr_helper = std::mem::replace(&mut self.mcr_helper, None);
        if let Some(mcr_helper) = mcr_helper.as_mut() {
            mcr_helper.save_all(self)?;
        }
        self.mcr_helper = mcr_helper;
        let mut file = std::fs::File::create(&format!(
            "{}/level.dat",
            self.mcr_helper.as_ref().unwrap().world_dir
        ))?;
        let mut root_tag = CompoundTag::new();
        let mut tag = CompoundTag::new();
        tag.insert_i32("SpawnX", self.spawn_position.x as i32);
        tag.insert_i32("SpawnY", self.spawn_position.x as i32);
        tag.insert_i32("SpawnZ", self.spawn_position.x as i32);
        write_compound_tag(&mut file, &root_tag)?;
        return Ok(());
        /*         let start = Instant::now();
        log::info!("Saving world to \"{}\"", file);
        use std::fs;
        fs::create_dir_all(file).unwrap();
        use rayon::prelude::*;
        self.chunks.par_iter().for_each(|x| {
            let (coords, chunk) = x;
            let path = format!("{}/{}-{}.nbt", file, coords.x, coords.z);
            let chunk = chunk.clone();
            if let Err(e) = chunk.to_file(&path) {
                log::error!("Error saving chunk: {:?}", e);
            }
        });
        /*         for (coords, chunk) in self.chunks.iter() {
            let path = format!("{}/{}-{}.nbt", file, coords.x, coords.z);
            let chunk = chunk.clone();
            if let Err(e) = chunk.to_file(&path) {
                log::error!("Error saving chunk: {:?}", e);
            }
        } */
        use nbt::encode::write_compound_tag;
        use nbt::CompoundTag;
        let mut root = CompoundTag::new();
        root.insert_i64("seed", self.generator.get_seed() as i64);
        root.insert_str("generator", self.generator.get_name());
        let mut file = std::fs::File::create(format!("{}/main", file)).unwrap();
        write_compound_tag(&mut file, &root).unwrap();
        log::info!("Done in {}ms.", start.elapsed().as_millis()); */
    }
    pub fn from_file_mcr(dir: &str) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(&format!("{}/level.dat", dir))?;
        let tag = read_compound_tag(&mut file)?;
        let tag = tag
            .get_compound_tag("Data")
            .or(Err(anyhow::anyhow!("Tag read error")))?
            .clone();
        let spawn_x = tag
            .get_i32("SpawnX")
            .or(Err(anyhow::anyhow!("Tag read error")))?;
        let spawn_y = tag
            .get_i32("SpawnY")
            .or(Err(anyhow::anyhow!("Tag read error")))?;
        let spawn_z = tag
            .get_i32("SpawnZ")
            .or(Err(anyhow::anyhow!("Tag read error")))?;
        let mut world = Self::new(
            Box::new(MountainWorldGenerator::new(
                tag.get_i64("RandomSeed")
                    .or(Err(anyhow::anyhow!("Tag read error")))? as u64,
            )),
            MCRegionLoader::new(dir)?,
        );
        world.spawn_position = Position::from_pos(spawn_x as f64, spawn_y as f64, spawn_z as f64);
        drop(tag);
        Ok(world)
    }
    pub fn from_file(file: &str) -> anyhow::Result<Self> {
        let start = Instant::now();
        log::info!("Loading world from {}/", file);
        let mut faxvec: Vec<std::path::PathBuf> = Vec::new();
        for element in std::path::Path::new(file).read_dir()? {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "nbt" {
                    faxvec.push(path);
                }
            }
        }
        let mut chunks = HashMap::new();
        use std::sync::mpsc::sync_channel;
        let (tx, rx) = sync_channel(1000000);
        use rayon::prelude::*;
        faxvec.into_par_iter().for_each(move |path| {
            let tx = tx.clone();
            let insert = Chunk::from_file(path.to_str().unwrap())
                .ok_or(anyhow::anyhow!("cant make chunk"))
                .unwrap();
            //log::info!("Loading chunk {}, {}", insert.x, insert.z);
            tx.clone().send((insert.pos, insert)).unwrap();
        });
        for chunk in rx.iter() {
            chunks.insert(chunk.0, chunk.1);
        }
        /*         for path in faxvec {
            let insert = Chunk::from_file(path.to_str().unwrap())
                .ok_or(anyhow::anyhow!("cant make chunk"))?;
            log::info!("Loading chunk {}, {}", insert.x, insert.z);
            chunks.insert(
                ChunkCoords {
                    x: insert.x,
                    z: insert.z,
                },
                insert,
            );
        } */
        use nbt::decode::read_compound_tag;
        use nbt::CompoundTag;
        let mut file = std::fs::File::open(format!("{}/main", file)).unwrap();
        let root = read_compound_tag(&mut file).unwrap();
        let generator: Box<dyn WorldGenerator> = match root.get_str("generator").unwrap() {
            /*             "FlatChunkGenerator" => Box::new(FlatChunkGenerator {}),
            "FunnyChunkGenerator" => Box::new(FunnyChunkGenerator::new(
                root.get_i64("seed").unwrap() as u64,
                FunnyChunkPreset::REGULAR,
            )), */
            "MountainChunkGenerator" => Box::new(MountainWorldGenerator::new(
                root.get_i64("seed").unwrap() as u64,
            )),
            _ => Box::new(MountainWorldGenerator::new(0)),
        };
        log::info!("Done in {}s.", start.elapsed().as_secs());
        Ok(Self {
            chunks,
            generator: Arc::new(generator),
            spawn_position: Position::from_pos(3., 45., 8.),
            block_updates: VecDeque::new(),
            mcr_helper: Some(MCRegionLoader::new("")?),
        })
    }
    pub fn epic_test(&mut self) {
        let chunk = self.chunks.get_mut(&ChunkCoords { x: 0, z: 0 }).unwrap();
        chunk.to_file("bstestfile");
        *chunk = Chunk::from_file("bstestfile").unwrap();
    }
    pub fn generate_spawn_chunks(&mut self) {
        let start_time = Instant::now();
        let interval = Duration::from_secs(1);
        let mut last_update = Instant::now();
        let mut count = 0;
        log::info!("[World] Generating spawn chunks..");
        for x in -8..8 {
            for z in -8..8 {
                if last_update + interval < Instant::now() {
                    last_update = Instant::now();
                    let decimal = count as f64 / 256.0;
                    let percent = (decimal * 100.) as i32;
                    log::info!("[World] {}% complete.", percent);
                }
                let coords = ChunkCoords { x: x, z: z };
                //let mut coords = ChunkCoords::from_pos(&Position::from_pos(0, 0, 0));
                //coords.x += x;
                //coords.z += z;
                if !self.check_chunk_exists(&coords)
                /*  && !(x == 0 && z == 0) */
                {
                    self.init_chunk(&ChunkCoords {
                        x: coords.x,
                        z: coords.z,
                    });
                    count += 1;
                }
            }
        }
        log::info!(
            "[World] Done! ({}s)",
            Instant::now().duration_since(start_time).as_secs()
        );
    }
    pub fn check_chunk_exists(&self, coords: &ChunkCoords) -> bool {
        self.chunks.get(coords).is_some()
    }
    pub fn chunk_to_packets(
        &self,
        coords: ChunkCoords,
        player: Sender<ServerPacket>,
    ) -> anyhow::Result<()> {
        use rayon::prelude::*;
        let mut initialized = Vec::new();
        log::info!("Getting {:?}", coords);
        let chunk = self
            .chunks
            .get(&coords)
            .ok_or(anyhow::anyhow!("Tried to send a chunk that doesn't exist!"))?;
        for section in &chunk.data {
            if section.is_some() {
                log::debug!("Sending section");
                if !initialized.contains(&coords) {
                    player.send(ServerPacket::PreChunk {
                        x: chunk.pos.x,
                        z: chunk.pos.z,
                        mode: true,
                    })?;
                    initialized.push(coords);
                }
                section
                    .as_ref()
                    .unwrap()
                    .to_packets_section_raw(player.clone(), &mut Vec::new());
                //return Ok(());
            }
        }
        Ok(())
    }
    pub fn bad_to_packets(&self, player: Sender<ServerPacket>) -> anyhow::Result<()> {
        let mut initialized = Vec::new();
        for (coords, chunk) in &self.chunks {
            for section in &chunk.data {
                if section.is_some() {
                    log::debug!("Sending section");
                    if !initialized.contains(coords) {
                        player.send(ServerPacket::PreChunk {
                            x: chunk.pos.x,
                            z: chunk.pos.z,
                            mode: true,
                        })?;
                        initialized.push(*coords);
                    }
                    section
                        .as_ref()
                        .unwrap()
                        .to_packets_section_raw(player.clone(), &mut Vec::new());
                    //return Ok(());
                }
            }
        }
        Ok(())
    }
    /*     pub fn send_block_updates(&mut self, players: PlayerList) {
        loop {
            if let Some(update) = self.block_updates.pop_front() {
                //log::info!("Update: {:?}", update);
                for player in players.iter() {
                    let player = player.1;
                    if player
                        .get_loaded_chunks()
                        .contains(&update.0.to_chunk_coords())
                    {
                        if let Some(block) = self.get_block(update.0.x, update.0.y, update.0.z) {
                            if *block != update.1 {
                                let update = update.0;
                                //log::info!("Sending update to {}", player.get_username());
                                player.write_packet(ServerPacket::BlockChange {
                                    x: update.x,
                                    y: update.y as i8,
                                    z: update.z,
                                    block_type: block.b_type as i8,
                                    block_metadata: block.b_metadata as i8,
                                });
                                self.chunks
                                    .get_mut(&update.to_chunk_coords())
                                    .expect("Impossible")
                                    .calculate_heightmap();
                                self.chunks
                                    .get_mut(&update.to_chunk_coords())
                                    .expect("Impossible")
                                    .calculate_skylight(GAME_GLOBAL.get_time());
                            }
                        }
                    }
                }
            } else {
                self.block_updates = VecDeque::new();
                break;
            }
        }
    } */
    pub fn set_block(&mut self, pos: &BlockPosition, b: &Block) -> Option<()> {
        let idx = Self::pos_to_index(pos.x, pos.y, pos.z)?;
        self.init_chunk(&pos.to_chunk_coords());
        let chunk = self.chunks.get_mut(&pos.to_chunk_coords()).unwrap();
        let section = chunk.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        *section.get_block(ChunkSection::pos_to_index(
            pos.x.rem_euclid(16),
            pos.y.rem_euclid(16),
            pos.z.rem_euclid(16),
        ))? = b.clone();
        Some(())
    }
    pub fn get_block(&mut self, pos: &BlockPosition) -> Block {
        let idx = Self::pos_to_index(pos.x, pos.y, pos.z).unwrap();
        self.init_chunk(&pos.to_chunk_coords());
        let chunk = self.chunks.get_mut(&pos.to_chunk_coords()).unwrap();
        let section = chunk.data.get_mut(idx.2 as usize).unwrap();
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        section
            .get_block(ChunkSection::pos_to_index(
                pos.x.rem_euclid(16),
                pos.y.rem_euclid(16),
                pos.z.rem_euclid(16),
            ))
            .unwrap_or(&mut Block::default())
            .clone()
    }
    fn get_block_mut(&mut self, x: i32, y: i32, z: i32) -> Option<&mut Block> {
        let idx = Self::pos_to_index(x, y, z)?;
        let chunk = self
            .chunks
            .get(&ChunkCoords { x: idx.0, z: idx.1 })
            .is_some();
        if !chunk {
            self.init_chunk(&ChunkCoords { x: idx.0, z: idx.1 });
        }
        drop(chunk);
        let chunk = self.chunks.get_mut(&ChunkCoords { x: idx.0, z: idx.1 })?;
        let section = chunk.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        let block = section.get_block(ChunkSection::pos_to_index(
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16),
        ))?;
        self.block_updates
            .push_back((BlockPosition { x, y, z }, block.clone()));
        Some(block)
    }
    fn get_block_internal(&mut self, x: i32, y: i32, z: i32) -> Option<&Block> {
        let idx = Self::pos_to_index(x, y, z)?;
        let chunk = self
            .chunks
            .get(&ChunkCoords { x: idx.0, z: idx.1 })
            .is_some();
        if !chunk {
            if let Some(c) = self
                .mcr_helper
                .as_mut()
                .unwrap()
                .get_chunk(&ChunkCoords { x: idx.0, z: idx.1 })
            {
                self.chunks.insert(ChunkCoords { x: idx.0, z: idx.1 }, c);
            } else {
                // TODO a standardized thing for generating missing chunks, not repeating code
                //////////////////log::info!("Generating");
                self.chunks.insert(
                    ChunkCoords { x: idx.0, z: idx.1 },
                    self.generator.gen_chunk(ChunkCoords { x: idx.0, z: idx.1 }),
                );
                self.generator
                    .clone()
                    .gen_structures(self, ChunkCoords { x: idx.0, z: idx.1 });
            }
        }
        drop(chunk);
        let chunk = self.chunks.get_mut(&ChunkCoords { x: idx.0, z: idx.1 })?;
        let section = chunk.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        Some(section.get_block(ChunkSection::pos_to_index(
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16),
        ))?)
    }
    pub fn pos_to_index(x: i32, y: i32, z: i32) -> Option<(i32, i32, i32)> {
        //log::info!("X {} Y {} Z {}", x, y, z);
        let mut chunk_x = x >> 4;
        let mut chunk_z = z >> 4;
        let section = y / 16;
        //let section = ((((y * 1) as f64) / 16.) - 1.).max(0.).floor() as i32;
        if section < 0 {
            return None;
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
        Some((chunk_x, chunk_z, section))
    }
    pub fn new(generator: Box<dyn WorldGenerator>, mcr: MCRegionLoader) -> Self {
        let mut chunks = HashMap::new();
        //let coords = ChunkCoords { x: 0, z: 0 };
        //chunks.insert(coords, generator.gen_chunk(coords));
        let mut world = Self {
            chunks: chunks,
            generator: Arc::new(generator),
            spawn_position: Position::from_pos(3., 45., 8.),
            block_updates: VecDeque::new(),
            mcr_helper: Some(mcr),
        };
        //world.generator.clone().gen_structures(&mut world, coords);
        world
    }
    /*     pub fn epic_generate() -> Self {
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
    } */
}
impl std::default::Default for World {
    fn default() -> Self {
        let generator = MountainWorldGenerator::new(0);
        let mut chunks = HashMap::new();
        let mut world = Self {
            chunks: chunks,
            generator: Arc::new(Box::new(generator)),
            spawn_position: Position::from_pos(3., 45., 8.),
            block_updates: VecDeque::new(),
            mcr_helper: None,
        };
        world
    }
}
fn make_nibble_byte(mut a: u8, mut b: u8) -> Option<u8> {
    if a > 15 || b > 15 {
        return None;
    }
    b <<= 4;
    b &= 0b11110000;
    a &= 0b00001111;
    return Some(a | b);
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
pub fn compress_to_nibble(input: Vec<u8>) -> Option<Vec<u8>> {
    let mut output = vec![];
    if input.len() <= 0 {
        return None;
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
    return Some(output);
}
pub trait WorldGenerator {
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk;
    fn gen_structures(&self, world: &mut World, coords: ChunkCoords);
    fn get_seed(&self) -> u64 {
        0
    }
    fn get_name(&self) -> String;
}
pub trait StructureGenerator {
    fn gen_chunk(&self, world: &mut World, coords: ChunkCoords);
    fn get_seed(&self) -> u64 {
        0
    }
    fn get_name(&self) -> String;
}
pub trait ChunkGenerator {
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk;
    fn get_seed(&self) -> u64 {
        0
    }
    fn get_name(&self) -> String;
}
use rand::Rng;
use rand::SeedableRng;
use rand_xorshift::*;
use worldgen::constraint;
use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::ScaledNoiseMap;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, NoiseMapGeneratorBase, Seed, Size, Step};
use worldgen::world::tile::{Constraint, ConstraintType};
use worldgen::world::{Tile, World as NGWorld};

//use super::classic::FlatWorldGenerator;
use super::mcregion::MCRegionLoader;
pub struct MountainStructureGenerator {
    seed: u64,
}
impl MountainStructureGenerator {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }
}
pub struct MountainWorldGenerator {
    chunk_gen: MountainChunkGenerator,
    structure_gen: MountainStructureGenerator,
}
impl MountainWorldGenerator {
    pub fn new(seed: u64) -> Self {
        Self {
            chunk_gen: MountainChunkGenerator::new(seed),
            structure_gen: MountainStructureGenerator::new(seed),
        }
    }
}
/* pub fn sphere(center: BlockPosition, radius: i32) -> Vec<BlockPosition> {
    let mut vec = Vec::new();
    let mut x = center.x - radius;
    let mut y = center.y - radius;
    let mut z = center.z - radius;
    while x <= center.x + radius {
        while y <= center.y + radius {
            while z <= center.z + radius {
                if ((center.x - x) * (center.x - x)) + ((center.y - y) * (center.y - y)) + ((center.z - z) * (center.z - z)) <= (radius * radius) {
                    vec.push(BlockPosition { x, y, z });
                }
                z += 1;
            }
            y += 1;
        }
        x += 1;
    }
    vec
} */
impl WorldGenerator for MountainWorldGenerator {
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk {
        self.chunk_gen.gen_chunk(coords)
    }
    fn gen_structures(&self, world: &mut World, coords: ChunkCoords) {
        /*         use siphasher::sip::SipHasher13;
        use std::hash::Hasher;
        let mut hash = SipHasher13::new_with_keys(self.chunk_gen.seed, self.chunk_gen.seed);
        hash.write_i32(coords.x);
        hash.write_i32(coords.z);
        let hash = hash.finish();
        let mut rng = XorShiftRng::seed_from_u64(hash);
        let tree_x = rng.gen_range(0..16);
        let tree_z = rng.gen_range(0..16);
        let chunk = world.chunks.get_mut(&coords).unwrap();
        let mut last_y = 0;
        for y in (25..127).rev() {
            let block = chunk.get_block(tree_x, y, tree_z).unwrap();
            if block.b_type != 2 {
                continue;
            }
            for offset in 1..rng.gen_range(3..8) {
                let block = chunk.get_block(tree_x, y + offset, tree_z).unwrap();
                block.set_type(17);
                last_y = y + offset;
            }
        }
        drop(chunk);
        let blocks = BlockPosition {
            x: tree_x + (coords.x * 16),
            y: last_y,
            z: tree_z + (coords.z * 16),
        }
        .all_directions();
        for block in blocks {
            if let Some(block) = world.get_block_mut(block.x, block.y, block.z) {
                if ItemRegistry::global()
                    .get_item(block.b_type as i16)
                    .is_none()
                {
                    continue;
                }
                if !ItemRegistry::global()
                    .get_item(block.b_type as i16)
                    .unwrap()
                    .get_item()
                    .as_block()
                    .unwrap()
                    .is_solid()
                {
                    block.set_type(18);
                }
            }
        } */
    }
    fn get_name(&self) -> String {
        self.chunk_gen.get_name()
    }
    fn get_seed(&self) -> u64 {
        self.chunk_gen.get_seed()
    }
}
impl StructureGenerator for MountainStructureGenerator {
    fn gen_chunk(&self, world: &mut World, coords: ChunkCoords) {}
    fn get_name(&self) -> String {
        String::from("MountainStructureGenerator")
    }
}
pub struct MountainChunkGenerator {
    noise: ScaledNoiseMap<NoiseMap<PerlinNoise>>,
    seed: u64,
}
/*
        let noise = PerlinNoise::new();
        let nm = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of_value(seed))
            .set(Step::of(-0.005, 0.005));
        let nm = nm * 30;
        cool gen ( you didn't see this. )
*/
impl MountainChunkGenerator {
    pub fn new(seed: u64) -> Self {
        let noise = PerlinNoise::new();
        let nm = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of_value(seed))
            .set(Step::of(-0.02, 0.02));
        let nm = nm * 25;
        Self {
            noise: nm,
            seed: seed,
        }
    }
}
pub struct FunnyChunkGenerator {
    noise: ScaledNoiseMap<NoiseMap<PerlinNoise>>,
    seed: u64,
}
pub enum FunnyChunkPreset {
    REGULAR,
    MOUNTAIN,
}
impl FunnyChunkGenerator {
    pub fn new(seed: u64, preset: FunnyChunkPreset) -> Self {
        match preset {
            FunnyChunkPreset::MOUNTAIN => {
                let noise = PerlinNoise::new();
                let nm = NoiseMap::new(noise)
                    .set(Size::of(16, 16))
                    .set(Seed::of_value(seed))
                    .set(Step::of(-0.02, 0.02));
                let nm = nm * 25;
                Self {
                    noise: nm,
                    seed: seed,
                }
            }
            FunnyChunkPreset::REGULAR => {
                let noise = PerlinNoise::new();
                let nm = NoiseMap::new(noise)
                    .set(Size::of(16, 16))
                    .set(Seed::of_value(seed))
                    .set(Step::of(-0.02, 0.02));
                let nm = nm * 10;
                Self {
                    noise: nm,
                    seed: seed,
                }
            }
        }
    }
}
impl ChunkGenerator for MountainChunkGenerator {
    fn get_seed(&self) -> u64 {
        self.seed
    }
    fn get_name(&self) -> String {
        "MountainChunkGenerator".to_string()
    }
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk {
        static WATER_HEIGHT: i32 = 25;
        use siphasher::sip::SipHasher13;
        use std::hash::Hasher;
        let mut hash = SipHasher13::new_with_keys(self.seed, self.seed);
        hash.write_i32(coords.x);
        hash.write_i32(coords.z);
        let hash = hash.finish();
        let mut rng = XorShiftRng::seed_from_u64(hash);
        if CONFIGURATION.logging.chunk_gen {
            log::info!("Generating chunk at ({}, {})", coords.x, coords.z);
        }
        //log::info!("coords: {:?}", coords);
        let mut blocks = Vec::new();
        let mut chunk = Chunk {
            pos: ChunkCoords {
                x: coords.x,
                z: coords.z,
            },
            data: [
                Some(ChunkSection {
                    data: blocks,
                    x: coords.x,
                    z: coords.z,
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
            heightmap: [[0; 16]; 16],
        };
        chunk.calculate_heightmap().unwrap();
        let noise = self
            .noise
            .generate_chunk(-(coords.z as i64), -(coords.x as i64));
        let mut noisevec = Vec::new();
        /*         for value in noise[0].iter() {
            noisevec.push(*value);
        } */
        for row in noise {
            for value in row.into_iter() {
                noisevec.push(value);
            }
        }
        //log::info!("Noisevec length: {:?}", noisevec.len());
        for x in 0..16 {
            for z in 0..16 {
                if noisevec.len() <= 0 {
                    break;
                }
                let mut num = noisevec.pop().unwrap() as i32;
                num += 40;
                if num > 11 {
                    //continue;
                }
                if num < WATER_HEIGHT {
                    *chunk.get_block(x, num, z).unwrap() = Block {
                        b_type: 13,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    };
                } else {
                    *chunk.get_block(x, num, z).unwrap() = Block {
                        b_type: 2,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    };
                }
                for y in 0..num - 3 {
                    *chunk.get_block(x, y, z).unwrap() = Block {
                        b_type: 1,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    };
                }
                for y in num - 3..num {
                    *chunk.get_block(x, y, z).unwrap() = Block {
                        b_type: 3,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    };
                }
            }
        }
        for y in 0..WATER_HEIGHT {
            chunk
                .fill_layer_air(
                    y,
                    Block {
                        b_type: 9,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    },
                )
                .unwrap();
        }
        chunk
            .fill_layer(
                0,
                Block {
                    b_type: 7,
                    b_metadata: 0,
                    b_light: 0,
                    b_skylight: 0,
                },
            )
            .unwrap();
        for thing in chunk.data.iter() {
            //log::info!("IS? {}", thing.is_some());
        }
        chunk
    }
}
impl ChunkGenerator for FunnyChunkGenerator {
    fn get_seed(&self) -> u64 {
        self.seed
    }
    fn get_name(&self) -> String {
        "FunnyChunkGenerator".to_string()
    }
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk {
        use siphasher::sip::SipHasher13;
        use std::hash::Hasher;
        let mut hash = SipHasher13::new_with_keys(self.seed, self.seed);
        hash.write_i32(coords.x);
        hash.write_i32(coords.z);
        let hash = hash.finish();
        let mut rng = XorShiftRng::seed_from_u64(hash);
        if CONFIGURATION.logging.chunk_gen {
            log::info!("Generating chunk at ({}, {})", coords.x, coords.z);
        }
        //log::info!("coords: {:?}", coords);
        let mut blocks = Vec::new();
        let mut chunk = Chunk {
            pos: coords,
            data: [
                Some(ChunkSection {
                    data: blocks,
                    x: coords.x,
                    z: coords.z,
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
            heightmap: [[0; 16]; 16],
        };
        let noise = self
            .noise
            .generate_chunk(-(coords.z as i64), -(coords.x as i64));
        let mut noisevec = Vec::new();
        /*         for value in noise[0].iter() {
            noisevec.push(*value);
        } */
        for row in noise {
            for value in row.into_iter() {
                noisevec.push(value);
            }
        }
        //log::info!("Noisevec length: {:?}", noisevec.len());
        for x in 0..16 {
            for z in 0..16 {
                if noisevec.len() <= 0 {
                    break;
                }
                let mut num = noisevec.pop().unwrap() as i32;
                num += 40;
                if num > 11 {
                    //continue;
                }
                *chunk.get_block(x, num, z).unwrap() = Block {
                    b_type: 2,
                    b_metadata: 0,
                    b_light: 0,
                    b_skylight: 0,
                };
                for y in 0..num - 3 {
                    *chunk.get_block(x, y, z).unwrap() = Block {
                        b_type: 1,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    };
                }
                for y in num - 3..num {
                    *chunk.get_block(x, y, z).unwrap() = Block {
                        b_type: 3,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    };
                }
            }
        }
        let tree_x = rng.gen_range(0..16);
        let tree_z = rng.gen_range(0..16);
        for y in (0..127).rev() {
            let block = chunk.get_block(tree_x, y, tree_z).unwrap();
            if block.b_type != 2 {
                continue;
            }
            for offset in 1..rng.gen_range(3..8) {
                let block = chunk.get_block(tree_x, y + offset, tree_z).unwrap();
                block.set_type(17);
            }
        }
        chunk
            .fill_layer(
                0,
                Block {
                    b_type: 7,
                    b_metadata: 0,
                    b_light: 0,
                    b_skylight: 0,
                },
            )
            .unwrap();
        for thing in chunk.data.iter() {
            //log::info!("IS? {}", thing.is_some());
        }
        chunk
    }
}
pub struct FlatChunkGenerator {}
impl ChunkGenerator for FlatChunkGenerator {
    fn get_name(&self) -> String {
        "FlatChunkGenerator".to_string()
    }
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk {
        let blocks = Vec::new();
        /*         for _ in 0..4096 {
            blocks.push(Block {
                b_type: 3,
                b_metadata: 0,
                b_light: 0,
                b_skylight: 0,
            });
        } */
        let mut chunk = Chunk {
            pos: coords,
            data: [
                Some(ChunkSection {
                    data: blocks.clone(),
                    x: coords.x,
                    z: coords.z,
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
            heightmap: [[0; 16]; 16],
        };
        chunk
            .fill_layer(
                32,
                Block {
                    b_type: 2,
                    b_metadata: 0,
                    b_light: 0,
                    b_skylight: 0,
                },
            )
            .unwrap();
        for i in 1..32 {
            chunk
                .fill_layer(
                    i,
                    Block {
                        b_type: 3,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    },
                )
                .unwrap();
        }
        chunk
            .fill_layer(
                0,
                Block {
                    b_type: 7,
                    b_metadata: 0,
                    b_light: 0,
                    b_skylight: 0,
                },
            )
            .unwrap();
        chunk
    }
}
