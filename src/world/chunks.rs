use crate::configuration::CONFIGURATION;
//use crate::game::items::ItemRegistry;
use crate::game::ChunkCoords;
use crate::game::RefContainer;
use flume::{Receiver, Sender};
/// The width in blocks of a chunk column.
pub const CHUNK_WIDTH: usize = 16;

/// The height in blocks of a chunk section.
pub const SECTION_HEIGHT: usize = 16;

/// The width in blocks of a chunk section.
pub const SECTION_WIDTH: usize = CHUNK_WIDTH;
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
    pub fn is_air(&self) -> bool {
        self.b_type == 0
    }
    pub fn air() -> Self {
        Self {
            b_type: 0,
            b_metadata: 0,
            b_light: 0,
            b_skylight: 0,
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
#[derive(Clone, Debug, Default)]
pub struct ChunkSection {
    data: Vec<BlockState>,
    x: i32,
    z: i32,
    section: i8,
}
impl ChunkSection {
    fn block_index(x: usize, y: usize, z: usize) -> Option<usize> {
        if x >= SECTION_WIDTH || y >= SECTION_WIDTH || z >= SECTION_WIDTH {
            None
        } else {
            Some((y << 8) | (z << 4) | x)
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

    pub fn new(x: i32, z: i32, section: i8) -> Self {
        Self {
            data: Vec::new(),
            x,
            z,
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
    pub heightmap: [[i8; 16]; 16],
}
impl Chunk {
    /// Sets the section at index `y`.
    pub fn set_section_at(&mut self, y: isize, section: Option<ChunkSection>) {
        self.data[y as usize] = section;
    }
    /// Sets the block at the given position within this chunk.
    ///
    /// Returns `None` if the coordinates are out of bounds.
    /// FIXME: Do not update heightmap when it is not neccessary
    pub fn set_block_at(&mut self, x: usize, y: usize, z: usize, block: BlockState) -> Option<()> {
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
        self.data.get_mut((y / SECTION_HEIGHT) + 1)
    }

    fn section_for_y(&self, y: usize) -> Option<&Option<ChunkSection>> {
        self.data.get((y / SECTION_HEIGHT) + 1)
    }
    pub fn sections(&self) -> &[Option<ChunkSection>] {
        &self.data
    }
    pub fn position(&self) -> ChunkCoords {
        self.pos
    }
    pub fn plain(x: i32, z: i32) -> Self {
        let mut v = Self {
            pos: ChunkCoords { x, z },
            data: [None, None, None, None, None, None, None, None],
            heightmap: [[0; 16]; 16],
        };
        v
    }
}
use crate::game::{BlockPosition, Position};
use std::collections::VecDeque;
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
