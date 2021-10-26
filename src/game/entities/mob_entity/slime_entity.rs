use super::*;
use crate::network::packet::ServerPacket;
use std::default::Default;
pub struct SlimeEntity {
    entity_id: EntityID,
    position: Position,
    ticks_spawned: u128,
    health: i16,
    dead: bool,
    despawn_at: Option<u128>,
    last_move: u128,
    last_hit: u128,
    size: u8,
    motion: Motion,
}
impl SlimeEntity {
    pub fn new(position: Position, ticks_spawned: u128, size: u8) -> Self {
        Self {
            entity_id: EntityID::new(),
            position: position,
            ticks_spawned: ticks_spawned,
            health: 10,
            dead: false,
            despawn_at: None,
            last_move: 0,
            last_hit: 0,
            size: size,
            motion: Motion::default(),
        }
    }
    pub fn destruct(self) {
        IDS.lock().unwrap().push(self.entity_id.0);
    }
}
impl MobEntity for SlimeEntity {
    fn get_speed(&self) -> f64 {
        1.0
    }
    fn get_motion(&mut self) -> &mut Motion {
        &mut self.motion
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_id(&self) -> EntityID {
        self.entity_id.clone()
    }
    fn is_dead(&self) -> bool {
        self.dead
    }
    fn get_metadata(&mut self) -> Metadata {
        let mut md = Metadata::new();
        md.insert_byte_idx(self.size, 16);
        md
    }
    fn set_position(&mut self, pos: Position) {
        self.position = pos;
    }
    fn set_dead(&mut self, state: bool) {
        self.dead = state;
    }
    fn get_health(&self) -> i16 {
        self.health
    }
    fn set_health(&mut self, health: i16) {
        self.health = health;
    }
    fn damage(&mut self, game: &mut Game, amount: i16) {
        //log::info!("I am being damaged!");
        game.broadcast_player_loaded_entity(self.entity_id, ServerPacket::Animation {
            eid: self.entity_id.0,
            animate: 2,
        });
        game.broadcast_player_loaded_entity(self.entity_id, ServerPacket::EntityStatus {
            eid: self.entity_id.0,
            entity_status: 2,
        });
        self.health -= amount;
    }
    fn tick(&mut self, game: &mut Game) {
        if let Some(despawn_at) = self.despawn_at {
            if game.ticks >= despawn_at {
                game.entities.borrow_mut().remove(&self.entity_id);
            }
        } else {
            let mut closest_position: Option<Position> = None;
            for player in game.players.0.lock().unwrap().clone().iter() {
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
                self.motion.1 += 1.;
                //self.position.move_towards(&closest_position.unwrap(), 0.5);
                self.last_move = game.ticks;
            }
            //self.position.y += 1.0;
            //log::info!("Going up!");
            if game.ticks - self.ticks_spawned > 600 {
                game.entities.borrow_mut().remove(&self.entity_id);
            }
            let clone = self.position.y - 0.1;
            let block = game.world.get_block(&BlockPosition::new(self.position.x as i32, clone as i32, self.position.z as i32));
            if block.get_type() == 0 {
                self.position.y -= 0.1;
            }
            if self.health < 0 {
                for player in game.players.0.lock().unwrap().clone().iter() {
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
    fn get_type(&mut self) -> i8 {
        55
    }
}
