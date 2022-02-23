use crate::{item::{item::{BlockIdentifier, block::Block}, inventory_slot::InventorySlot, stack::{ItemStack, ItemStackType}}, world::chunks::BlockState};

use super::tools::ToolMaterials;

pub struct OreBlock(pub BlockIdentifier, pub ToolMaterials, pub fn() -> ItemStack);
impl Block for OreBlock {
    fn id(&self) -> BlockIdentifier {
        self.0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }

    fn dropped_items(&self, _state: BlockState, held_item: InventorySlot) -> Vec<ItemStack> {
        let mut items = Vec::new();
        if let InventorySlot::Filled(item) = held_item {
            match item.item() {
                ItemStackType::Item(i) => {
                    if let Some(t) = i.tool_type() {
                        if t >= self.1 {
                            items.push(self.2());   
                        }
                    }  
                },
                ItemStackType::Block(_) => (),
            }
        }
        items
    }
}