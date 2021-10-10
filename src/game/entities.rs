pub mod item_entity;
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
    fn as_any(&mut self) -> &mut dyn Any;
}