use hecs::Entity;

use crate::ecs::EntityRef;
use crate::game::{BlockPosition, DamageType, Game, ItemStack, Message, Position};
use crate::network::ids::EntityID;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::server::Server;
use std::cell::RefCell;
use std::sync::Arc;
pub fn handle_packet(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: ClientPacket,
) -> anyhow::Result<()> {
    //player.unwrap().unwrap().last_keepalive_time = game.ticks;
    match packet {
        ClientPacket::PlayerPositionAndLookPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.x = p.x;
            pos.y = p.y;
            pos.z = p.z;
            pos.yaw = p.yaw;
            pos.pitch = p.pitch;
            pos.on_ground = p.on_ground;
            log::info!("PPAL");
        }
        ClientPacket::PlayerPositionPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.x = p.x;
            pos.y = p.y;
            pos.z = p.z;
            pos.on_ground = p.on_ground;
            log::info!("PP");
        }
        ClientPacket::PlayerLookPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.yaw = p.yaw;
            pos.pitch = p.pitch;
            pos.on_ground = p.on_ground;
            log::info!("L");
        }
        ClientPacket::PlayerPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.on_ground = p.on_ground;
            log::info!("P");
        }
        _ => {}
    }
    Ok(())
}