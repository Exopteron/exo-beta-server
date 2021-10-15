use super::*;
pub mod slime_entity;
pub type Motion = (f64, f64, f64);
pub trait MobEntity {
    fn get_id(&self) -> EntityID;
    fn get_type(&mut self) -> i8;
    fn get_metadata(&mut self) -> Metadata;
    fn get_position(&mut self) -> &mut Position;
    fn as_any(&self) -> &dyn Any;
    fn set_position(&mut self, pos: Position);
    fn get_motion(&mut self) -> &mut Motion;
    fn set_dead(&mut self, state: bool);
    fn is_dead(&self) -> bool;
    fn get_health(&self) -> i16;
    fn set_health(&mut self, state: i16);
    fn damage(&mut self, game: &mut Game, amount: i16);
    fn tick(&mut self, game: &mut Game);
    fn get_speed(&self) -> f64;
}
impl<T> Entity for T
where
    T: MobEntity,
{
    fn tick(&mut self, game: &mut Game) {
        let mut pos = self.get_position().clone();
        let motion = self.get_motion().clone();
        let amount_x = if motion.0 >= self.get_speed() {
            self.get_speed()
        } else {
            0.
        };
        let amount_y = if motion.1 >= self.get_speed() {
            self.get_speed()
        } else {
            0.
        };
        let amount_z = if motion.2 >= self.get_speed() {
            self.get_speed()
        } else {
            0.
        };
        pos.x += amount_x;
        pos.y += amount_y;
        pos.z += amount_z;
        self.set_position(pos);
        let motion = self.get_motion();
        motion.0 = amount_x;
        motion.1 = amount_y;
        motion.2 = amount_z;
        MobEntity::tick(self, game);
    }
    fn damage(&mut self, game: &mut Game, amount: i16) {
        MobEntity::damage(self, game, amount);
    }
    fn destruct_entity(&self, mut player: &mut RefMut<'_, Player>) {
        player.write(ServerPacket::DestroyEntity {
            eid: self.get_id().0,
        });
    }
    fn is_dead(&self) -> bool {
        MobEntity::is_dead(self)
    }
    fn spawn_entity(&mut self, player: &mut RefMut<'_, Player>) {
        let mut metadata = MobEntity::get_metadata(self);
        let pos = self.get_position().clone();
        player.write(ServerPacket::MobSpawn {
            eid: self.get_id().0,
            m_type: 55,
            x: (pos.x * 32.0) as i32,
            y: (pos.y * 32.0) as i32,
            z: (pos.z * 32.0) as i32,
            yaw: pos.yaw as i8,
            pitch: pos.pitch as i8,
            metadata: metadata,
        });
    }
    fn as_any(&mut self) -> &mut (dyn Any + 'static) where Self: 'static {
        self
    }
    fn get_id(&self) -> EntityID {
        MobEntity::get_id(self)
    }
    fn get_position(&mut self) -> &mut Position {
        MobEntity::get_position(self)
    }
}