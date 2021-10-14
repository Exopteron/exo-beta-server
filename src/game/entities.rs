pub mod item_entity;
pub mod slime_entity;
pub mod tile_entity;
pub mod gravel_entity;
use super::*;
pub trait Entity {
    fn spawn_entity(&mut self, player: &mut RefMut<'_, Player>);
    fn destruct_entity(&self, player: &mut RefMut<'_, Player>);
    fn tick(&mut self, game: &mut Game);
    fn get_position(&mut self) -> &mut Position;
    fn get_id(&self) -> EntityID;
    fn broadcast_pos_change(&mut self) -> bool {
        true
    }
    fn damage(&mut self, game: &mut Game, amount: i16) {

    }
    fn add_velocity(&mut self, velocity: [f64; 3]) {
        
    }
    fn is_dead(&self) -> bool {
        false
    }
    fn as_any(&mut self) -> &mut dyn Any;
}