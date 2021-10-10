use crate::game::ChunkCoords;
use crate::network::packet::ServerPacket;
use crate::configuration::CONFIGURATION;
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
        log::debug!("Packet {:?}", packet);
        player.send(packet).ok()?;
        //log::debug!("G");
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
    pub fn calculate_skylight(&mut self, time: i64) {
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
    }
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
            *section = Some(ChunkSection::new(self.x, self.z, idx.2 as i8));
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
            *section = Some(ChunkSection::new(self.x, self.z, sec_num as i8));
        }
        let section = section.as_mut().unwrap();
        for x in 0..16 {
            for z in 0..16 {
                **section
                    .get_block(ChunkSection::pos_to_index(x, y, z))
                    .as_mut()
                    .unwrap() = block;
            }
        }
        Ok(())
    }
}
pub struct World {
    pub chunks: HashMap<ChunkCoords, Chunk>,
    pub generator: Box<dyn ChunkGenerator>,
}
use std::time::*;
impl World {
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
                if !self.check_chunk_exists(coords)
                /*  && !(x == 0 && z == 0) */
                {
                    self.chunks.insert(
                        coords,
                        self.generator.gen_chunk(ChunkCoords { x: coords.x, z :coords.z }), // crate::chunks::Chunk::epic_generate(coords.x, coords.z) 
                        /*                 Chunk { 
                                                                                     x: idx.0,
                                                                                     z: idx.1,
                                                                                     data: [None, None, None, None, None, None, None, None],
                                                                                 }, */
                    );
                    count += 1;
                }
            }
        }
        log::info!("[World] Done! ({}s)", Instant::now().duration_since(start_time).as_secs());
    }
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
                log::debug!("Sending section");
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
                    log::debug!("Sending section");
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
        let idx = Self::pos_to_index(x, y, z)?;
        let chunk = self
            .chunks
            .get(&ChunkCoords { x: idx.0, z: idx.1 })
            .is_some();
        if !chunk {
            log::info!("Generating");
            self.chunks.insert(
                ChunkCoords { x: idx.0, z: idx.1 },
                self.generator.gen_chunk(ChunkCoords { x: idx.0, z: idx.1 }),
            );
        }
        drop(chunk);
        let chunk = self.chunks.get_mut(&ChunkCoords { x: idx.0, z: idx.1 })?;
        let section = chunk.data.get_mut(idx.2 as usize)?;
        if section.is_none() {
            *section = Some(ChunkSection::new(idx.0, idx.1, idx.2 as i8));
        }
        let section = section.as_mut().unwrap();
        section.get_block(ChunkSection::pos_to_index(
            x.rem_euclid(16),
            y.rem_euclid(16),
            z.rem_euclid(16),
        ))
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
    pub fn new(generator: Box<dyn ChunkGenerator>) -> Self {
        let mut chunks = HashMap::new();
        let coords = ChunkCoords { x: 0, z: 0 };
        chunks.insert(coords, generator.gen_chunk(coords));
        Self {
            chunks: chunks,
            generator,
        }
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
    if input.len() <= 0 {
        return None;
    }
    for i in 0..input.len() - 1 {
        output.push(make_nibble_byte(input[i], input[i + 1])?);
    }
    return Some(output);
}

pub trait ChunkGenerator {
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk;
}
use worldgen::constraint;
use worldgen::noise::perlin::PerlinNoise;
use worldgen::noisemap::ScaledNoiseMap;
use worldgen::noisemap::{NoiseMap, NoiseMapGenerator, NoiseMapGeneratorBase, Seed, Size, Step};
use worldgen::world::tile::{Constraint, ConstraintType};
use worldgen::world::{Tile, World as NGWorld};
pub struct FunnyChunkGenerator {
    noise: ScaledNoiseMap<NoiseMap<PerlinNoise>>,
}
impl FunnyChunkGenerator {
    pub fn new() -> Self {
        let noise = PerlinNoise::new();
        let nm = NoiseMap::new(noise)
            .set(Size::of(16, 16))
            .set(Seed::of("Hello?"))
            .set(Step::of(-0.02, 0.02));
        let nm = nm * 10;
        Self { noise: nm }
    }
}
impl ChunkGenerator for FunnyChunkGenerator {
    fn gen_chunk(&self, coords: ChunkCoords) -> Chunk {
        if CONFIGURATION.logging.chunk_gen {
            log::info!("Generating chunk at ({}, {})", coords.x, coords.z);
        }
        //log::info!("coords: {:?}", coords);
        let mut blocks = Vec::new();
        let mut chunk = Chunk {
            x: coords.x,
            z: coords.z,
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
        };

        /*         let nm2 = NoiseMap::new(noise)
            .set(Seed::of("Hello!"))
            .set(Step::of(0.05, 0.05));
        let nm = Box::new(nm1 + nm2 * 3);
        let world = World::new()
            .set(Size::of(64, 64))

            // Grass
            .add(Tile::new(Block { b_type: 0, b_metadata: 0, b_light: 0, b_skylight: 0 })
                .when(constraint!(nm.clone(), < -0.1)))
            // Air
            .add(Tile::new(Block { b_type: 3, b_metadata: 0, b_light: 0, b_skylight: 0 }));
        let mut blocksarray = Vec::new();
        let data = world.generate(coords.x as i64, coords.z as i64);
        for row in data.iter() {
            for val in row.iter() {
                for c in val.iter() {
                    blocksarray.push(c);
                }
            }
        } */
        let noise = self.noise.generate_chunk(-(coords.z as i64), -(coords.x as i64));
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
        //log::info!("Generated chunk: {:?}", chunk.data.len());
        /*         for i in 0..4 {
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
        use rand::prelude::*;
        let rng = rand::thread_rng().gen_range(0..3);
        for i in 0..rng {
            chunk
                .fill_layer(
                    i + 4,
                    Block {
                        b_type: 3,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    },
                )
                .unwrap();
        } */
        chunk
    }
}
pub struct FlatChunkGenerator {}
impl ChunkGenerator for FlatChunkGenerator {
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
            x: coords.x,
            z: coords.z,
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
