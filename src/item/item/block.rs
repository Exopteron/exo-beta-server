use std::{sync::Arc, fmt::Debug};

use hecs::{Entity, EntityBuilder};

use crate::{game::{BlockPosition, Game}, protocol::packets::Face, item::stack::ItemStack, events::block_interact::BlockPlacementEvent, world::chunks::BlockState, ecs::{systems::SysResult, EntityRef}, server::Server, aabb::AABB};

use super::BlockIdentifier;
pub enum ActionResult {
    PASS,
    SUCCESS
}
pub trait Block {
    fn id(&self) -> BlockIdentifier;
    fn item_stack_size(&self) -> i8;
    fn on_break(&self, game: &mut Game, breaker: Entity, mut position: BlockPosition, face: Face, world: i32) {
        
    }
    fn place(&self, game: &mut Game, placer: Entity, item: ItemStack, mut position: BlockPosition, face: Face, world: i32) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }
    fn is_solid(&self) -> bool {
        true
    }
    /// Face will be invalid if caused by itself
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        Ok(())
    }
    fn can_place_on(&self, world: i32, game: &mut Game, position: BlockPosition, face: Face) -> bool {
        true
    }
    fn interacted_with(&self, world: i32, game: &mut Game, server: &mut Server, position: BlockPosition, state: BlockState, player: Entity) -> ActionResult {
        ActionResult::PASS
    }
    fn collision_box(&self, state: BlockState, position: BlockPosition) -> AABB {
        AABB::new(position.x as f64, position.y as f64, position.z as f64, position.x as f64 + 1., position.y as f64 + 1., position.z as f64 + 1.)
    }
    fn block_entity(&self, entity_builder: &mut EntityBuilder, state: BlockState, position: BlockPosition) -> bool {
        false
    }
}
impl Debug for RegistryBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{id: {}:{}}}", self.id().0, self.id().1)
    }
}
impl PartialEq for RegistryBlock {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for RegistryBlock {}
pub type RegistryBlock = Box<dyn Block + Sync + Send>;
pub type AtomicRegistryBlock = Arc<RegistryBlock>;