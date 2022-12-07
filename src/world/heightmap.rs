use std::{marker::PhantomData, sync::Arc};

use super::{chunks::{BlockState, CHUNK_WIDTH}, packed_array::PackedArray};
use crate::{world::chunk_map::CHUNK_HEIGHT, item::item::{block::Block, ItemRegistry}};
/// Stores heightmaps for a chunk.
#[derive(Debug, Clone)]
pub struct HeightmapStore {
    pub light_blocking: Heightmap<LightBlocking>,
}

impl Default for HeightmapStore {
    fn default() -> Self {
        Self::new()
    }
}

impl HeightmapStore {
    pub fn new() -> Self {
        Self {
            light_blocking: Heightmap::new(),
        }
    }

    pub fn update(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        old_block: BlockState,
        new_block: BlockState,
        get_block: impl Fn(usize, usize, usize) -> BlockState,
    ) {
        self.light_blocking
            .update(x, y, z, old_block, new_block, &get_block);
    }

    pub fn recalculate(&mut self, get_block: impl Fn(usize, usize, usize) -> BlockState) {
        self.light_blocking.recalculate(&get_block);
    }
}

/// A function used to compute heightmaps.
pub trait HeightmapFunction {
    /// Returns whether a block should be considered
    /// "solid" during the heightmap computation.
    fn is_solid(block: BlockState, registry: Arc<ItemRegistry>) -> bool;
}

#[derive(Debug, Clone)]
pub struct LightBlocking;
impl HeightmapFunction for LightBlocking {
    fn is_solid(block: BlockState, registry: Arc<ItemRegistry>) -> bool {
        if let Some(block) = registry.get_block(block.b_type) {
            return block.opacity() > 0;
        }
        true
    }
}

/* #[derive(Debug, Clone)]
pub struct MotionBlocking;
impl HeightmapFunction for MotionBlocking {
    fn is_solid(block: BlockState) -> bool {
        block.is_solid()
    }
}
 */
/* #[derive(Debug, Clone)]
pub struct MotionBlockingNoLeaves;
impl HeightmapFunction for MotionBlockingNoLeaves {
    fn is_solid(block: BlockState) -> bool {
        (block.is_solid() || block.is_fluid())
            && block.simplified_kind() != SimplifiedBlockKind::Leaves
    }
}
 */

#[derive(Debug, Clone)]
pub struct Heightmap<F> {
    heights: PackedArray,
    _marker: PhantomData<F>,
}

impl<F> Default for Heightmap<F>
where
    F: HeightmapFunction,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<F> Heightmap<F>
where
    F: HeightmapFunction,
{
    pub fn new() -> Self {
        Self {
            heights: PackedArray::new(256, 9),
            _marker: PhantomData,
        }
    }

    pub fn set_height(&mut self, x: usize, z: usize, height: usize) {
        let index = self.index(x, z);
        self.heights.set(index, height as u64);
    }

    pub fn height(&self, x: usize, z: usize) -> Option<usize> {
        let index = self.index(x, z);
        self.heights.get(index).map(|x| x as usize)
    }

    pub fn as_u64_slice(&self) -> &[u64] {
        self.heights.as_u64_slice()
    }

    fn index(&self, x: usize, z: usize) -> usize {
        (z << 4) | x
    }

    /// Updates this height map after a block has been updated.
    pub fn update(
        &mut self,
        x: usize,
        y: usize,
        z: usize,
        old_block: BlockState,
        new_block: BlockState,
        get_block: impl Fn(usize, usize, usize) -> BlockState,
    ) {
        log::info!("Calculating heightmap for {}, {}, {}", x, y, z);
        let registry = ItemRegistry::global();
        let y = y + 1;
        if F::is_solid(old_block, registry.clone()) && self.height(x, z) == Some(y) {
            // This was old the highest block
            for i in (0..y).rev() {
                let block = get_block(x, i, z);

                if F::is_solid(block, registry.clone()) {
                    self.set_height(x, z, i + 1);
                    break;
                } else if y == 0 {
                    self.set_height(x, z, 0);
                }
            }
        }
        if F::is_solid(new_block, registry) && self.height(x, z).unwrap() < y {
            // This is the new highest block
            self.set_height(x, z, y);
        }
    }

    /// Recalculates this entire heightmap.
    pub fn recalculate(&mut self, get_block: impl Fn(usize, usize, usize) -> BlockState) {
        let registry = ItemRegistry::global();
        for x in 0..CHUNK_WIDTH {
            for z in 0..CHUNK_WIDTH {
                for y in (0..CHUNK_HEIGHT).rev() {
                    if F::is_solid(get_block(x, y as usize, z), registry.clone()) {
                        self.set_height(x, z, y as usize + 1);
                        break;
                    }
                }
            }
        }
    }
}
