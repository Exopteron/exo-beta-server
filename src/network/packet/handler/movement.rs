use crate::{server::Server, ecs::{EntityRef, systems::SysResult}, game::Position, network::ids::NetworkID, protocol::packets::client::{PlayerPosition, PlayerPositionAndLook, PlayerLook, PlayerMovement}};

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
fn update_client_position(server: &Server, player: EntityRef, pos: Position) -> SysResult {
    if let Some(client) = server.clients.get(&*player.get::<NetworkID>()?) {
        client.set_client_known_position(pos);
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
    pos.x = packet.x;
    pos.y = packet.y;
    pos.z = packet.z;
    pos.stance = packet.stance;
    pos.on_ground = packet.on_ground;
    update_client_position(server, player, *pos)?;
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
    pos.x = packet.x;
    pos.y = packet.y;
    pos.z = packet.z;
    pos.yaw = packet.yaw;
    pos.pitch = packet.pitch;
    pos.on_ground = packet.on_ground;
    update_client_position(server, player, *pos)?;
    Ok(())
}
pub fn handle_player_look(
    server: &Server,
    player: EntityRef,
    packet: PlayerLook,
) -> SysResult {
    if should_skip_movement(server, &player)? {
        return Ok(());
    }
    let mut pos = player.get_mut::<Position>()?;
    pos.yaw = packet.yaw;
    pos.pitch = packet.pitch;
    pos.on_ground = packet.on_ground;
    update_client_position(server, player, *pos)?;
    Ok(())
}
pub fn handle_player_movement(player: EntityRef, packet: PlayerMovement) -> SysResult {
    player.get_mut::<Position>()?.on_ground = packet.on_ground;
    Ok(())
}
