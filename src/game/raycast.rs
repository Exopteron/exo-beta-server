use super::*;
pub struct Raycaster {

}
pub enum RaycastResult {
    Block(crate::chunks::Block),
    Entity(EntityID)
}
impl Raycaster {
    pub fn cast_blocks(game: &mut Game, mut from: Position, to: Position, max_distance: u64) -> Option<RaycastResult> {  
        let mut i = 0;
        while i < max_distance {
            if from.distance(&to) < 0.1 {
                return None;
            }
            log::info!("Checking {:?}", from);
            if let Some(block) = game.world.get_block(from.x.floor() as i32, from.y.floor() as i32, from.z.floor() as i32) {
                if block.get_type() != 0 {
                    return Some(RaycastResult::Block(block.clone()));
                }
            }
            from.move_towards(&to, 1.); 
            i += 1;
        }
        None
    }
    pub fn cast(game: &mut Game, mut from: Position, to: Position, max_distance: u64) -> Option<RaycastResult> {  
        let mut i = 0;
        while i < max_distance {
            if from.distance(&to) < 0.1 {
                return None;
            }
            log::info!("Checking {:?}", from);
            if let Some(block) = game.world.get_block(from.x.floor() as i32, from.y.floor() as i32 + 1, from.z.floor() as i32) {
                if block.get_type() != 0 {
                    return Some(RaycastResult::Block(block.clone()));
                }
            }
            for (id, entity) in game.entities.borrow().clone().iter() {
                if from.distance(&entity.borrow_mut().get_position()) < 0.11 {
                    return Some(RaycastResult::Entity(*id));
                }
            }
            from.move_towards(&to, 0.25); 
            i += 1;
        }
        None
    }
}