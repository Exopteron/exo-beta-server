use rand::Rng;

use crate::item::{item::{BlockIdentifier, block::Block, ItemIdentifier}, stack::ItemStack};

pub mod wheat;

pub struct CropBlock(pub BlockIdentifier);

impl Block for CropBlock {
    fn hardness(&self) -> i32 {
        1
    }
    fn id(&self) -> BlockIdentifier {
        self.0
    }
    fn opacity(&self) -> u8 {
        0
    }

    fn item_stack_size(&self) -> i8 {
        0
    }
    fn dropped_items(&self, state: crate::world::chunks::BlockState, _held_item: crate::item::inventory_slot::InventorySlot) -> Vec<crate::item::stack::ItemStack> {
        let mut items = Vec::new();
        if state.b_metadata == 7 {
            items.push(ItemStack::new(296, 1, 0));
            items.push(ItemStack::new(295, 1, 0));
        } else {
            items.push(ItemStack::new(295, 1, 0));
        }
        items
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn opaque(&self) -> bool {
        false
    }
    fn tick(&self, world: i32, game: &mut crate::game::Game, mut state: crate::world::chunks::BlockState, position: crate::game::BlockPosition) {
        if state.b_metadata < 7 {
            let rate = 10.0;
            if rand::thread_rng().gen_range(0..(100. / rate) as usize) == 0 {
                state.b_metadata += 1;
                game.set_block(position, state, position.world);
            }
        }
    }
}
impl CropBlock {

}