use crate::{
    ecs::{entities::player::OffgroundHeight, systems::SysResult, EntityRef},
    game::Position,
    network::ids::NetworkID,
    protocol::packets::client::{
        PlayerLook, PlayerMovement, PlayerPosition, PlayerPositionAndLook,
    },
    server::Server, configuration::CONFIGURATION,
};

// Feather license in FEATHER_LICENSE.md

/// If a player has been teleported by the server,
/// we don't want to override their position if
/// we receive a movement packet before the client
/// is aware of the position update.
fn should_skip_movement(server: &Server, player: &EntityRef) -> SysResult<bool> {
    if let Some(client) = server.clients.get(&*player.get::<NetworkID>()?) {
        let server_position = *player.get::<Position>()?;
        let client_position = client.client_known_position();
        if let Some(client_position) = client_position {
            if client_position != server_position {
                // Player has been teleported by the server.
                // Don't override.
                return Ok(true);
            }
        }
    }
    Ok(false)
}
fn update_client_position(server: &Server, player: &EntityRef, pos: Position) -> SysResult {
    if let Some(client) = server.clients.get(&*player.get::<NetworkID>()?) {
        client.set_client_known_position(pos);
    }
    Ok(())
}
fn update_offground_height(player: &EntityRef, pos: Position) -> SysResult {
    if pos.on_ground {
        let mut h = player.get_mut::<OffgroundHeight>()?;
        h.1 = h.0;
        h.0 = 0.0;
    } else {
        let mut height = player.get_mut::<OffgroundHeight>()?;
        //log::info!("Height is: {}", height.0);
        if pos.y as f32 > height.0 {
            //log::info!("Setting the height to {} because it is greater than {}", pos.y as f32, height.0);
            *height = OffgroundHeight(pos.y as f32, height.1);
        }
    }
    Ok(())
}
pub fn handle_player_position(
    server: &Server,
    player: EntityRef,
    packet: PlayerPosition,
) -> SysResult {
    if should_skip_movement(server, &player)? {
        return Ok(());
    }
    let mut pos = player.get_mut::<Position>()?;
    let previous_pos = pos.clone();
    pos.x = packet.x;
    pos.y = packet.y;
    pos.z = packet.z;
    pos.stance = packet.stance;
    pos.on_ground = packet.on_ground;
    update_offground_height(&player, *pos)?;
    update_client_position(server, &player, *pos)?;
    on_movement(&player, previous_pos, &mut pos)?;
    Ok(())
}
pub fn handle_player_position_and_look(
    server: &Server,
    player: EntityRef,
    packet: PlayerPositionAndLook,
) -> SysResult {
    if should_skip_movement(server, &player)? {
        return Ok(());
    }
    let mut pos = player.get_mut::<Position>()?;
    let previous_pos = pos.clone();
    pos.x = packet.x;
    pos.y = packet.y;
    pos.z = packet.z;
    pos.yaw = packet.yaw;
    pos.pitch = packet.pitch;
    pos.on_ground = packet.on_ground;
    update_offground_height(&player, *pos)?;
    update_client_position(server, &player, *pos)?;
    on_movement(&player, previous_pos, &mut pos)?;
    Ok(())
}
pub fn handle_player_look(server: &Server, player: EntityRef, packet: PlayerLook) -> SysResult {
    if should_skip_movement(server, &player)? {
        return Ok(());
    }
    let mut pos = player.get_mut::<Position>()?;
    let previous_pos = pos.clone();
    pos.yaw = packet.yaw;
    pos.pitch = packet.pitch;
    pos.on_ground = packet.on_ground;
    update_offground_height(&player, *pos)?;
    update_client_position(server, &player, *pos)?;
    on_movement(&player, previous_pos, &mut pos)?;
    Ok(())
}
pub fn handle_player_movement(player: EntityRef, packet: PlayerMovement) -> SysResult {
    let mut pos = player.get_mut::<Position>()?;
    let previous_pos = pos.clone();
    pos.on_ground = packet.on_ground;
    update_offground_height(&player, *pos)?;
    on_movement(&player, previous_pos, &mut pos)?;
    Ok(())
}

fn on_movement(player: &EntityRef, previous_pos: Position, pos: &mut Position) -> SysResult {
    let border = CONFIGURATION.world_border;
    if pos.x > border as f64 {
        pos.x = border as f64;
    }
    if pos.x < -(border as f64) {
        pos.x = -(border as f64);
    }

    if pos.z > border as f64 {
        pos.z = border as f64;
    }
    if pos.z < -(border as f64) {
        pos.z = -(border as f64);
    }
    Ok(())
}