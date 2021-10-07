use crate::game::*;
pub trait Entity {
    fn get_position(&self) -> Position;
}
pub struct ItemEntity {
    position: Position,
}
impl Entity for ItemEntity {
    fn get_position(&self) -> Position {
        self.position.clone()
    }
}
impl Entity for Player {
    fn get_position(&self) -> Position {
        self.position.clone()
    }
}