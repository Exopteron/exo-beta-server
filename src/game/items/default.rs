use super::*;
use events::*;
use std::boxed::Box;
mod material_uses {
    pub const WOOD: u64 = 59;
    pub const STONE: u64 = 131;
}
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
pub struct FenceBlock {}
impl block::Block for FenceBlock {
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
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(85, 0, 1))
    }
    fn hardness(&self) -> f32 {
        2.
    }
}
pub struct AirBlock {}
impl block::Block for AirBlock {
    fn stack_size(&self) -> i16 {
        0
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) {
        log::info!("Was used!");
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        //Some(ItemStack::new(85, 0, 1))
        None
    }
    fn hardness(&self) -> f32 {
        0.
    }
    fn is_solid(&self) -> bool {
        false
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
        if !player.is_crouching() {
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
            return false;
        }
        true
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
pub struct WaterBlock {}
impl block::Block for WaterBlock {
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
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        //Some(ItemStack::new(5, 0, 1))
        None
    }
    fn hardness(&self) -> f32 {
        2.
    }
    fn is_fluid(&self) -> bool {
        true
    }
    fn is_solid(&self) -> bool {
        false
    }
}
pub struct GlassBlock {}
impl block::Block for GlassBlock {
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
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
    ) -> Option<ItemStack> {
        //Some(ItemStack::new(5, 0, 1))
        None
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
pub struct CobblestonePickaxeItem {}
impl Item for CobblestonePickaxeItem {
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
    fn recipe(&self) -> Option<Recipe> {
        let mut grid = Grid::default();
        grid[0][0] = Some(ItemStack::new(4, 0, 1));
        grid[1][0] = Some(ItemStack::new(4, 0, 1));
        grid[2][0] = Some(ItemStack::new(4, 0, 1));
        grid[1][1] = Some(ItemStack::new(280, 0, 1));
        grid[1][2] = Some(ItemStack::new(280, 0, 1));
        crafting::normalize(&mut grid);
        Some(Recipe::Shaped(ShapedRecipe {
            input: grid,
            output: ItemStack::new(274, 0, 1),
        }))
    }
    fn max_uses(&self) -> Option<u64> {
        Some(material_uses::STONE)
    }
}
pub struct WoodAxeItem {}
impl Item for WoodAxeItem {
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
        Some(ToolType::AXE)
    }
    fn recipe(&self) -> Option<Recipe> {
        let mut grid = Grid::default();
        grid[0][1] = Some(ItemStack::new(5, 0, 1));
        grid[0][0] = Some(ItemStack::new(5, 0, 1));
        grid[1][0] = Some(ItemStack::new(5, 0, 1));
        grid[1][1] = Some(ItemStack::new(280, 0, 1));
        grid[1][2] = Some(ItemStack::new(280, 0, 1));
        crafting::normalize(&mut grid);
        Some(Recipe::Shaped(ShapedRecipe {
            input: grid,
            output: ItemStack::new(271, 0, 1),
        }))
    }
    fn max_uses(&self) -> Option<u64> {
        Some(material_uses::WOOD)
    }
}
pub struct StoneAxeItem {}
impl Item for StoneAxeItem {
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
        Some(ToolType::AXE)
    }
    fn recipe(&self) -> Option<Recipe> {
        let mut grid = Grid::default();
        grid[0][0] = Some(ItemStack::new(4, 0, 1));
        grid[1][0] = Some(ItemStack::new(4, 0, 1));
        grid[2][0] = Some(ItemStack::new(4, 0, 1));
        grid[1][1] = Some(ItemStack::new(280, 0, 1));
        grid[1][2] = Some(ItemStack::new(280, 0, 1));
        crafting::normalize(&mut grid);
        Some(Recipe::Shaped(ShapedRecipe {
            input: grid,
            output: ItemStack::new(275, 0, 1),
        }))
    }
    fn max_uses(&self) -> Option<u64> {
        Some(material_uses::STONE)
    }
}
pub struct WoodSwordItem {}
impl Item for WoodSwordItem {
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
        Some(ToolType::SWORD)
    }
    fn recipe(&self) -> Option<Recipe> {
        let mut grid = Grid::default();
        //grid[0][0] = Some(ItemStack::new(5, 0, 1));
        grid[1][0] = Some(ItemStack::new(5, 0, 1));
        //grid[2][0] = Some(ItemStack::new(5, 0, 1));
        grid[1][1] = Some(ItemStack::new(5, 0, 1));
        grid[1][2] = Some(ItemStack::new(280, 0, 1));
        crafting::normalize(&mut grid);
        Some(Recipe::Shaped(ShapedRecipe {
            input: grid,
            output: ItemStack::new(268, 0, 1),
        }))
    }
    fn max_uses(&self) -> Option<u64> {
        Some(material_uses::WOOD)
    }
    fn damage(&self) -> Option<i16> {
        Some(5)
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
    fn recipe(&self) -> Option<Recipe> {
        let mut grid = Grid::default();
        grid[0][0] = Some(ItemStack::new(5, 0, 1));
        grid[1][0] = Some(ItemStack::new(5, 0, 1));
        grid[2][0] = Some(ItemStack::new(5, 0, 1));
        grid[1][1] = Some(ItemStack::new(280, 0, 1));
        grid[1][2] = Some(ItemStack::new(280, 0, 1));
        crafting::normalize(&mut grid);
        Some(Recipe::Shaped(ShapedRecipe {
            input: grid,
            output: ItemStack::new(270, 0, 1),
        }))
    }
    fn max_uses(&self) -> Option<u64> {
        Some(material_uses::WOOD)
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
    registry.register_item(274, "stone_pickaxe_item", Box::new(CobblestonePickaxeItem {}));
    registry.register_item(271, "wood_axe_item", Box::new(WoodAxeItem {}));
    registry.register_item(275, "stone_axe_item", Box::new(StoneAxeItem {}));
    registry.register_item(9, "water_block", Box::new(WaterBlock {}));
    registry.register_item(20, "glass_block", Box::new(GlassBlock {}));
    registry.register_item(268, "wood_sword_item", Box::new(WoodSwordItem {}));
    registry.register_item(85, "fence_block", Box::new(FenceBlock {}));
    registry.register_item(0, "air", Box::new(AirBlock {}));
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
/*     // Wood pickaxe
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
    */
    for (_, item) in registry.items.clone().iter() {
        if let Some(recipe) = item.get_item().recipe() {
            registry.get_solver().register(recipe);
        }
    }
    /*     registry.register_2x2_recipe([ItemStack::new(5, 0, 1), ItemStack::new(5, 0, 1), ItemStack::new(5, 0, 1), ItemStack::new(5, 0, 1)], ItemStack::new(58, 0, 1));
    registry.register_2x2_recipe([ItemStack::default(), plank.clone(), ItemStack::default(), plank.clone()], ItemStack::new(280, 0, 4));
    registry.register_3x3_recipe([ItemStack::default(), plank.clone(), ItemStack::default(), ItemStack::default(), plank.clone(), ItemStack::default(), ItemStack::default(), ItemStack::default(), ItemStack::default()], ItemStack::new(280, 0, 4)); */
    //log::info!("[ItemRegistry] Registry initialized!");
}
