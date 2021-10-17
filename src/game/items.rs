use super::*;
use once_cell::sync::OnceCell;
pub mod default;
pub mod block_utils;
pub mod block;
pub mod crafting;
use crafting::*;
pub static ITEM_REGISTRY: OnceCell<ItemRegistry> = OnceCell::new();
use std::collections::HashMap;
#[derive(Hash, PartialEq)]
pub struct Recipe2X2 {
    recipe: [ItemStack; 4],
}
impl Eq for Recipe2X2 {}

#[derive(Hash, PartialEq)]
pub struct Recipe3X3 {
    recipe: [ItemStack; 9],
}
#[derive(Hash, PartialEq)]
pub struct RecipeShapeless {
    recipe: [[Option<ItemStack>; 3]; 3],
}
impl Eq for Recipe3X3 {}
pub struct ItemRegistry {
    items: HashMap<i16, Arc<RegistryItem>>,
    recipe_solver: Solver,
}
pub enum ToolType {
    PICKAXE,
    CHESTPLATE,
    LEGGINGS,
    HELMET,
    BOOTS,
    AXE,
    SWORD,
}
impl ItemRegistry {
    pub fn global() -> &'static ItemRegistry {
        ITEM_REGISTRY.get().expect("Item registry is not initialized!")
    } 
    pub fn new() -> Self {
        log::info!("Initializing item registry");
        Self { items: HashMap::new(), recipe_solver: Solver::new() }
    }
    pub fn register_item(&mut self, id: i16, registry_name: &str, item: Box<dyn Item + Send + Sync>) {
        //log::info!("Registering item \"{}\" ({})", registry_name, id);
        self.items.insert(id, Arc::new(RegistryItem { name: registry_name.to_string(), item: Arc::new(item) }));
    }
    pub fn get_solver(&mut self) -> &mut Solver {
        &mut self.recipe_solver
    }
    pub fn get_solver_ref(&self) -> &Solver {
        &self.recipe_solver
    }
    pub fn get_item(&self, id: i16) -> Option<Arc<RegistryItem>> {
        Some(self.items.get(&id)?.clone())
    }
/*     pub fn get_block(&self, id: i16) -> Option<Box<dyn block::Block>> { 
        Some(Box::new(self.items.get(&id)?.get_item().as_block()?.clone()))
    } */
    pub fn get_item_name(&self, id: i8) -> Option<String> {
        for item in self.items.iter() {
            if *item.0 == id as i16 {
                return Some(item.1.name.clone());
            }
        }
        None
    }
    pub fn get_items(&self) -> &HashMap<i16, Arc<RegistryItem>> {
        &self.items
    }
/*     pub fn register_3x3_recipe(&mut self, recipe: [ItemStack; 9], output: ItemStack) {
        self.recipe3x3.insert(Recipe3X3 { recipe }, output);
    }
    pub fn register_2x2_recipe(&mut self, recipe: [ItemStack; 4], output: ItemStack) {
        self.recipe2x2.insert(Recipe2X2 { recipe }, output);
    }
    pub fn get_recipe_3x3(&self, recipe: [ItemStack; 9]) -> Option<ItemStack> {
        for (recipe_ot, out) in self.recipe3x3.iter() {
            if recipe_ot.recipe == recipe {
                //log::debug!("Recipe is correct!");
                return Some(out.clone());
            } else {
                //log::debug!("{:?} is not {:?}!", recipe, recipe_ot.recipe);
            }
        }
        None
        //Some(self.recipe3x3.get(&Recipe3X3 { recipe })?.clone())
    }
    pub fn get_recipe(&self, recipe: [ItemStack; 4]) -> Option<ItemStack> {
        Some(self.recipe2x2.get(&Recipe2X2 { recipe })?.clone())
    } */
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
    fn recipe(&self) -> Option<Recipe> {
        None
    }
    fn max_uses(&self) -> Option<u64> {
        None
    }
    fn damage(&self) -> Option<i16> {
        None
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
    pub fn get_name(&self) -> &str {
        &self.name
    }
}