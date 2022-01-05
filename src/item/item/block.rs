use std::{sync::Arc, fmt::Debug};
pub mod fluid;
use anvil_region::position;
use hecs::{Entity, EntityBuilder};

use crate::{game::{BlockPosition, Game}, protocol::packets::Face, item::stack::ItemStack, events::block_interact::BlockPlacementEvent, world::chunks::BlockState, ecs::{systems::SysResult, EntityRef}, server::Server, aabb::AABB, block_entity::{BlockEntityNBTLoader, BlockEntityNBTLoaders}};

use super::BlockIdentifier;
pub enum ActionResult {
    PASS,
    SUCCESS
}
pub struct BurnRate(pub i32, pub i32);
pub trait Block {
    fn burn_rate(&self) -> Option<BurnRate> {
        None
    }
    fn id(&self) -> BlockIdentifier;
    fn item_stack_size(&self) -> i8;
    fn on_break(&self, game: &mut Game, server: &mut Server, breaker: Entity, mut position: BlockPosition, face: Face, world: i32) {
        
    }
    fn added(&self, world: i32, game: &mut Game, server: &mut Server, position: BlockPosition, state: BlockState) {

    }
    fn on_collide(&self, game: &mut Game, position: BlockPosition, state: BlockState, player: Entity) -> SysResult {
        Ok(())
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
    fn can_place_over(&self) -> bool {
        false
    }
    fn collision_box(&self, state: BlockState, position: BlockPosition) -> Option<AABB> {
        Some(AABB::new(position.x as f64, position.y as f64, position.z as f64, position.x as f64 + 1., position.y as f64 + 1., position.z as f64 + 1.))
    }
    fn block_entity(&self, entity_builder: &mut EntityBuilder, state: BlockState, position: BlockPosition) -> bool {
        false
    }
    fn tick(&self, world: i32, game: &mut Game, state: BlockState, position: BlockPosition) {

    }
    fn upd_tick(&self, world: i32, game: &mut Game, state: BlockState, position: BlockPosition, reschedule: &mut Option<u128>) {
        
    }
    fn absorbs_fall(&self) -> bool {
        false
    }
    fn block_entity_loader(&self, loaders: &mut BlockEntityNBTLoaders) {
        
    }
    fn opaque(&self) -> bool {
        true
    }
}
impl Debug for RegistryBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{id: {}}}", self.id())
    }
}
impl PartialEq for RegistryBlock {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for RegistryBlock {}
pub type NonBoxedRegBlock = dyn Block + Sync + Send;
pub type RegistryBlock = Box<dyn Block + Sync + Send>;
pub type AtomicRegistryBlock = Arc<RegistryBlock>;