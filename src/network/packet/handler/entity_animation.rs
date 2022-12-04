use crate::{server::Server, ecs::{EntityRef, systems::SysResult, entities::player::Sleeping}, game::Position, network::ids::NetworkID, protocol::packets::{client::Animation, EntityAnimationType}};

pub fn handle_animation(
    server: &mut Server,
    player: EntityRef,
    packet: Animation,
) -> SysResult {
    let pos = *player.get::<Position>()?;
    let world = pos.world;
    let network_id = *player.get::<NetworkID>()?;
    match packet.animation {
        EntityAnimationType::SwingArm => {
            server.broadcast_nearby_with(pos, |client| {
                client.send_entity_animation(network_id, packet.animation);
            });
        }
        _ => ()
    }
    Ok(())
}