use super::*;
use crate::game::entities::item_entity::*;
pub trait Block {
    fn stack_size(&self) -> i16;
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool;
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
        position: BlockPosition,
    ) -> Option<ItemStack>;
    fn on_right_click(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        true
    }
    fn nearby_block_update(&self, game: &mut Game, from: BlockPosition, to: BlockPosition) {
        
    }
    fn opacity(&self) -> u64 {
        100
    }
    fn hardness(&self) -> f32;
    fn needs_align(&self) -> bool {
        false
    }
    fn is_fluid(&self) -> bool {
        false
    }
    fn is_solid(&self) -> bool {
        true
    }
    fn insta_break(&self) -> bool {
        false
    }
    fn random_tick(&self, game: &mut Game, block: BlockPosition) {
        //log::info!("I am grass, and I was ticked!");
    }
}
impl<T> Item for T
where
    T: Block,
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
        mut packet: crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> anyhow::Result<()> {
        let hand = player.get_item_in_hand().count.clone();
        if hand > 0 {
            let objects = game.objects.clone();
            let mut event_handler = objects
                .get_mut::<EventHandler>()
                .expect("No event handler?");
            if block_utils::place_validator(game, &packet) {
                if self.on_place(game, &mut packet, player.clone()) {
                    //log::info!("Valid!");
                    let mut hand = player.get_item_in_hand();
                    if hand.count - 1 > 0 {
                        hand.count -= 1;
                    } else {
                        hand.count = 0;
                        hand.id = 0;
                        hand.damage = 0;
                    }
                    log::debug!("Held item: {:?}", hand);
                    drop(hand);
                    player.held_item_changed(true);
                    event_handler.cause_event(Box::new(BlockPlacementEvent {
                        cancelled: false,
                        packet: packet.clone(),
                        player: player.clone(),
                        needs_align: self.needs_align(),
                    }))?;
                } else {
                    let mut packet2 = packet.clone();
                    match packet2.direction {
                        0 => {
                            packet2.y -= 1;
                        }
                        1 => {
                            //packet.y += 1;
                            packet2.y = match packet2.y.checked_add(1) {
                                Some(num) => num,
                                None => {
                                    return Ok(());
                                }
                            }
                        }
                        2 => {
                            packet2.z -= 1;
                        }
                        3 => {
                            packet2.z += 1;
                        }
                        4 => {
                            packet2.x -= 1;
                        }
                        5 => {
                            packet2.x += 1;
                        }
                        x => {
                            log::debug!("Fal {}", x);
                            return Ok(());
                        }
                    }
                    let block = game.world.get_block(&BlockPosition::new(packet2.x, packet2.y as i32, packet2.z));
                    player.write_packet(ServerPacket::BlockChange {
                        x: packet2.x,
                        y: packet2.y,
                        z: packet2.z,
                        block_type: block.get_type() as i8,
                        block_metadata: block.get_meta() as i8,
                    });
                }
            }
        }
        Ok(())
    }
}
