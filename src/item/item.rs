use std::{sync::Arc, fmt::Debug};

use ahash::AHashMap;
use hecs::Entity;
use once_cell::unsync::OnceCell;
use parking_lot::{RawMutex, MutexGuard};
use rustc_hash::FxHashMap;

use crate::{game::{BlockPosition, Game}, protocol::packets::Face, ecs::systems::SysResult, server::Server, block_entity::BlockEntityNBTLoaders, world::chunks::BlockState};

use self::block::{AtomicRegistryBlock, RegistryBlock, Block};

use super::{stack::ItemStack, inventory_slot::InventorySlot, crafting::Solver, default::tools::ToolMaterials};
pub mod block;
// featerlicense in FEATHER_LICENSE.md
static mut ITEM_REGISTRY: Option<Arc<ItemRegistry>> = None;
pub type ItemIdentifier = i16;
pub type BlockIdentifier = u8;
pub struct BlockUseTarget {
    pub position: BlockPosition,
    pub world: i32,
    pub face: Face,
}
pub trait Item {
    fn id(&self) -> ItemIdentifier;
    fn stack_size(&self) -> i8;
    fn durability(&self) -> Option<i16>;
    fn on_use(&self, game: &mut Game, server: &mut Server, item: MutexGuard<InventorySlot>, slot: usize, user: Entity, target: Option<BlockUseTarget>) -> SysResult {
        Ok(())
    }
    fn on_eat(&self, game: &mut Game, server: &mut Server, eater: Entity, item: MutexGuard<InventorySlot>, slot: usize) -> SysResult {
        Ok(())
    }
    fn on_stop_using(&self, game: &mut Game, server: &mut Server, eater: Entity, item: MutexGuard<InventorySlot>, slot: usize) -> SysResult {
        Ok(())
    }
    fn tool_type(&self) -> Option<ToolMaterials> {
        None
    }
    fn on_dig_with(&self, game: &mut Game, server: &mut Server, digger: Entity, item: &mut MutexGuard<InventorySlot>, slot: usize, target: BlockUseTarget) -> SysResult {
        Ok(())
    }
}
impl Debug for RegistryItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{id: {}}}", self.id())
    }
}
impl PartialEq for RegistryItem {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}
impl Eq for RegistryItem {}
type RegistryItem = Box<dyn Item + Sync + Send>;
pub type AtomicRegistryItem = Arc<RegistryItem>;
#[derive(Clone)]
pub struct ItemRegistry {
    items: AHashMap<ItemIdentifier, AtomicRegistryItem>,
    blocks: AHashMap<BlockIdentifier, AtomicRegistryBlock>,
    pub solver: Solver,
}

impl ItemRegistry {
    pub fn global() -> Arc<ItemRegistry> {
        unsafe {
            Arc::clone(ITEM_REGISTRY.as_ref().unwrap())
        }
    }
    pub fn set(self) {
        unsafe {
            ITEM_REGISTRY = Some(Arc::new(self));
        }
    }
    pub fn new(recipe_solver: Solver) -> Self {
        Self {
            items: AHashMap::new(),
            blocks: AHashMap::default(),
            solver: recipe_solver
        }
    }
    pub fn register_block(&mut self, block: impl Block + Sync + Send + 'static) {
        if self.blocks.contains_key(&block.id()) {
            return;
        }
        self.blocks.insert(block.id(), Arc::new(Box::new(block)));
    }
    pub fn get_block(&self, id: BlockIdentifier) -> Option<AtomicRegistryBlock> {
        self.blocks.get(&id).cloned()
    }
    pub fn register_item(&mut self, item: impl Item + Sync + Send + 'static) {
        self.items.insert(item.id(), Arc::new(Box::new(item)));
    }
    pub fn get_item(&self, id: ItemIdentifier) -> Option<AtomicRegistryItem> {
        self.items.get(&id).cloned()
    }
    pub fn apply_loaders(&self, loaders: &mut BlockEntityNBTLoaders) {
        for (_, block) in self.blocks.iter() {
            block.block_entity_loader(loaders);
        }
    }
}