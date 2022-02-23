use ahash::AHashMap;

use super::{item::ItemIdentifier, stack::ItemStack};

#[derive(Default, Clone)]
pub struct FurnaceSolver {
    items: AHashMap<ItemIdentifier, ItemStack>,
}
impl FurnaceSolver {
    pub fn add_item(&mut self, id: ItemIdentifier, stack: ItemStack) {
        self.items.insert(id, stack);
    }
    pub fn get_item(&self, id: ItemIdentifier) -> Option<ItemStack> {
        self.items.get(&id).cloned()
    }
}