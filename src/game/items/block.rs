use super::*;
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
    fn stack_size(&self) -> i16;
    fn on_place(&self, game: &mut Game, packet: &mut crate::network::packet::PlayerBlockPlacement, player: Arc<PlayerRef>);
    fn on_break(&self, game: &mut Game, player: std::cell::RefMut<'_, Player>, block: BlockPosition, packet: crate::network::packet::PlayerDigging, orig_type: i32) -> anyhow::Result<()> {
        let block = game.world.get_block(block.x, block.y, block.z).unwrap();
        block.set_type(0);
        game.block_updates.push(crate::game::Block {
            position: crate::game::BlockPosition {
                x: packet.x,
                y: (packet.y + 0) as i32,
                z: packet.z,
            },
            block: block.clone(),
        });
        log::info!("orig_type: {}", orig_type);
        game.broadcast_to_loaded(
            &player,
            ServerPacket::SoundEffect {
                effect_id: 2001,
                x: packet.x,
                y: packet.y,
                z: packet.z,
                sound_data: orig_type as i32,
            },
        )?;
        Ok(())
    }
}
impl<T> Item for T
where
    T: Block
{
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
                log::info!("Valid!");
                let mut hand = player.get_item_in_hand();
                hand.count -= 1;
                drop(hand);
                event_handler.cause_event(Box::new(BlockPlacementEvent {
                    cancelled: false,
                    packet,
                    player: player.clone(),
                }))?;
            }
        }
        Ok(())
    }
}