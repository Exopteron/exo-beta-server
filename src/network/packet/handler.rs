use hecs::Entity;

use crate::configuration::CONFIGURATION;
use crate::ecs::EntityRef;
use crate::ecs::entities::player::{ChunkLoadQueue, CurrentWorldInfo, NetworkManager};
use crate::game::{BlockPosition, ChunkCoords, DamageType, Game, ItemStack, Message, Position};
use crate::network::ids::EntityID;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::server::Server;
use crate::world::ChunkLoadData;
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
        ClientPacket::ChatMessage(p) => {
            log::info!("Got one");
            if p.message.starts_with("doepic") {
                log::info!("Doing epic.");
                game.teleport_player_notify(player, &Position::from_pos(0., 256., 0.))?;
            }
            game.ecs.entity(player)?.get_mut::<NetworkManager>()?.write(ServerPacket::ChatMessage { message: format!("You said: {}", p.message) });
        }
        ClientPacket::PlayerPositionAndLookPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.x = p.x;
            pos.y = p.y;
            pos.z = p.z;
            pos.yaw = p.yaw;
            pos.pitch = p.pitch;
            pos.on_ground = p.on_ground;
            //log::info!("PPAL");
            drop(plr);
            drop(pos);
            check_nearby_chunks(game, player);
        }
        ClientPacket::PlayerPositionPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.x = p.x;
            pos.y = p.y;
            pos.z = p.z;
            pos.on_ground = p.on_ground;
            //log::info!("PP");
            drop(plr);
            drop(pos);
            check_nearby_chunks(game, player);
        }
        ClientPacket::PlayerLookPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.yaw = p.yaw;
            pos.pitch = p.pitch;
            pos.on_ground = p.on_ground;
            //log::info!("L");
            drop(plr);
            drop(pos);
            check_nearby_chunks(game, player);
        }
        ClientPacket::PlayerPacket(p) => {
            let plr = game.ecs.entity(player)?;
            let mut pos = plr.get_mut::<Position>()?;
            pos.on_ground = p.on_ground;
            //log::info!("P");
            drop(plr);
            drop(pos);
            check_nearby_chunks(game, player);
        }
        _ => {}
    }
    Ok(())
}

fn check_nearby_chunks(game: &mut Game, player: Entity) {
    if let Ok(player) = game.ecs.entity(player) {
        let p = player.get::<Position>().unwrap();
        let pos = p.to_chunk_coords();
        let mut loaded = Vec::new();
        for x in -CONFIGURATION.chunk_distance..CONFIGURATION.chunk_distance {
            for z in -CONFIGURATION.chunk_distance..CONFIGURATION.chunk_distance {
                let c = ChunkCoords { x: pos.x + x, z: pos.z + z };
                loaded.push(c.clone());
                if !player.get::<ChunkLoadQueue>().unwrap().contains(&c) {
                    game.worlds.get_mut(&player.get::<CurrentWorldInfo>().unwrap().world_id).unwrap().load_chunk(&c);
                    player.get_mut::<ChunkLoadQueue>().unwrap().add(&c);  
                } 
            }
        }
        let mut to_notify_unload = Vec::new();
        player.get_mut::<ChunkLoadQueue>().unwrap().retain(|c| {
            let x = loaded.contains(c);
            if !x {
                to_notify_unload.push(c.clone());
            }
            x
        });
        for c in to_notify_unload {
            player.get_mut::<NetworkManager>().unwrap().write(ServerPacket::PreChunk { x: c.x, z: c.z, mode: false });
        }
    } else {
        log::info!("Error on checking nearby chunks");
    }
}