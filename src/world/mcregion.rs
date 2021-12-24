use anvil_region::position::RegionChunkPosition;
use anvil_region::position::RegionPosition;
use anvil_region::provider::FolderRegionProvider;
use anvil_region::provider::RegionProvider;
use nbt::encode::write_zlib_compound_tag;
use nbt::CompoundTag;
use once_cell::sync::OnceCell;

use crate::game::BlockPosition;
use crate::game::ChunkCoords;
use crate::game::Position;
use crate::world;
use crate::world::chunks::*;
use crate::world::heightmap::HeightmapStore;
use anvil_region::*;
pub struct MCRegionLoader {
    pub world_dir: String,
    cheating: FolderRegionProvider,
}
static PATH: OnceCell<String> = OnceCell::new();
impl MCRegionLoader {
    pub fn new<'a>(path: &'a str) -> anyhow::Result<Self> {
        std::fs::create_dir_all(&format!("{}/region", path))?;
        //PATH.set(format!("{}/region", path.to_string())).unwrap();
        Ok(Self {
            world_dir: path.to_string(),
            cheating: FolderRegionProvider::new(&format!("{}/region", path.to_string())),
        })
    }
    pub fn save_chunk(
        &mut self,
        chunk: ChunkHandle,
        tile_entities: Vec<CompoundTag>,
    ) -> anyhow::Result<()> {
        let coords = chunk.read().pos;
        let mut region = self
            .cheating
            .get_region(RegionPosition::from_chunk_position(coords.x, coords.z))?;
        region
            .write_chunk(
                RegionChunkPosition::from_chunk_position(coords.x, coords.z),
                Self::chunk_to_nbt(chunk.clone(), tile_entities)?,
            )
            .or(Err(anyhow::anyhow!("Bad write")))?;
        Ok(())
    }
    pub fn set_chunk(
        &mut self,
        chunk: ChunkHandle,
        tile_entities: Vec<CompoundTag>,
    ) -> anyhow::Result<()> {
        let pos = chunk.read().pos;
        let mut region = self
            .cheating
            .get_region(RegionPosition::from_chunk_position(pos.x, pos.z))?;
        region
            .write_chunk(
                RegionChunkPosition::from_chunk_position(pos.x, pos.z),
                Self::chunk_to_nbt(chunk, tile_entities)?,
            )
            .or(Err(anyhow::anyhow!("Bad write")))?;
        Ok(())
    }
    pub fn chunk_to_nbt(
        chunk: ChunkHandle,
        tile_entities: Vec<CompoundTag>,
    ) -> anyhow::Result<CompoundTag> {
        let mut root_tag = CompoundTag::new();
        let mut level_tag = CompoundTag::new();
        level_tag.insert_compound_tag_vec("TileEntities", tile_entities);
        let mut chunk = match chunk.try_read() {
            Some(chunk) => chunk,
            None => {
                return Err(anyhow::anyhow!("Couldn't grab chunk"));
            }
        };
        let mut block_data = Vec::with_capacity(chunk.data.len());
        let mut metadata = Vec::with_capacity(chunk.data.len() / 2);
        let mut skylight = Vec::with_capacity(chunk.data.len() / 2);
        let mut blocklight = Vec::with_capacity(chunk.data.len() / 2);
        for x in 0..16 {
            for z in 0..16 {
                for y in 0..128 {
                    // TODO: write this stuff here
                    block_data.push(chunk.block_at(x, y, z).unwrap().b_type as i8);
                    metadata.push(chunk.block_at(x, y, z).unwrap().b_metadata);
                    skylight.push(chunk.block_at(x, y, z).unwrap().b_skylight);
                    blocklight.push(chunk.block_at(x, y, z).unwrap().b_light);
                }
            }
        }
        let v = crate::world::chunks::compress_to_nibble(skylight).unwrap_or(Vec::new());
        let v = vec_u8_into_i8(v);
        level_tag.insert_i8_vec("SkyLight", v);
        let v = crate::world::chunks::compress_to_nibble(blocklight).unwrap_or(Vec::new());
        let v = vec_u8_into_i8(v);
        level_tag.insert_i8_vec("BlockLight", v);
        //log::info!("Block len: {:?}", block_data.len());
        let metadata = crate::world::chunks::compress_to_nibble(metadata).unwrap_or(Vec::new());
        let metadata = vec_u8_into_i8(metadata);
        level_tag.insert_i8_vec("Data", metadata);
        level_tag.insert_i8_vec("Blocks", block_data);
        level_tag.insert_i32("xPos", chunk.pos.x);
        level_tag.insert_i32("zPos", chunk.pos.z);
        root_tag.insert_compound_tag("Level", level_tag);
        //let mut output = Vec::new();
        //write_zlib_compound_tag(&mut output, &root_tag)?;
        Ok(root_tag)
        //Err(anyhow::anyhow!(""))
    }
    pub fn get_chunk(&mut self, coords: &ChunkCoords) -> ChunkLoadResult {
        let region_pos = RegionPosition::from_chunk_position(coords.x, coords.z);
        let region_chunk_pos = RegionChunkPosition::from_chunk_position(coords.x, coords.z);
        let mut region = match self.cheating.get_region(region_pos) {
            Ok(c) => c,
            Err(_) => {
                return ChunkLoadResult::Missing(coords.clone());
            }
        };
        let chunk_tag = match region.read_chunk(region_chunk_pos) {
            Ok(c) => c,
            Err(_) => {
                return ChunkLoadResult::Missing(coords.clone());
            }
        };
        let level_tag = match chunk_tag.get_compound_tag("Level") {
            Ok(c) => c,
            Err(_) => {
                return ChunkLoadResult::Error(anyhow::anyhow!("Level tag read error"));
            }
        };
        return match Region::chunk_from_tag(level_tag) {
            Ok(c) => ChunkLoadResult::Loaded(LoadedChunk {
                chunk: c.0,
                pos: coords.clone(),
                tile_entity_data: c.1,
            }),
            Err(e) => ChunkLoadResult::Error(e),
        };
    }
}
#[derive(Clone)]
pub struct Region {
    pub chunks: Vec<Chunk>,
}
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryInto;
use std::fs::File;
use std::fs::OpenOptions;
use std::hash::Hash;
use std::io::Write;
use std::ops::Deref;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
/*
        let mut faxvec: Vec<std::path::PathBuf> = Vec::new();
        for element in std::path::Path::new(directory).read_dir()? {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "mcr" {
                    faxvec.push(path);
                }
            }
        }
        let mut chunks = Vec::new();
        for path in faxvec {

        }
*/
struct PresentChunk {
    offset: u32,
    sector_count: u8,
    timestamp: u32,
}
use super::chunk_lock::ChunkHandle;
use super::chunks::*;
use super::worker::ChunkLoadResult;
use super::worker::LoadedChunk;
use super::World;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
/* pub fn temp_from_dir(dir: &str) -> anyhow::Result<World> {
    let mut faxvec: Vec<std::path::PathBuf> = Vec::new();
    for element in std::path::Path::new(&format!("{}/region/", dir)).read_dir()? {
        let path = element.unwrap().path();
        if let Some(extension) = path.extension() {
            if extension == "mcr" {
                faxvec.push(path);
            }
        }
    }
    let mut regions = Vec::new();
    for path in faxvec {
        regions
            .push(Region::from_file(path.to_str().ok_or(anyhow::anyhow!(
                "Couldn't convert path to string."
            ))?)?);
    }
    Ok(temp_from_regions(regions))
}
pub fn temp_from_regions(regions: Vec<Region>) -> World {
    let mut world = World::new(Box::new(MountainWorldGenerator::new(0)), MCRegionLoader::new());
    world.spawn_position = Position::from_pos(0., 150., 0.);
    for region in regions {
        for chunk in region.chunks.into_iter() {
            world.chunks.insert(
                ChunkCoords {
                    x: chunk.x,
                    z: chunk.z,
                },
                chunk,
            );
        }
    }
    world
} */
impl Region {
    pub fn chunk_from_tag(tag: &CompoundTag) -> anyhow::Result<(Chunk, Vec<CompoundTag>)> {
        let tile_entities = match tag.get_compound_tag_vec("TileEntities") {
            Ok(t) => t.into_iter().cloned().collect(),
            Err(_) => Vec::new(),
        };
        let val = tag
            .get_i8_vec("Blocks")
            .or(Err(anyhow::anyhow!("Does not exist!")))?;
        let block_ids = vec_i8_into_u8(val.clone());
        if block_ids.len() == 0 {
            return Err(anyhow::anyhow!("0 length blocks"));
        }
        let val = tag
            .get_i8_vec("Data")
            .or(Err(anyhow::anyhow!("Does not exist!")))?;
        //log::info!("Got to here!");
        let block_metadata = vec_i8_into_u8(val.clone());

        //log::info!("Got to here!");
        use super::chunks::*;
        let metadata = super::chunks::decompress_vec(block_metadata);
        let mut skylight: Option<Vec<u8>> = None;
        if let Ok(val) = tag
        .get_i8_vec("SkyLight") {
            let block_skylight = vec_i8_into_u8(val.clone());
            skylight = super::chunks::decompress_vec(block_skylight);
        }
        let mut blocks = Vec::new();
        let mut i = 0;
        for block in block_ids {
            let meta = if let Some(ref meta) = metadata {
                meta[i]
            } else {
                0
            };
            let sky = if let Some(ref sky) = skylight {
                sky[i]
            } else {
                0
            };
            blocks.push(BlockState {
                b_type: block,
                b_metadata: meta,
                b_light: 0,
                b_skylight: sky,
            });
            i += 1;
        }
        let x_pos = tag
            .get_i32("xPos")
            .or(Err(anyhow::anyhow!("Does not exist!")))?;
        let z_pos = tag
            .get_i32("zPos")
            .or(Err(anyhow::anyhow!("Does not exist!")))?;
        //log::info!("Compression type: {}", comp_type);
        //log::info!("Pos: {} {}", x_pos, z_pos);
        let mut chunksections = Vec::new();
        for i in 0..8 {
            chunksections.push(ChunkSection::new(i));
        }
        for section in 0..8 {
            for x in 0..16 {
                for z in 0..16 {
                    for y in 0..16 {
                        let y = y + (section * 16);
                        //log::info!("Doing section {}, {} {} {}", section, x, y, z);
                        let section = chunksections.get_mut(section).unwrap();
                        section
                            .get_data()
                            .push(blocks[Self::pos_to_idx(x, y as i32, z)]);
                    }
                }
            }
        }
        let mut chunk = Chunk {
            pos: ChunkCoords { x: x_pos, z: z_pos },
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
            heightmaps: HeightmapStore::new(),
        };
        let heightmap = tag.get_i8_vec("HeightMap");
        match heightmap {
            Ok(heightmap) => {
                for x in 0..16 {
                    for z in 0..16 {
                        chunk.heightmaps.light_blocking.set_height(
                            x,
                            z,
                            heightmap[Self::index_hm_vec(x, z)] as usize,
                        );
                    }
                }
            }
            Err(_) => {
                //log::info!("Current thread: {:?}", std::thread::current().name());
                chunk
                    .heightmaps
                    .recalculate(Chunk::block_at_fn(&chunk.data));
            }
        }
        //chunk.calculate_full_skylight();
        return Ok((chunk, tile_entities));
    }
    fn index_hm_vec(x: usize, z: usize) -> usize {
        z << 4 | x
    }
    pub fn from_file(file: &str) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(file)?;
        let mut chunks = Vec::new();
        let mut present_chunks = Vec::new();
        file.seek(SeekFrom::Start(0))?;
        for i in 0..1024 {
            let mut offset = vec![0; 3];
            file.read_exact(&mut offset)?;
            offset.reverse();
            offset.push(0);
            offset.reverse();
            if offset.iter().sum::<u8>() > 0 {
                log::info!("Bytes: {:?}", offset);
            }
            let offset: u32 = u32::from_be_bytes(
                offset
                    .try_into()
                    .or_else(|_| Err(anyhow::anyhow!("couldn't convert?")))?,
            );
            let mut sector_count = [0; 1];
            file.read_exact(&mut sector_count)?;
            if sector_count[0] > 0 {
                log::info!("Count: {:?}", sector_count);
            }
            if i > 100 {
                //std::process::exit(0);
            }
            if offset != 0 && sector_count[0] != 0 {
                if offset > 0 {
                    log::info!("Adding {} {}", offset, sector_count[0]);
                }
                present_chunks.push(PresentChunk {
                    offset,
                    sector_count: sector_count[0],
                    timestamp: 0,
                });
            } else {
                //log::info!("Not!");
            }
        }
        //std::process::exit(0);
        for i in 0..present_chunks.len() {
            let mut integer = [0; 4];
            file.read_exact(&mut integer)?;
            if let Some(chunk) = present_chunks.get_mut(i) {
                chunk.timestamp = u32::from_be_bytes(integer);
            } else {
                return Err(anyhow::anyhow!("Chunk doesn't exist?"));
            }
        }
        file.seek(SeekFrom::Start(8192))?;
        for chunk in present_chunks {
            log::info!("Offset: {:?}", chunk.offset as u64 * 4096);
            file.seek(SeekFrom::Start(chunk.offset as u64 * 4096))?;
            log::info!("A");
            let mut integer = [0; 4];
            file.read_exact(&mut integer)?;
            log::info!("B");
            let length = u32::from_be_bytes(integer);
            log::info!("Len: {:?}", length);
            if length == 0 {
                continue;
            }
            let mut chunk = vec![0; length as usize];
            file.read_exact(&mut chunk)?;
            let mut chunk = std::io::Cursor::new(chunk);
            log::info!("C");
            let mut comp_type = [0; 1];
            chunk.read_exact(&mut comp_type).unwrap();
            log::info!("D");
            let comp_type = comp_type[0];
            if comp_type != 2 {
                log::info!("Wrong. {}", comp_type);
                return Err(anyhow::anyhow!("Unknown compression type!"));
            }
            log::info!("Here lol!");
            use flate2::read::ZlibDecoder;
            use nbt::decode::read_zlib_compound_tag;
            use nbt::CompoundTag;
            let mut tag = read_zlib_compound_tag(&mut chunk)?;
            let mut tag = tag
                .get_compound_tag("Level")
                .or(Err(anyhow::anyhow!("Does not exist!")))?;
            let val = tag
                .get_i8_vec("Blocks")
                .or(Err(anyhow::anyhow!("Does not exist!")))?;
            let block_ids = vec_i8_into_u8(val.clone());
            let val = tag
                .get_i8_vec("Data")
                .or(Err(anyhow::anyhow!("Does not exist!")))?;
            log::info!("Got to here!");
            let block_metadata = vec_i8_into_u8(val.clone());
            use super::chunks::*;
            let metadata = super::chunks::decompress_vec(block_metadata).unwrap();
            let mut blocks = Vec::new();
            let mut i = 0;
            for block in block_ids {
                blocks.push(BlockState {
                    b_type: block,
                    b_metadata: metadata[i],
                    b_light: 0,
                    b_skylight: 0,
                });
                i += 1;
            }
            let x_pos = tag
                .get_i32("xPos")
                .or(Err(anyhow::anyhow!("Does not exist!")))?;
            let z_pos = tag
                .get_i32("zPos")
                .or(Err(anyhow::anyhow!("Does not exist!")))?;
            //log::info!("Compression type: {}", comp_type);
            log::info!("Pos: {} {}", x_pos, z_pos);
            let mut chunksections = Vec::new();
            for i in 0..8 {
                chunksections.push(ChunkSection::new(i));
            }
            for section in 0..8 {
                for x in 0..16 {
                    for z in 0..16 {
                        for y in 0..16 {
                            let y = y + (section * 16);
                            //log::info!("Doing section {}, {} {} {}", section, x, y, z);
                            let section = chunksections.get_mut(section).unwrap();
                            section
                                .get_data()
                                .push(blocks[Self::pos_to_idx(x, y as i32, z)]);
                        }
                    }
                }
            }
            let mut chunk = Chunk {
                pos: ChunkCoords { x: x_pos, z: z_pos },
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
                heightmaps: HeightmapStore::new(),
            };
            chunks.push(chunk);
        }
        Ok(Self { chunks })
        //Err(anyhow::anyhow!("Balls"))
    }
    fn pos_to_idx(x: i32, y: i32, z: i32) -> usize {
        (y + (z * 128) + (x * 128 * 16)) as usize
    }
}

fn vec_u8_into_i8(v: Vec<u8>) -> Vec<i8> {
    // ideally we'd use Vec::into_raw_parts, but it's unstable,
    // so we have to do it manually:

    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = std::mem::ManuallyDrop::new(v);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut i8, len, cap) }
}

// Stackoverflow lol
fn vec_i8_into_u8(v: Vec<i8>) -> Vec<u8> {
    // ideally we'd use Vec::into_raw_parts, but it's unstable,
    // so we have to do it manually:

    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = std::mem::ManuallyDrop::new(v);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p as *mut u8, len, cap) }
}
