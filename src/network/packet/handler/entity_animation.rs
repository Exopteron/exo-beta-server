use crate::{server::Server, ecs::{EntityRef, systems::SysResult}, game::Position, network::ids::NetworkID, protocol::packets::{client::Animation, EntityAnimationType}};

pub fn handle_animation(
    server: &mut Server,
    player: EntityRef,
    packet: Animation,
) -> SysResult {
    let pos = *player.get::<Position>()?;
    let network_id = *player.get::<NetworkID>()?;

    if matches!(packet.animation, EntityAnimationType::SwingArm) {
        server.broadcast_nearby_with(pos, |client| {
            client.send_entity_animation(network_id, packet.animation.clone());
        });
    }
    Ok(())
}