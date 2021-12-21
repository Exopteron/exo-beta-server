use serde::{Deserialize, Serialize};

use crate::{game::{BlockPosition}, protocol::packets::Face, item::stack::ItemStack};

#[derive(Debug, Clone)]
pub struct BlockInteractEvent {
    pub held_item: ItemStack,
    pub location: BlockPosition,
    pub face: Face,
}

#[derive(Debug, Clone)]
pub struct BlockPlacementEvent {
    pub held_item: ItemStack,
    pub location: BlockPosition,
    pub face: Face,
    pub world: i32,
}
