use std::iter;

use itertools::Either;

use crate::{game::{BlockPosition, ChunkCoords}, world::chunks::{SECTION_HEIGHT, SECTION_VOLUME}};

/// Event triggered when one or more blocks are changed.
///
/// This event can efficiently store bulk block updates
/// using a variety of different representations. Cloning
/// is cheap as it is, at worst, cloning an `Arc`.
#[derive(Debug, Clone)]
pub struct BlockChangeEvent {
    changes: BlockChanges,
    pub update_neighbors: bool,
}

impl BlockChangeEvent {
    /// Creates an event affecting a single block.
    pub fn single(pos: BlockPosition, world: i32) -> Self {
        Self {
            changes: BlockChanges::Single { pos, world },
            update_neighbors: true,
        }
    }

    /// Creates an event corresponding to a block update
    /// that fills an entire chunk section with the same block.
    pub fn fill_chunk_section(chunk: ChunkCoords, section: u32, world: i32) -> Self {
        Self {
            changes: BlockChanges::FillChunkSection { chunk, section, world },
            update_neighbors: true
        }
    }

    /// Determines the number of blocks that were
    /// changed in this block change event.
    pub fn count(&self) -> usize {
        match &self.changes {
            BlockChanges::Single { .. } => 1,
            BlockChanges::FillChunkSection { .. } => SECTION_VOLUME,
        }
    }
    pub fn world(&self) -> i32 {
        self.changes.world()
    }
    /// Returns an iterator over block positions affected by this block change.
    pub fn iter_changed_blocks(&self) -> impl Iterator<Item = BlockPosition> + '_ {
        match &self.changes {
            BlockChanges::Single { pos, .. } => Either::Left(iter::once(*pos)),
            BlockChanges::FillChunkSection { chunk, section, .. } => {
                Either::Right(iter_section_blocks(*chunk, *section))
            }
        }
    }

    /// Returns an iterator over chunk section positions affected by this block change.
    ///
    /// The yielded tuple consists of `(chunk, section_y, num_changed_blocks)`,
    /// where `num_changed_blocks` is the number of blocks changed within that chunk.
    pub fn iter_affected_chunk_sections(
        &self,
    ) -> impl Iterator<Item = (ChunkCoords, usize, usize)> + '_ {
        match &self.changes {
            BlockChanges::Single { pos, .. } => {
                iter::once((pos.to_chunk_coords(), pos.y as usize / SECTION_HEIGHT, 1))
            }
            BlockChanges::FillChunkSection { chunk, section, .. } => {
                iter::once((*chunk, *section as usize, SECTION_VOLUME))
            }
        }
    }
}

fn iter_section_blocks(
    chunk: ChunkCoords,
    section: u32,
) -> impl Iterator<Item = BlockPosition> {
    (0..16)
        .flat_map(|x| (0..16).map(move |y| (x, y)))
        .flat_map(|(x, y)| (0..16).map(move |z| (x, y, z)))
        .map(move |(dx, dy, dz)| {
            let x = dx + chunk.x * 16;
            let y = dy + section as i32 * 16;
            let z = dz + chunk.z * 16;

            // It's safe to unwrap because we are working from a valid source of block positions
            BlockPosition::new(x, y, z, chunk.world)
        })
}

#[derive(Debug, Clone)]
enum BlockChanges {
    /// A single block change.
    Single { pos: BlockPosition, world: i32 },
    /// A whole chunk section was filled with the same block.
    FillChunkSection { chunk: ChunkCoords, section: u32, world: i32 },
}

impl BlockChanges {
    pub fn world(&self) -> i32 {
        match self {
            Self::Single { pos, world } => *world,
            Self::FillChunkSection { world, .. } => *world,
        }
    }
}