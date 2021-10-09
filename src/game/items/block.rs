use super::*;
use crate::game::entities::item_entity::*;
pub trait AsBlock {
    fn as_block(&self) -> Option<&dyn Block>;
}
impl<T> AsBlock for T {
    default fn as_block(&self) -> Option<&dyn Block> { None }
} 
impl<T> AsBlock for T where T: Block {
    fn as_block(&self) -> Option<&dyn Block> { Some(self) }
}
pub trait Block {
    fn get_block_drop(&self) -> Option<ItemStack>;
    fn stack_size(&self) -> i16;
    fn on_place(&self, game: &mut Game, packet: &mut crate::network::packet::PlayerBlockPlacement, player: Arc<PlayerRef>);
    fn on_break(&self, game: &mut Game, packet: crate::network::packet::PlayerDigging, player: std::cell::RefMut<'_, Player>, tool: ItemStack) -> Option<ItemStack>;
    fn on_right_click(&self, game: &mut Game, packet: &mut crate::network::packet::PlayerBlockPlacement, player: Arc<PlayerRef>) -> bool { true }
}
impl<T> Item for T
where
    T: Block
{
    fn as_block(&self) -> Option<&dyn Block> {
        Some(self)
    }
    fn stack_size(&self) -> i16 {
        self.stack_size()
    }
    fn is_block(&self) -> bool {
        true
    }
    fn on_use(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> anyhow::Result<()> {
        let hand = player.get_item_in_hand().count.clone();
        if hand > 0 {
            let objects = game.objects.clone();
            let mut event_handler = objects
                .get_mut::<EventHandler>()
                .expect("No event handler?");
            if block_utils::place_validator(game, &packet) {
                //log::info!("Valid!");
                let mut hand = player.get_item_in_hand();
                hand.count -= 1;
                drop(hand);
                event_handler.cause_event(Box::new(BlockPlacementEvent {
                    cancelled: false,
                    packet: packet.clone(),
                    player: player.clone(),
                }))?;
            }
        }
        Ok(())
    }
}