use super::*;
pub struct Internal {
    position: BlockPosition,
    is_open: bool,
}
pub struct TrapdoorTileEntity {
    internal: Option<Internal>,
}
impl std::default::Default for TrapdoorTileEntity {
    fn default() -> Self {
        Self { internal: None }
    }
}
impl BlockTileEntity for TrapdoorTileEntity {
    fn stack_size(&self) -> i16 {
        64
    }
    fn on_right_click(
        &mut self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        let internal = self.internal.as_mut().unwrap();
        internal.is_open ^= true;
        log::info!("Is open? {}", internal.is_open);
        for player in game.players.iter() {
            let player = player.1;
            if player.get_loaded_chunks().contains(&internal.position.to_chunk_coords()) {
                player.write_packet(ServerPacket::SoundEffect { effect_id: 1003, x: internal.position.x, y: internal.position.y as i8, z: internal.position.z, sound_data: 1 });
            }
        }
        if internal.is_open {
            game.world.get_block(&internal.position).set_meta(1);
        } else {
            game.world.get_block(&internal.position).set_meta(0);
        }
        true
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        true
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
        position: BlockPosition,
    ) -> Option<ItemStack> {
        Some(ItemStack::new(96, 0, 1))
    }
    fn hardness(&self) -> f32 {
        2.
    }
    fn get_position(&self) -> BlockPosition {
        self.internal.as_ref().unwrap().position
    }
    fn new_entity(&self, position: BlockPosition) -> Box<dyn BlockTileEntity> {
        Box::new(Self { internal: Some(Internal { position, is_open: false }) } )
    }
    fn tick(&mut self, game: &mut Game, position: BlockPosition) {
        //log::info!("being ticked");
    }
}