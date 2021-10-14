use super::*;
pub mod trapdoor_tile_entity;
use crate::game::items::block::*;
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};
pub trait BlockTileEntity {
    fn new_entity(&self, position: BlockPosition) -> Box<dyn BlockTileEntity>;
    fn get_position(&self) -> BlockPosition;
    fn tick(&mut self, game: &mut Game, position: BlockPosition);
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
        position: BlockPosition,
    ) -> Option<ItemStack>;
    fn hardness(&self) -> f32;
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool;
    fn stack_size(&self) -> i16;
    fn on_right_click(
        &mut self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        true
    }
    fn is_solid(&self) -> bool {
        true
    }
}
impl<T> crate::game::items::block::Block for T
where
    T: BlockTileEntity,
{
    fn is_solid(&self) -> bool {
        BlockTileEntity::is_solid(self)
    }
    fn on_right_click(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        let mut tile_entities = game.tile_entities.borrow().clone();
        let pos = BlockPosition { x: packet.x, y: packet.y as i32, z: packet.z };
        if let None = tile_entities.get(&pos) {
            let mut tile_entities_2 = game.tile_entities.borrow_mut();
            tile_entities_2.insert(pos.clone(), Arc::new(RefCell::new(self.new_entity(pos))));
            tile_entities = tile_entities_2.clone();
        }
        let mut entity = tile_entities.get(&BlockPosition { x: packet.x, y: packet.y as i32, z: packet.z }).expect("Shouldn't be possible").borrow_mut();
        BlockTileEntity::on_right_click(&mut **entity, game, packet, player)
    }
    fn stack_size(&self) -> i16 {
        BlockTileEntity::stack_size(self)
    }
    fn on_place(
        &self,
        game: &mut Game,
        packet: &mut crate::network::packet::PlayerBlockPlacement,
        player: Arc<PlayerRef>,
    ) -> bool {
        log::info!("Yo! Was called");
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
                        return false;
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
                return false;
            }
        }
        let pos = BlockPosition { x: packet2.x, y: packet2.y as i32, z: packet2.z };
        game.tile_entities.borrow_mut().insert(pos.clone(), Arc::new(RefCell::new(BlockTileEntity::new_entity(self, pos))));
        BlockTileEntity::on_place(self, game, packet, player)
    }
    fn on_break(
        &self,
        game: &mut Game,
        packet: crate::network::packet::PlayerDigging,
        player: std::cell::RefMut<'_, Player>,
        tool: ItemStack,
        position: BlockPosition,
    ) -> Option<ItemStack> {
        game.tile_entities.borrow_mut().remove(&position);
        BlockTileEntity::on_break(self, game, packet, player, tool, position)
    }
    fn hardness(&self) -> f32 {
        BlockTileEntity::hardness(self)
    }
}
pub fn init_items(registry: &mut ItemRegistry) {
    registry.register_item(96, "trapdoor_block", Box::new(trapdoor_tile_entity::TrapdoorTileEntity::default()));
}