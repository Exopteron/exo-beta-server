use super::*;
pub fn place_validator(game: &mut Game, packet: &crate::network::packet::PlayerBlockPlacement) -> bool {
    if packet.direction < 0 {
        return false;
    }
    if packet.y >= 127 {
        return false;
    }
    // BLOCKS thing
    let block = if let Some(blk) = game
        .world
        .get_block(packet.x, packet.y as i32, packet.z)
    {
        blk
    } else {
        return false;
    };
    for user in game.players.0.borrow().iter() {
        /*                     let mut pos = user.1.try_borrow();
        if pos.is_err() {
            continue;
        } */
        //let mut pos = pos;
        let pos = user.1.get_position_clone();
        if pos.contains_block(crate::game::BlockPosition {
            x: packet.x,
            y: (packet.y + 1) as i32,
            z: packet.z,
        }) {
            return false;
        }
        if pos.contains_block(crate::game::BlockPosition {
            x: packet.x,
            y: (packet.y) as i32,
            z: packet.z,
        }) {
            return false;
        }
    }
    true
}
