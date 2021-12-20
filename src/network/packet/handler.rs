use hecs::Entity;

use crate::configuration::CONFIGURATION;
use crate::ecs::EntityRef;
use crate::ecs::entities::player::{ChunkLoadQueue, CurrentWorldInfo, Username, Chatbox, ChatMessage};
use crate::game::{BlockPosition, ChunkCoords, DamageType, Game, ItemStack, Message, Position};
use crate::network::ids::NetworkID;
use crate::protocol::ClientPlayPacket;
use crate::server::Server;
use crate::world::ChunkLoadData;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
mod movement;
mod entity_action;
mod entity_animation;
pub fn handle_packet(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: ClientPlayPacket,
) -> anyhow::Result<()> {
    match packet {
        ClientPlayPacket::PlayerPositionAndLook(p) => {
            movement::handle_player_position_and_look(server, game.ecs.entity(player)?, p)?;
        }
        ClientPlayPacket::PlayerMovement(p) => {
            movement::handle_player_movement(game.ecs.entity(player)?, p)?;
        }
        ClientPlayPacket::PlayerPosition(p) => {
            movement::handle_player_position(server, game.ecs.entity(player)?, p)?;
        }
        ClientPlayPacket::PlayerLook(p) => {
            movement::handle_player_look(server, game.ecs.entity(player)?, p)?;
        }
        ClientPlayPacket::ChatMessage(p) => {
            if p.message.0.contains("amogus") {
                let id = game.ecs.entity(player)?.get::<NetworkID>()?.deref().clone();
                let client = server.clients.get_mut(&id).expect("Player has no client?");
                client.disconnect("Adios susboy");
            }
            let name = game.ecs.entity(player)?.get::<Username>()?;
            let format = format!("<{}> {}", name.0, p.message.0);
            drop(name);
            let message = ChatMessage::new(format);
            for (_, chatbox) in game.ecs.query::<&mut Chatbox>().iter() {
                chatbox.send_message(message.clone());
            }
        }
        ClientPlayPacket::Disconnect(p) => {
            let name = game.ecs.entity(player)?.get::<Username>()?;
            log::info!("{} disconnected: {}", name.0, p.reason);
            drop(name);
            let id = game.ecs.entity(player)?.get::<NetworkID>()?.deref().clone();
            let client = server.clients.get_mut(&id).expect("Player has no client?");
            client.set_disconnected(true);
        }
        ClientPlayPacket::EntityAction(p) => {
            entity_action::handle_entity_action(game, player, p)?;
        }
        ClientPlayPacket::Animation(p) => {
            entity_animation::handle_animation(server, game.ecs.entity(player)?, p)?;
        }
        _ => {

        }
    }
    Ok(())
}