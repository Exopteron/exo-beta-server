use crate::item::item::block::Block;

pub struct VinesBlock;
impl Block for VinesBlock {
    fn id(&self) -> crate::item::item::BlockIdentifier {
        106
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(&self, game: &mut crate::game::Game, placer: hecs::Entity, item: crate::item::stack::ItemStack, mut position: crate::game::BlockPosition, face: crate::protocol::packets::Face, world: i32) -> Option<crate::events::block_interact::BlockPlacementEvent> {
        None
    }
    fn opacity(&self) -> u8 {
        0
    }
}