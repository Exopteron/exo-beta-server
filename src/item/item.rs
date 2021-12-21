use std::{sync::Arc, fmt::Debug};

use ahash::AHashMap;
use once_cell::sync::OnceCell;

// featerlicense in FEATHER_LICENSE.md
static ITEM_REGISTRY: OnceCell<ItemRegistry> = OnceCell::new();
pub type ItemIdentifier = (i16, i16);
pub trait Item {
    fn id(&self) -> ItemIdentifier;
    fn stack_size(&self) -> i8;
    fn durability(&self) -> Option<i16>;
}
impl Debug for RegistryItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Item {{id: {}:{}}}", self.id().0, self.id().1)
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
    items: AHashMap<ItemIdentifier, AtomicRegistryItem>
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
            items: AHashMap::new()
        }
    }
    pub fn register_item(&mut self, item: RegistryItem) {
        self.items.insert(item.id(), Arc::new(item));
    }
    pub fn get_item(&self, id: ItemIdentifier) -> Option<AtomicRegistryItem> {
        self.items.get(&id).cloned()
    }
}