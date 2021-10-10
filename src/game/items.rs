use super::*;
use once_cell::sync::OnceCell;
pub mod default;
pub mod block_utils;
pub mod block;
pub static ITEM_REGISTRY: OnceCell<ItemRegistry> = OnceCell::new();
use std::collections::HashMap;
#[derive(Hash, PartialEq)]
pub struct Recipe2X2 {
    recipe: [ItemStack; 4],
}
impl Eq for Recipe2X2 {}
pub struct ItemRegistry {
    items: HashMap<i16, Arc<RegistryItem>>,
    recipe2x2: HashMap<Recipe2X2, ItemStack>,
}
pub enum ToolType {
    PICKAXE,
    CHESTPLATE,
    LEGGINGS,
    HELMET,
    BOOTS,
}
impl ItemRegistry {
    pub fn global() -> &'static ItemRegistry {
        ITEM_REGISTRY.get().expect("Item registry is not initialized!")
    } 
    pub fn new() -> Self {
        log::info!("[ItemRegistry] Initializing item registry");
        Self { items: HashMap::new(), recipe2x2: HashMap::new() }
    }
    pub fn register_item(&mut self, id: i16, registry_name: &str, item: Box<dyn Item + Send + Sync>) {
        log::info!("[ItemRegistry] Registering item \"{}\" ({})", registry_name, id);
        self.items.insert(id, Arc::new(RegistryItem { name: registry_name.to_string(), item: Arc::new(item) }));
    }
    pub fn register_2x2_recipe(&mut self, recipe: [ItemStack; 4], output: ItemStack) {
        self.recipe2x2.insert(Recipe2X2 { recipe }, output);
    }
    pub fn get_item(&self, id: i16) -> Option<Arc<RegistryItem>> {
        Some(self.items.get(&id)?.clone())
    }
    pub fn get_recipe(&self, recipe: [ItemStack; 4]) -> Option<ItemStack> {
        Some(self.recipe2x2.get(&Recipe2X2 { recipe })?.clone())
    }
}
pub trait Item {
    fn is_block(&self) -> bool;
    fn stack_size(&self) -> i16;
    fn on_use(&self, game: &mut Game, packet: crate::network::packet::PlayerBlockPlacement, player: Arc<PlayerRef>) -> anyhow::Result<()>;
    fn as_block(&self) -> Option<&dyn block::Block> {
        None
    }
    fn get_tool_type(&self) -> Option<ToolType> {
        None
    }
    fn wearable(&self) -> bool {
        false
    }
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