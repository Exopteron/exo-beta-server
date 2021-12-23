use std::{sync::Arc, fmt::Debug};

use ahash::AHashMap;
use hecs::Entity;
use once_cell::sync::OnceCell;
use parking_lot::{RawMutex, MutexGuard};

use crate::{game::{BlockPosition, Game}, protocol::packets::Face, ecs::systems::SysResult, server::Server};

use self::block::{AtomicRegistryBlock, RegistryBlock, Block};

use super::{stack::ItemStack, inventory_slot::InventorySlot};
pub mod block;
// featerlicense in FEATHER_LICENSE.md
static ITEM_REGISTRY: OnceCell<ItemRegistry> = OnceCell::new();
pub type ItemIdentifier = i16;
pub type BlockIdentifier = (u8, u8);
pub struct BlockUseTarget {
    pub position: BlockPosition,
    pub world: i32,
    pub face: Face,
}
pub trait Item {
    fn id(&self) -> ItemIdentifier;
    fn stack_size(&self) -> i8;
    fn durability(&self) -> Option<i16>;
    fn on_use(&self, game: &mut Game, server: &mut Server, item_slot: usize, user: Entity, target: Option<BlockUseTarget>) -> SysResult {
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
pub struct ItemRegistry {
    items: AHashMap<ItemIdentifier, AtomicRegistryItem>,
    blocks: AHashMap<BlockIdentifier, AtomicRegistryBlock>,
}

impl ItemRegistry {
    pub fn global() -> &'static ItemRegistry {
        ITEM_REGISTRY.get().expect("Attempted to get item registry before init")
    }
    pub fn set(self) {
        ITEM_REGISTRY.set(self).ok().expect("Already set item registry!");
    }
    pub fn new() -> Self {
        Self {
            items: AHashMap::new(),
            blocks: AHashMap::new(),
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
}