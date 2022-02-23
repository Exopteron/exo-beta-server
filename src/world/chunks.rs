use crate::configuration::CONFIGURATION;
use crate::ecs::systems::world::light::LightPropagationManager;
use crate::ecs::systems::world::light::LightPropagationRequest;
//use crate::game::items::ItemRegistry;
use crate::game::ChunkCoords;
use crate::game::RefContainer;
use crate::item::item::block::AtomicRegistryBlock;
use crate::item::item::ItemRegistry;
use crate::world::chunk_map::CHUNK_HEIGHT;
use flume::{Receiver, Sender};
/// The width in blocks of a chunk column.
pub const CHUNK_WIDTH: usize = 16;

/// The height in blocks of a chunk section.
pub const SECTION_HEIGHT: usize = 16;

/// The width in blocks of a chunk section.
pub const SECTION_WIDTH: usize = CHUNK_WIDTH;

/// The volume in blocks of a chunk section.
pub const SECTION_VOLUME: usize = (SECTION_HEIGHT * SECTION_WIDTH * SECTION_WIDTH) as usize;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BlockState {
    pub b_type: u8,
    pub b_metadata: u8,
    pub b_light: u8,
    pub b_skylight: u8,
}
impl std::default::Default for BlockState {
    fn default() -> Self {
        Self {
            b_type: 0,
            b_metadata: 0,
            b_light: 0,
            b_skylight: 0,
        }
    }
}
impl BlockState {
    pub fn is_solid(&self) -> bool {
        if let Ok(t) = self.registry_type() {
            t.is_solid()
        } else {
            false
        }
    }
    pub fn new(id: u8, meta: u8) -> Self {
        Self {
            b_type: id,
            b_metadata: meta,
            b_light: 0,
            b_skylight: 15,
        }
    }
    pub fn registry_type(&self) -> anyhow::Result<AtomicRegistryBlock> {
        ItemRegistry::global()
            .get_block(self.b_type)
            .ok_or(anyhow::anyhow!("Block does not exist in registry"))
    }
    pub fn from_id(id: u8) -> Self {
        Self {
            b_type: id,
            b_skylight: 15,
            b_light: 15,
            b_metadata: 0,
        }
    }
    pub fn is_air(&self) -> bool {
        self.b_type == 0
    }
    pub fn air() -> Self {
        Self {
            b_type: 0,
            b_metadata: 0,
            b_light: 0,
            b_skylight: 15,
        }
    }
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
#[derive(Clone, Debug)]
pub struct ChunkSection {
    data: Vec<BlockState>,
    section: i8,
}
impl Default for ChunkSection {
    fn default() -> Self {
        let mut vec = Vec::new();
        vec.resize(16 * 16 * 16, BlockState::air());
        Self {
            data: vec,
            section: Default::default(),
        }
    }
}
impl ChunkSection {
    fn block_index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= SECTION_WIDTH || y >= SECTION_WIDTH || z >= SECTION_WIDTH {
            None
        } else {
            //Some((y << 8) | (z << 4) | x)
            Some(y + (z * 16) + (x * 16 * 16))
        }
    }
    /// Sets the block at the given coordinates within
    /// this chunk section.
    ///
    /// Returns `None` if the coordinates were out of bounds.
    pub fn set_block_at(&mut self, x: usize, y: usize, z: usize, block: BlockState) -> Option<()> {
        *self.data.get_mut(Self::block_index(x, y, z)?)? = block;
        Some(())
    }
    /// Gets the block at the given coordinates within this
    /// chunk section.
    pub fn block_at(&self, x: usize, y: usize, z: usize) -> Option<BlockState> {
        self.data.get(Self::block_index(x, y, z)?).cloned()
    }

    pub fn new(section: i8) -> Self {
        Self {
            data: Vec::new(),
            section,
        }
    }
    pub fn get_data(&mut self) -> &mut Vec<BlockState> {
        &mut self.data
    }
    pub fn data(&self) -> &Vec<BlockState> {
        &self.data
    }
}
impl ChunkSection {
    pub fn pos_to_index(mut x: i32, mut y: i32, mut z: i32) -> usize {
        /*         x %= 16;
        y %= 16;
        z %= 16; */
        (y + (z * 16) + (x * 16 * 16)) as usize
    }
    pub fn get_block(&mut self, idx: usize) -> Option<&BlockState> {
        //log::info!("Here! {}", idx);
        let len = self.data.len();
        let possible = self.data.get(idx);
        if possible.is_some() {
            return self.data.get(idx);
        } else {
            for i in 0..idx + 5 {
                if let None = self.data.get(i) {
                    self.data.push(BlockState {
                        b_type: 0,
                        b_metadata: 0,
                        b_light: 0,
                        b_skylight: 0,
                    });
                }
            }
            return self.data.get(idx);
        }
        None
    }
}
#[derive(Clone, Debug)]
pub struct Chunk {
    pub pos: ChunkCoords,
    pub data: [Option<ChunkSection>; 8],
    pub heightmaps: HeightmapStore,
}
impl Chunk {
    pub fn fill_layer(&mut self, level: usize, block: BlockState) {
        for x in 0..16 {
            for z in 0..16 {
                self.set_block_at(x, level, z, block);
            }
        }
    }
    pub fn fill_layer_air(&mut self, level: usize, block: BlockState) {
        for x in 0..16 {
            for z in 0..16 {
                if let Some(b) = self.block_at(x, level, z) {
                    if b.is_air() {
                        self.set_block_at(x, level, z, block);
                    }
                }
            }
        }
    }
    pub fn new(coords: ChunkCoords) -> Self {
        let data = [
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
            Some(ChunkSection::default()),
        ];
        Self {
            pos: coords,
            data,
            heightmaps: HeightmapStore::new(),
        }
    }
    pub fn block_at_fn(
        sections: &[Option<ChunkSection>],
    ) -> impl Fn(usize, usize, usize) -> BlockState + '_ {
        move |x, y, z| {
            let section = &sections[y / SECTION_HEIGHT];
            match section {
                Some(section) => section.block_at(x, y % SECTION_HEIGHT, z).unwrap(),
                None => BlockState::air(),
            }
        }
    }
    /// Sets the section at index `y`.
    pub fn set_section_at(&mut self, y: isize, section: Option<ChunkSection>) {
        self.data[y as usize] = section;
    }
    pub fn calculate_full_skylight(&mut self) {
        for x in 0..16 {
            for z in 0..16 {
                self.calculate_skylight(x, z);
            }
        }
    }
    // pub fn global_skylight_requests(&mut self) -> Vec<LightPropagationRequest> {
    //     let mut requests = Vec::new();
    //     for x in 0..16 {
    //         for z in 0..16 {
    //             for y in 0..CHUNK_HEIGHT {
    //                 if let Some(mut b) = self.block_at(x, y as usize, z) {
    //                     b.b_skylight = 0;
    //                     self.set_block_at_nlh(x, y as usize, z, b);
    //                 }
    //             }
    //             if let Some(y) = self.heightmaps.light_blocking.height(x, z) {
    //                 for y in y..128 {
    //                     if let Some(mut b) = self.block_at(x, y, z) {
    //                         b.b_skylight = 15;
    //                         self.set_block_at_nlh(x, y, z, b);
    //                     }
    //                 }
    //                 let pos = BlockPosition::new((x + (self.pos.x as usize) * 16) as i32, y as i32, (z + (self.pos.z as usize) * 16) as i32, self.pos.world);
    //                 requests.push(LightPropagationRequest { position: pos, world: 0, level: 15, skylight: true }); //TODO: unhardcodeworld
    //             }
    //         }
    //     }
    //     requests
    // }
    pub fn calculate_skylight(&mut self, x: usize, z: usize) -> Option<BlockPosition> {
        if let Some(y) = self.heightmaps.light_blocking.height(x, z) {
            for y in y..128 {
                if let Some(mut b) = self.block_at(x, y, z) {
                    b.b_skylight = 15;
                    self.set_block_at_nlh(x, y, z, b);
                }
            }
            for y in 0..y {
                if let Some(mut b) = self.block_at(x, y, z) {
                    b.b_skylight = 0;
                    self.set_block_at_nlh(x, y, z, b);
                }
            }
            return Some(BlockPosition::new((x + (self.pos.x as usize) * 16) as i32, (y + 0) as i32, (z + (self.pos.z as usize) * 16) as i32, self.pos.world));
        } else {
            for y in 0..128 {
                if let Some(mut b) = self.block_at(x, y, z) {
                    b.b_skylight = 15;
                    self.set_block_at_nlh(x, y, z, b);
                }
            }
        }
        None
    }
    /// Sets the block at the given position within this chunk without updating the heightmap/skylight.
    ///
    /// Returns `None` if the coordinates are out of bounds.
    pub fn set_block_at_nlh(&mut self, x: usize, y: usize, z: usize, block: BlockState) -> Option<()> {
        let old_block = self.block_at(x, y, z)?;
        let section = self.section_for_y_mut(y)?;
        let result = match section {
            Some(section) => {
                let result = section.set_block_at(x, y % SECTION_HEIGHT, z, block);
                /*                 // If the block update caused the section to contain only
                // air, free it to conserve memory.
                if section.is_empty() {
                    self.clear_section(y);
                } */
                result
            }
            None => {
                if !block.is_air() {
                    let mut section = ChunkSection::default();
                    let result = section.set_block_at(x, y % SECTION_HEIGHT, z, block);
                    self.set_section_at((y / SECTION_HEIGHT) as isize, Some(section));
                    result
                } else {
                    Some(())
                }
            }
        };
        result
    }
    /// Sets the block at the given position within this chunk.
    ///
    /// Returns `None` if the coordinates are out of bounds.
    pub fn set_block_at(&mut self, x: usize, y: usize, z: usize, block: BlockState) -> Option<()> {
        let old_block = self.block_at(x, y, z)?;
        let result = self.set_block_at_nlh(x, y, z, block);
        //log::info!("Updating heightmap at {}, {}, {}", x, y, z);
        self.heightmaps
            .update(x, y, z, old_block, block, Self::block_at_fn(&self.data));
        result
    }
    /// Gets the block at the given position within this chunk.
    ///
    /// Returns `None` if the coordinates are out of bounds.
    pub fn block_at(&self, x: usize, y: usize, z: usize) -> Option<BlockState> {
        let section = self.section_for_y(y)?;
        match section {
            Some(section) => section.block_at(x, y % SECTION_HEIGHT, z),
            None => Some(BlockState::air()),
        }
    }
    fn section_for_y_mut(&mut self, y: usize) -> Option<&mut Option<ChunkSection>> {
        self.data.get_mut(y / SECTION_HEIGHT)
    }

    fn section_for_y(&self, y: usize) -> Option<&Option<ChunkSection>> {
        self.data.get(y / SECTION_HEIGHT)
    }
    pub fn sections(&self) -> &[Option<ChunkSection>] {
        &self.data
    }
    pub fn position(&self) -> ChunkCoords {
        self.pos
    }
    pub fn plain(x: i32, z: i32, world: i32) -> Self {
        let mut v = Self {
            pos: ChunkCoords { x, z, world },
            data: [None, None, None, None, None, None, None, None],
            heightmaps: HeightmapStore::new(),
        };
        v
    }
}
use crate::game::{BlockPosition, Position};
use std::collections::VecDeque;
use std::time::Instant;

use super::heightmap::HeightmapStore;
use super::light;
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
    if input.len() == 0 {
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
    if input.len() == 0 {
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
