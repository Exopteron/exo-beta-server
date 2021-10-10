use super::*;
use crate::network::packet::ServerPacket;
use std::default::Default;
pub struct ItemEntity {
    entity_id: EntityID,
    position: Position,
    ticks_spawned: u128,
    item: ItemStack,
    to_remove: bool,
    velocity: Option<[f64; 3]>,
    do_movement: bool,
}
impl ItemEntity {
    pub fn new(position: Position, ticks_spawned: u128, item: ItemStack, velocity: Option<[f64; 3]>) -> Self {
        Self {
            entity_id: EntityID::new(),
            position: position,
            ticks_spawned: ticks_spawned,
            item: item,
            to_remove: false,
            velocity: velocity,
            do_movement: false,
        }
    }
    pub fn destruct(self) {
        IDS.lock().unwrap().push(self.entity_id.0);
    }
}
impl Entity for ItemEntity {
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
        player.write(ServerPacket::PickupSpawn {
            eid: self.entity_id.0,
            item: self.item.id,
            count: self.item.count,
            damage: self.item.damage,
            x: (self.position.x * 32.0) as i32,
            y: (self.position.y * 32.0) as i32,
            z: (self.position.z * 32.0) as i32,
            rotation: self.position.yaw as i8,
            pitch: self.position.pitch as i8,
            roll: 0,
        });
    }
    fn tick(&mut self, game: &mut Game) {
        if self.velocity.is_some() {
/*             let velocity = self.velocity.unwrap();
            log::info!("we have velocity! {:?}", velocity);
            self.position.x += velocity[0] as f64;
            self.position.y += velocity[1] as f64;
            self.position.z += velocity[2] as f64;
            self.velocity = None;
            self.do_movement = true; */
        }
        if self.to_remove {
            game.entities.borrow_mut().remove(&self.entity_id);
            return;
        }
        log::debug!("I am being ticked! I'm at {:?}", self.position);
        if game.ticks - self.ticks_spawned > 600 {
            game.entities.borrow_mut().remove(&self.entity_id);
        }
        let clone = self.position.y - 0.1;
        if let Some(blk) = game.world.get_block(self.position.x as i32, clone as i32, self.position.z as i32) {
            if blk.get_type() == 0 {
                self.position.y -= 0.1;
            }
        }
        use crate::game::entities::*;
        if game.ticks - self.ticks_spawned > 5 {
            let entities = game.entities.borrow().clone();
            for entity in entities {
                if entity.0 == self.entity_id {
                    continue;
                }
                if self.position.distance(&entity.1.borrow_mut().get_position()) < 1.5 {
                    let mut entity = entity.1.borrow_mut();
                    if let Some(entity) = entity.as_any().downcast_mut::<ItemEntity>() {
                        if entity.item.id == self.item.id && entity.item.damage == self.item.damage {
                            let registry = ItemRegistry::global();
                            let stack_size = registry.get_item(self.item.id).expect("Fix later").get_item().stack_size();
                            if (entity.item.count as u64 + self.item.count as u64) < stack_size as u64 {
                                entity.item.count += self.item.count;
                                game.entities.borrow_mut().remove(&self.entity_id);
                                return;
                            }
                        }
                    }
                }
            }
            let players = game.players.0.borrow().clone();
            for player in players {
                if self.position.distance(&player.1.get_position()) < 1.5 && !player.1.is_dead() {
                    self.to_remove = true;
                    let plr_id = player.1.get_id();
                    let packet = ServerPacket::CollectItem { collected_eid: self.entity_id.0, collector_eid: plr_id.0};
                    game.broadcast_to_loaded(&player.1.unwrap().unwrap(), packet.clone()).expect("Couldn't broadcast");
                    player.1.write_packet(packet);
                    let mut inv = player.1.get_inventory();
                    'main: for (num, slot) in &mut inv.items {
                        if num > &8 {
                            if slot.id == self.item.id && slot.damage == self.item.damage {
                                let registry = ItemRegistry::global();
                                let mut our_count = self.item.count;
                                if let Some(item) = registry.get_item(slot.id) {
                                    for _ in 0..self.item.count {
                                        if slot.count + our_count > item.get_item().stack_size() as i8 {
                                            continue 'main;
                                        }
                                        our_count -= 1;
                                        slot.count += 1;
                                    }
                                }
                                //slot.count += self.item.count;
                                return;
                            }
                        }
                    }
                    for (num, slot) in &mut inv.items {
                        if num > &8 {
                            if slot.id == 0 {
                                slot.id = self.item.id;
                                slot.damage = self.item.damage;
                                slot.count = self.item.count;
                                return;
                            }
                        }
                    }
                }
            }
        }
    }
    fn get_position(&mut self) -> &mut Position {
        &mut self.position
    }
    fn broadcast_pos_change(&mut self) -> bool {
        if self.do_movement {
            self.do_movement = false;
            return true;
        }
        false
    }
}
