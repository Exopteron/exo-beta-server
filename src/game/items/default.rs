use super::*;
use events::*;
use std::boxed::Box;
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
}
pub fn init_items(registry: &mut ItemRegistry) {
    registry.register_item(3, "dirt_block", Box::new(DirtBlock {}));
}
