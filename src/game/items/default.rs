use super::*;
use events::*;
use std::boxed::Box;
pub struct GrassBlock {}
impl block::Block for GrassBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(3, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(3, 0, 1))
    }
    fn hardness(&self) -> f32 {
        0.6
    }
}
pub struct DirtBlock {}
impl block::Block for DirtBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(3, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(3, 0, 1))
    }
    fn hardness(&self) -> f32 {
        0.5
    }
}
pub struct CobblestoneBlock {}
impl block::Block for CobblestoneBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(5, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(4, 0, 1))
    }
    fn hardness(&self) -> f32 {
        2.
    }
}
pub struct StoneBlock {}
impl block::Block for StoneBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(5, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        let registry = ItemRegistry::global();
        if let Some(item) = registry.get_item(tool.id) {
            if let Some(item) = item.get_item().get_tool_type() {
                match item {
                    ToolType::PICKAXE => {
                        return Some(ItemStack::new(4, 0, 1));
                    }
                    _ => {}
                }
            }
        }
        None
    }
    fn hardness(&self) -> f32 {
        1.5
    }
}
pub struct CraftingTableBlock {}
impl block::Block for CraftingTableBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(58, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(58, 0, 1))
    }
    fn on_right_click(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        let mut craftory = Inventory::default();
        for i in 0..10 {
            craftory.items.insert(i, ItemStack::default());
        }
        //log::info!("Lenga: {:?}", craftory.items.len());
        player.open_window(
            Window {
                inventory_type: 1,
                window_title: "Crafting".to_string(),
                inventory: Arc::new(RefCell::new(craftory)),
            },
            69,
        );
        false
    }
    fn hardness(&self) -> f32 {
        2.5
    }
}
pub struct TorchBlock {}
impl block::Block for TorchBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(50, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(50, 0, 1))
    }
    fn hardness(&self) -> f32 {
        2.
    }
    fn needs_align(&self) -> bool {
        true
    }
}
pub struct LogBlock {}
impl block::Block for LogBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(17, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(17, 0, 1))
    }
    fn hardness(&self) -> f32 {
        2.
    }
}
pub struct PlanksBlock {}
impl block::Block for PlanksBlock {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn get_block_drop(&self) -> Option<ItemStack> {
        Some(ItemStack::new(5, 0, 1))
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(5, 0, 1))
    }
    fn hardness(&self) -> f32 {
        2.
    }
}
pub struct GoldChestplateItem {}
impl Item for GoldChestplateItem {
    fn is_block(&self) -> bool {
        false
    }
    fn stack_size(&self) -> i16 {
        1
    }
    fn on_use(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_tool_type(&self) -> Option<ToolType> {
        Some(ToolType::CHESTPLATE)
    }
    fn wearable(&self) -> bool {
        true
    }
}
pub struct StickItem {}
impl Item for StickItem {
    fn is_block(&self) -> bool {
        false
    }
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_use(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}
pub struct GoldPickaxeItem {}
impl Item for GoldPickaxeItem {
    fn is_block(&self) -> bool {
        false
    }
    fn stack_size(&self) -> i16 {
        1
    }
    fn on_use(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_tool_type(&self) -> Option<ToolType> {
        Some(ToolType::PICKAXE)
    }
}
pub struct WoodPickaxeItem {}
impl Item for WoodPickaxeItem {
    fn is_block(&self) -> bool {
        false
    }
    fn stack_size(&self) -> i16 {
        1
    }
    fn on_use(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn get_tool_type(&self) -> Option<ToolType> {
        Some(ToolType::PICKAXE)
    }
}
pub fn init_items(registry: &mut ItemRegistry) {
    registry.register_item(3, "dirt_block", Box::new(DirtBlock {}));
    registry.register_item(5, "planks_block", Box::new(PlanksBlock {}));
    registry.register_item(4, "cobblestone_block", Box::new(CobblestoneBlock {}));
    registry.register_item(1, "stone_block", Box::new(StoneBlock {}));
    registry.register_item(285, "gold_pickaxe_item", Box::new(GoldPickaxeItem {}));
    registry.register_item(315, "gold_chestplate_item", Box::new(GoldChestplateItem {}));
    registry.register_item(58, "craftingtable_block", Box::new(CraftingTableBlock {}));
    registry.register_item(2, "grass_block", Box::new(GrassBlock {}));
    registry.register_item(280, "stick_item", Box::new(StickItem {}));
    registry.register_item(50, "torch_block", Box::new(TorchBlock {}));
    registry.register_item(17, "log_block", Box::new(LogBlock {}));
    registry.register_item(270, "wood_pickaxe_item", Box::new(WoodPickaxeItem {}));
    let plank = ItemStack::new(5, 0, 1);
    let mut arrvec = arrayvec::ArrayVec::new();
    arrvec.push(ItemStack::new(17, 0, 1));
    let shapeless = ShapelessRecipe::new(arrvec, ItemStack::new(5, 0, 4));
    registry.get_solver().register(Recipe::Shapeless(shapeless));
    // Crafting table
    let mut grid = Grid::default();
    grid[0][0] = Some(ItemStack::new(5, 0, 1));
    grid[1][0] = Some(ItemStack::new(5, 0, 1));
    grid[0][1] = Some(ItemStack::new(5, 0, 1));
    grid[1][1] = Some(ItemStack::new(5, 0, 1));
    registry.get_solver().register(Recipe::Shaped(ShapedRecipe {
        input: grid,
        output: ItemStack::new(58, 0, 1),
    }));
    // Stick
    let mut grid = Grid::default();
    grid[0][0] = Some(ItemStack::new(5, 0, 1));
    grid[0][1] = Some(ItemStack::new(5, 0, 1));
    registry.get_solver().register(Recipe::Shaped(ShapedRecipe {
        input: grid,
        output: ItemStack::new(280, 0, 4),
    }));
    // Wood pickaxe
    let mut grid = Grid::default();
    grid[0][0] = Some(ItemStack::new(5, 0, 1));
    grid[1][0] = Some(ItemStack::new(5, 0, 1));
    grid[2][0] = Some(ItemStack::new(5, 0, 1));
    grid[1][1] = Some(ItemStack::new(280, 0, 1));
    grid[1][2] = Some(ItemStack::new(280, 0, 1));
    crafting::normalize(&mut grid);
    log::info!("Recipe: {:?}", grid);
    registry.get_solver().register(Recipe::Shaped(ShapedRecipe {
        input: grid,
        output: ItemStack::new(270, 0, 1),
    }));
    /*     registry.register_2x2_recipe([ItemStack::new(5, 0, 1), ItemStack::new(5, 0, 1), ItemStack::new(5, 0, 1), ItemStack::new(5, 0, 1)], ItemStack::new(58, 0, 1));
    registry.register_2x2_recipe([ItemStack::default(), plank.clone(), ItemStack::default(), plank.clone()], ItemStack::new(280, 0, 4));
    registry.register_3x3_recipe([ItemStack::default(), plank.clone(), ItemStack::default(), ItemStack::default(), plank.clone(), ItemStack::default(), ItemStack::default(), ItemStack::default(), ItemStack::default()], ItemStack::new(280, 0, 4)); */
    //log::info!("[ItemRegistry] Registry initialized!");
}
