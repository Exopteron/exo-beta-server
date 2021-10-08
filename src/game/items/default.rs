use super::*;
use std::boxed::Box;
use events::*;
pub struct DirtBlock {}
impl Item for DirtBlock {
    fn is_block(&self) -> bool {
        true
    }
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_use(&self, game: &mut Game, packet: crate::network::packet::PlayerBlockPlacement, player: Arc<PlayerRef>) -> anyhow::Result<()> {
        log::info!("Was used!");
        let objects = game.objects.clone();
        let mut event_handler = objects.get_mut::<EventHandler>().expect("No event handler?");
        event_handler.cause_event(Box::new(BlockPlacementEvent { cancelled: false, packet, player: player.clone() }))?;
        Ok(())
    }
}
pub fn init_items(registry: &mut ItemRegistry) {
    registry.register_item(3, "dirt_block", Box::new(DirtBlock {}));
}