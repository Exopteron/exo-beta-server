use super::*;
use crate::network::packet::ServerPacket;
use std::default::Default;
pub struct SlimeEntity {
    entity_id: EntityID,
    position: Position,
    ticks_spawned: u128,
    to_remove: bool,
    do_movement: bool,
    health: i16,
    dead: bool,
    despawn_at: Option<u128>,
    last_move: u128,
    last_hit: u128,
}
impl SlimeEntity {
    pub fn new(position: Position, ticks_spawned: u128) -> Self {
        Self {
            entity_id: EntityID::new(),
            position: position,
            ticks_spawned: ticks_spawned,
            to_remove: false,
            do_movement: false,
            health: 10,
            dead: false,
            despawn_at: None,
            last_move: 0,
            last_hit: 0,
        }
    }
    pub fn destruct(self) {
        IDS.lock().unwrap().push(self.entity_id.0);
    }
}
impl Entity for SlimeEntity {
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
    fn is_dead(&self) -> bool {
        self.dead
    }
    fn spawn_entity(&mut self, player: &mut RefMut<'_, Player>) {
        player.write(ServerPacket::MobSpawn {
            eid: self.entity_id.0,
            m_type: 55,
            x: (self.position.x * 32.0) as i32,
            y: (self.position.y * 32.0) as i32,
            z: (self.position.z * 32.0) as i32,
            yaw: self.position.yaw as i8,
            pitch: self.position.pitch as i8,
        });
    }
    fn add_velocity(&mut self, velocity: [f64; 3]) {
        //self.position.x += velocity[0];
        //self.position.y += velocity[1];
        //self.position.z += velocity[2];
    }
    fn damage(&mut self, game: &mut Game, amount: i16) {
        //log::info!("I am being damaged!");
        for player in game.players.0.borrow().clone().iter() {
            if player.1.unwrap().unwrap().rendered_entities.get(&self.entity_id).is_some() {
                player.1.write_packet(ServerPacket::Animation {
                    eid: self.entity_id.0,
                    animate: 2,
                });
                player.1.write_packet(ServerPacket::EntityStatus {
                    eid: self.entity_id.0,
                    entity_status: 2,
                });
            }
        }
        self.health -= amount;
    }
    fn tick(&mut self, game: &mut Game) {
        if let Some(despawn_at) = self.despawn_at {
            if game.ticks >= despawn_at {
                game.entities.borrow_mut().remove(&self.entity_id);
            }
        } else {
            let mut closest_position: Option<Position> = None;
            for player in game.players.0.borrow().clone().iter() {
                let player = player.1;
                if closest_position.is_none() {
                    closest_position = Some(player.get_position());
                    continue;
                }
                if self.position.distance(&player.get_position()) < self.position.distance(&closest_position.unwrap()) {
                    closest_position = Some(player.get_position());
                }
                if self.position.distance(&player.get_position()) < 2. {
                    if !player.is_dead() {
                        if game.ticks > self.last_hit + 15 {
                            player.damage(DamageType::Mob { damager: "Slime".to_string()}, 3, None );
                            self.last_hit = game.ticks;
                        }
                    } else {
                        log::info!("Dead.");
                    }
                }
            }
            if self.last_move + 15 < game.ticks {
                self.position.move_towards(&closest_position.unwrap(), 0.5);
                self.last_move = game.ticks;
            }
            //self.position.y += 1.0;
            //log::info!("Going up!");
            if game.ticks - self.ticks_spawned > 600 {
                game.entities.borrow_mut().remove(&self.entity_id);
            }
            let clone = self.position.y - 0.1;
            if let Some(blk) = game.world.get_block(self.position.x as i32, clone as i32, self.position.z as i32) {
                if blk.get_type() == 0 {
                    self.position.y -= 0.1;
                }
            }
            if self.health < 0 {
                for player in game.players.0.borrow().clone().iter() {
                    player.1.write_packet(ServerPacket::EntityStatus {
                        eid: self.entity_id.0,
                        entity_status: 3,
                    });
                }
                self.despawn_at = Some(game.ticks + 22);
                return;
            }
        }
    }
    fn get_position(&mut self) -> &mut Position {
        &mut self.position
    }
}
