use super::*;
use once_cell::sync::OnceCell;
pub mod default;
pub static ITEM_REGISTRY: OnceCell<ItemRegistry> = OnceCell::new();
use std::collections::HashMap;
pub struct ItemRegistry {
    items: HashMap<i16, Arc<RegistryItem>>
}
impl ItemRegistry {
    pub fn global() -> &'static ItemRegistry {
        ITEM_REGISTRY.get().expect("Item registry is not initialized!")
    } 
    pub fn new() -> Self {
        Self { items: HashMap::new() }
    }
    pub fn register_item(&mut self, id: i16, registry_name: &str, item: Box<dyn Item + Send + Sync>) {
        self.items.insert(id, Arc::new(RegistryItem { name: registry_name.to_string(), item: Arc::new(item) }));
    }
    pub fn get_item(&self, id: i16) -> Option<Arc<RegistryItem>> {
        Some(self.items.get(&id)?.clone())
    }
}
pub trait Item {
    fn is_block(&self) -> bool;
    fn stack_size(&self) -> i16;
    fn on_use(&self, game: &mut Game, packet: crate::network::packet::PlayerBlockPlacement, player: Arc<PlayerRef>) -> anyhow::Result<()>;
}
pub struct RegistryItem {
    name: String,
    item: Arc<Box<dyn Item + Send + Sync>>
}
impl RegistryItem {
    pub fn get_item(&self) -> Arc<Box<dyn Item + Send + Sync>> {
        self.item.clone()
    }
}