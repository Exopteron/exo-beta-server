use crate::game::ChunkCoords;
use crate::game::Position;
use crate::world;
use crate::world::chunks::*;
pub struct MCRegionLoader {
    world_dir: String,
    region_cache: HashMap<(i32, i32), Arc<Region>>,
}
impl MCRegionLoader {
    pub fn new(path: &str) -> Self {
        Self {
            world_dir: path.to_string(),
            region_cache: HashMap::new(),
        }
    }
    pub fn get_chunk(&mut self, coords: ChunkCoords) -> Option<Chunk> {
        let region_x = ((coords.x as f32) / 32.0).floor() as i32;
        let region_z = ((coords.z as f32) / 32.0).floor() as i32;
        log::info!(
            "{:?} in region {}, {} looking for {} {}",
            coords,
            region_x,
            region_z,
            coords.x,
            coords.z
        );
        let region: Arc<Region>;
        if let Some(r) = self.region_cache.get(&(region_x, region_z)) {
            region = r.clone();
        } else {
            // TODO don't do this. just read from the offset.
            region = Arc::new(Region::from_file(&format!(
                "{}/region/r.{}.{}.mcr",
                self.world_dir, region_x, region_z
            ))
            .ok()?);
            self.region_cache
            .insert((region_x, region_z), region.clone());
        }
        for chunk in region.chunks.iter() {
            //log::info!("Running {} {}", chunk.x, chunk.z);
            if (chunk.x == coords.x) && (chunk.z == coords.z) {
                log::info!("Found: {}, {}", chunk.x, chunk.z);
                return Some(chunk.clone());
            }
        }
        None
    }
}
#[derive(Clone)]
pub struct Region {
    pub chunks: Vec<Chunk>,
}
use std::borrow::Borrow;
use std::collections::HashMap;
use std::convert::TryInto;
use std::hash::Hash;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
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
use super::chunks::*;
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
    pub fn from_file(file: &str) -> anyhow::Result<Self> {
        let mut file = std::fs::File::open(file)?;
        let mut chunks = Vec::new();
        let mut present_chunks = Vec::new();
        file.seek(SeekFrom::Start(0))?;
        for _ in 0..1024 {
            let mut offset = vec![0; 3];
            file.read_exact(&mut offset)?;
            offset.reverse();
            offset.push(0);
            offset.reverse();
            let offset: u32 = u32::from_be_bytes(
                offset
                    .try_into()
                    .or_else(|_| Err(anyhow::anyhow!("couldn't convert?")))?,
            );
            let mut sector_count = [0; 1];
            file.read_exact(&mut sector_count)?;
            if offset != 0 && sector_count[0] != 0 {
                //log::info!("Adding {} {}", offset as i32, sector_count[0]);
                present_chunks.push(PresentChunk {
                    offset,
                    sector_count: sector_count[0],
                    timestamp: 0,
                });
            }
        }
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
            file.seek(SeekFrom::Start(chunk.offset as u64 * 4096))?;
            let mut integer = [0; 4];
            file.read_exact(&mut integer)?;
            let length = u32::from_be_bytes(integer);
            let mut chunk = vec![0; length as usize];
            file.read_exact(&mut chunk)?;
            let mut chunk = std::io::Cursor::new(chunk);
            let mut comp_type = [0; 1];
            chunk.read_exact(&mut comp_type)?;
            let comp_type = comp_type[0];
            if comp_type != 2 {
                return Err(anyhow::anyhow!("Unknown compression type!"));
            }
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
            let block_metadata = vec_i8_into_u8(val.clone());
            use super::chunks::*;
            let metadata = super::chunks::decompress_vec(block_metadata).unwrap();
            let mut blocks = Vec::new();
            let mut i = 0;
            for block in block_ids {
                blocks.push(Block {
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
            let mut chunksections = Vec::new();
            for i in 0..8 {
                chunksections.push(ChunkSection::new(x_pos, z_pos, i));
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
                x: x_pos,
                z: z_pos,
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
            chunk.calculate_heightmap()?;
            chunks.push(chunk);
        }
        Ok(Self { chunks })
        //Err(anyhow::anyhow!("Balls"))
    }
    fn pos_to_idx(x: i32, y: i32, z: i32) -> usize {
        (y + (z * 128) + (x * 128 * 16)) as usize
    }
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
