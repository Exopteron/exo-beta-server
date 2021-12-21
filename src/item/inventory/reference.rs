#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Area {
    Storage,
    CraftingOutput,
    CraftingInput,
    Helmet,
    Chestplate,
    Leggings,
    Boots,
    Hotbar,
    FurnaceIngredient,
    FurnaceFuel,
    FurnaceOutput,
}
#[derive(Debug, Clone)]
pub enum BackingWindow {
    Player {
        player: super::Inventory,
    },
    Generic9x1 {
        block: super::Inventory,
        player: super::Inventory,
    },
    Generic9x2 {
        block: super::Inventory,
        player: super::Inventory,
    },
    Generic9x3 {
        block: super::Inventory,
        player: super::Inventory,
    },
    Generic9x4 {
        block: super::Inventory,
        player: super::Inventory,
    },
    Generic9x5 {
        block: super::Inventory,
        player: super::Inventory,
    },
    Generic9x6 {
        left_chest: super::Inventory,
        right_chest: super::Inventory,
        player: super::Inventory,
    },
    Generic3x3 {
        block: super::Inventory,
        player: super::Inventory,
    },
    Crafting {
        crafting_table: super::Inventory,
        player: super::Inventory,
    },
    Furnace {
        furnace: super::Inventory,
        player: super::Inventory,
    }
}
impl BackingWindow {
    #[allow(unused_comparisons)]
    pub fn index_to_slot(&self, index: usize) -> Option<(&super::Inventory, Area, usize)> {
        match self {
            BackingWindow::Player { player } => {
                if (0..1).contains(&index) {
                    let area = Area::CraftingOutput;
                    let slot = index;
                    Some((player, area, slot))
                } else if (1..5).contains(&index) {
                    let area = Area::CraftingInput;
                    let slot = index - 1;
                    Some((player, area, slot))
                } else if (5..6).contains(&index) {
                    let area = Area::Helmet;
                    let slot = index - 5;
                    Some((player, area, slot))
                } else if (6..7).contains(&index) {
                    let area = Area::Chestplate;
                    let slot = index - 6;
                    Some((player, area, slot))
                } else if (7..8).contains(&index) {
                    let area = Area::Leggings;
                    let slot = index - 7;
                    Some((player, area, slot))
                } else if (8..9).contains(&index) {
                    let area = Area::Boots;
                    let slot = index - 8;
                    Some((player, area, slot))
                } else if (9..36).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 9;
                    Some((player, area, slot))
                } else if (36..45).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 36;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic9x1 { block, player } => {
                if (0..9).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((block, area, slot))
                } else if (9..36).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 9;
                    Some((player, area, slot))
                } else if (36..45).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 36;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic9x2 { block, player } => {
                if (0..18).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((block, area, slot))
                } else if (18..45).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 18;
                    Some((player, area, slot))
                } else if (45..54).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 45;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic9x3 { block, player } => {
                if (0..27).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((block, area, slot))
                } else if (27..54).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 27;
                    Some((player, area, slot))
                } else if (54..63).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 54;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic9x4 { block, player } => {
                if (0..36).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((block, area, slot))
                } else if (36..63).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 36;
                    Some((player, area, slot))
                } else if (63..72).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 63;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic9x5 { block, player } => {
                if (0..45).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((block, area, slot))
                } else if (45..72).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 45;
                    Some((player, area, slot))
                } else if (72..81).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 72;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic9x6 {
                left_chest,
                right_chest,
                player,
            } => {
                if (0..27).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((left_chest, area, slot))
                } else if (27..54).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 27;
                    Some((right_chest, area, slot))
                } else if (54..81).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 54;
                    Some((player, area, slot))
                } else if (81..90).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 81;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Generic3x3 { block, player } => {
                if (0..9).contains(&index) {
                    let area = Area::Storage;
                    let slot = index;
                    Some((block, area, slot))
                } else if (9..36).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 9;
                    Some((player, area, slot))
                } else if (36..45).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 36;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Crafting {
                crafting_table,
                player,
            } => {
                if (0..1).contains(&index) {
                    let area = Area::CraftingOutput;
                    let slot = index;
                    Some((crafting_table, area, slot))
                } else if (1..10).contains(&index) {
                    let area = Area::CraftingInput;
                    let slot = index - 1;
                    Some((crafting_table, area, slot))
                } else if (10..37).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 10;
                    Some((player, area, slot))
                } else if (37..46).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 37;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
            BackingWindow::Furnace { furnace, player } => {
                if (0..1).contains(&index) {
                    let area = Area::FurnaceIngredient;
                    let slot = index;
                    Some((furnace, area, slot))
                } else if (1..2).contains(&index) {
                    let area = Area::FurnaceFuel;
                    let slot = index - 1;
                    Some((furnace, area, slot))
                } else if (2..3).contains(&index) {
                    let area = Area::FurnaceOutput;
                    let slot = index - 2;
                    Some((furnace, area, slot))
                } else if (3..30).contains(&index) {
                    let area = Area::Storage;
                    let slot = index - 3;
                    Some((player, area, slot))
                } else if (30..39).contains(&index) {
                    let area = Area::Hotbar;
                    let slot = index - 30;
                    Some((player, area, slot))
                } else {
                    None
                }
            }
        }
    }
    pub fn slot_to_index(
        &self,
        inventory: &super::Inventory,
        area: Area,
        slot: usize,
    ) -> Option<usize> {
        match self {
            BackingWindow::Player { player } => {
                if area == Area::CraftingOutput && player.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::CraftingInput && player.ptr_eq(inventory) {
                    Some(slot + 1)
                } else if area == Area::Helmet && player.ptr_eq(inventory) {
                    Some(slot + 5)
                } else if area == Area::Chestplate && player.ptr_eq(inventory) {
                    Some(slot + 6)
                } else if area == Area::Leggings && player.ptr_eq(inventory) {
                    Some(slot + 7)
                } else if area == Area::Boots && player.ptr_eq(inventory) {
                    Some(slot + 8)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 9)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 36)
                } else {
                    None
                }
            }
            BackingWindow::Generic9x1 { block, player } => {
                if area == Area::Storage && block.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 9)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 36)
                } else {
                    None
                }
            }
            BackingWindow::Generic9x2 { block, player } => {
                if area == Area::Storage && block.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 18)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 45)
                } else {
                    None
                }
            }
            BackingWindow::Generic9x3 { block, player } => {
                if area == Area::Storage && block.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 27)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 54)
                } else {
                    None
                }
            }
            BackingWindow::Generic9x4 { block, player } => {
                if area == Area::Storage && block.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 36)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 63)
                } else {
                    None
                }
            }
            BackingWindow::Generic9x5 { block, player } => {
                if area == Area::Storage && block.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 45)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 72)
                } else {
                    None
                }
            }
            BackingWindow::Generic9x6 {
                left_chest,
                right_chest,
                player,
            } => {
                if area == Area::Storage && left_chest.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && right_chest.ptr_eq(inventory) {
                    Some(slot + 27)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 54)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 81)
                } else {
                    None
                }
            }
            BackingWindow::Generic3x3 { block, player } => {
                if area == Area::Storage && block.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 9)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 36)
                } else {
                    None
                }
            }
            BackingWindow::Crafting {
                crafting_table,
                player,
            } => {
                if area == Area::CraftingOutput && crafting_table.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::CraftingInput && crafting_table.ptr_eq(inventory) {
                    Some(slot + 1)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 10)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 37)
                } else {
                    None
                }
            }
            BackingWindow::Furnace { furnace, player } => {
                if area == Area::FurnaceIngredient && furnace.ptr_eq(inventory) {
                    Some(slot)
                } else if area == Area::FurnaceFuel && furnace.ptr_eq(inventory) {
                    Some(slot + 1)
                } else if area == Area::FurnaceOutput && furnace.ptr_eq(inventory) {
                    Some(slot + 2)
                } else if area == Area::Storage && player.ptr_eq(inventory) {
                    Some(slot + 3)
                } else if area == Area::Hotbar && player.ptr_eq(inventory) {
                    Some(slot + 30)
                } else {
                    None
                }
            }
        }
    }
}
#[allow(warnings)]
#[allow(clippy::all)]
impl BackingWindow {
    /// Returns the `name` property of this `Window`.
    pub fn name(&self) -> &'static str {
        match self {
            BackingWindow::Player { .. } => "player",
            BackingWindow::Generic9x1 { .. } => "generic_9x1",
            BackingWindow::Generic9x2 { .. } => "generic_9x2",
            BackingWindow::Generic9x3 { .. } => "generic_9x3",
            BackingWindow::Generic9x4 { .. } => "generic_9x4",
            BackingWindow::Generic9x5 { .. } => "generic_9x5",
            BackingWindow::Generic9x6 { .. } => "generic_9x6",
            BackingWindow::Generic3x3 { .. } => "generic_3x3",
            BackingWindow::Crafting { .. } => "crafting",
            BackingWindow::Furnace { .. } => "furnace",
        }
    }
}

#[derive(Debug, Clone)]
pub enum InventoryBacking<T> {
    Player {
        crafting_input: [T; 4],
        crafting_output: [T; 1],
        helmet: [T; 1],
        chestplate: [T; 1],
        leggings: [T; 1],
        boots: [T; 1],
        storage: [T; 27],
        hotbar: [T; 9],
    },
    Chest {
        storage: [T; 27],
    },
    CraftingTable {
        crafting_input: [T; 9],
        crafting_output: [T; 1],
    },
    Furnace {
        furnace_ingredient: [T; 1],
        furnace_fuel: [T; 1],
        furnace_output: [T; 1],
    },
}
impl<T> InventoryBacking<T> {
    pub fn area_slice(&self, area: Area) -> Option<&[T]> {
        match self {
            InventoryBacking::Player {
                crafting_input,
                crafting_output,
                helmet,
                chestplate,
                leggings,
                boots,
                storage,
                hotbar,
            } => match area {
                Area::CraftingInput => Some(crafting_input.as_ref()),
                Area::CraftingOutput => Some(crafting_output.as_ref()),
                Area::Helmet => Some(helmet.as_ref()),
                Area::Chestplate => Some(chestplate.as_ref()),
                Area::Leggings => Some(leggings.as_ref()),
                Area::Boots => Some(boots.as_ref()),
                Area::Storage => Some(storage.as_ref()),
                Area::Hotbar => Some(hotbar.as_ref()),
                _ => None,
            },
            InventoryBacking::Chest { storage } => match area {
                Area::Storage => Some(storage.as_ref()),
                _ => None,
            },
            InventoryBacking::CraftingTable {
                crafting_input,
                crafting_output,
            } => match area {
                Area::CraftingInput => Some(crafting_input.as_ref()),
                Area::CraftingOutput => Some(crafting_output.as_ref()),
                _ => None,
            },
            InventoryBacking::Furnace {
                furnace_ingredient,
                furnace_fuel,
                furnace_output,
            } => match area {
                Area::FurnaceIngredient => Some(furnace_ingredient.as_ref()),
                Area::FurnaceFuel => Some(furnace_fuel.as_ref()),
                Area::FurnaceOutput => Some(furnace_output.as_ref()),
                _ => None,
            },
        }
    }
    pub fn areas(&self) -> &'static [Area] {
        match self {
            InventoryBacking::Player { .. } => {
                static AREAS: [Area; 8] = [
                    Area::CraftingInput,
                    Area::CraftingOutput,
                    Area::Helmet,
                    Area::Chestplate,
                    Area::Leggings,
                    Area::Boots,
                    Area::Storage,
                    Area::Hotbar,
                ];
                &AREAS
            }
            InventoryBacking::Chest { .. } => {
                static AREAS: [Area; 1] = [Area::Storage];
                &AREAS
            }
            InventoryBacking::CraftingTable { .. } => {
                static AREAS: [Area; 2] = [Area::CraftingInput, Area::CraftingOutput];
                &AREAS
            }
            InventoryBacking::Furnace { .. } => {
                static AREAS: [Area; 3] = [
                    Area::FurnaceIngredient,
                    Area::FurnaceFuel,
                    Area::FurnaceOutput,
                ];
                &AREAS
            }
        }
    }
    pub fn player() -> Self
    where
        T: Default,
    {
        InventoryBacking::Player {
            crafting_input: Default::default(),
            crafting_output: Default::default(),
            helmet: Default::default(),
            chestplate: Default::default(),
            leggings: Default::default(),
            boots: Default::default(),
            storage: Default::default(),
            hotbar: Default::default(),
        }
    }
    pub fn chest() -> Self
    where
        T: Default,
    {
        InventoryBacking::Chest {
            storage: Default::default(),
        }
    }
    pub fn crafting_table() -> Self
    where
        T: Default,
    {
        InventoryBacking::CraftingTable {
            crafting_input: Default::default(),
            crafting_output: Default::default(),
        }
    }
    pub fn furnace() -> Self
    where
        T: Default,
    {
        InventoryBacking::Furnace {
            furnace_ingredient: Default::default(),
            furnace_fuel: Default::default(),
            furnace_output: Default::default(),
        }
    }
}
impl super::Inventory {
    pub fn player() -> Self {
        Self {
            backing: std::sync::Arc::new(InventoryBacking::player()),
        }
    }
    pub fn chest() -> Self {
        Self {
            backing: std::sync::Arc::new(InventoryBacking::chest()),
        }
    }
    pub fn crafting_table() -> Self {
        Self {
            backing: std::sync::Arc::new(InventoryBacking::crafting_table()),
        }
    }
    pub fn furnace() -> Self {
        Self {
            backing: std::sync::Arc::new(InventoryBacking::furnace()),
        }
    }
}
