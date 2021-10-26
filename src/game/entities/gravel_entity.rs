use super::*;
use crate::network::packet::ServerPacket;
use std::default::Default;
pub struct GravelEntity {
    entity_id: EntityID,
    position: Position,
    ticks_spawned: u128,
    to_remove: bool,
    do_movement: bool,
}
impl GravelEntity {
    pub fn new(position: Position, ticks_spawned: u128) -> Self {
        Self {
            entity_id: EntityID::new(),
            position: position,
            ticks_spawned: ticks_spawned,
            to_remove: false,
            do_movement: false,
        }
    }
    pub fn destruct(self) {
        IDS.lock().unwrap().push(self.entity_id.0);
    }
}
impl Entity for GravelEntity {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn get_id(&self) -> EntityID {
        self.entity_id.clone()
    }
    fn destruct_entity(&self, mut player: &mut RefMut<'_, Player>) {
        player.write(ServerPacket::DestroyEntity {
            eid: self.entity_id.0,
        });
    }
    fn spawn_entity(&mut self, player: &mut RefMut<'_, Player>) {
        player.write(ServerPacket::AddObjectVehicle {
            eid: self.entity_id.0,
            obj_type: 71,
            x: (self.position.x * 32.0) as i32,
            y: (self.position.y * 32.0) as i32,
            z: (self.position.z * 32.0) as i32,
            unknown_flag: 0,
            unk_1: None,
            unk_2: None,
            unk_3: None,
        });
    }
    fn tick(&mut self, game: &mut Game) {
        let clone = self.position.y.floor() - 0.3;
        let blk = game.world.get_block(&BlockPosition::new(self.position.x.floor() as i32, clone as i32, self.position.z.floor() as i32));
        if blk.get_type() != 0 {
            if let Some(reg_blk) = ItemRegistry::global().get_item(blk.get_type() as i16) {
                if let Some(reg_blk) = reg_blk.get_item().as_block() {
                    if !reg_blk.is_solid() {
                        self.position.y -= 0.3;
                    } else {
                        game.entities.borrow_mut().remove(&self.entity_id);
                        game.world.get_block(&BlockPosition::new(self.position.x.floor() as i32, self.position.y.floor() as i32, self.position.z.floor() as i32)).set_type(13);
                    }
                }
            } else {
                game.entities.borrow_mut().remove(&self.entity_id);
            }
        } else {
            game.entities.borrow_mut().remove(&self.entity_id);
        }
    }
    fn get_position(&mut self) -> &mut Position {
        &mut self.position
    }
    fn broadcast_pos_change(&mut self) -> bool {
        false
    }
}
