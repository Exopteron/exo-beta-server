use hecs::Entity;

use crate::aabb::AABBSize;
use crate::configuration::CONFIGURATION;
use crate::ecs::entities::living::{Health, Dead};
use crate::ecs::entities::player::{
    ChatMessage, Chatbox, ChunkLoadQueue, CurrentWorldInfo, Username, self, Player,
};
use crate::ecs::systems::SysResult;
use crate::ecs::EntityRef;
use crate::entities::SpawnPacketSender;
use crate::events::PlayerSpawnEvent;
use crate::game::{BlockPosition, ChunkCoords, DamageType, Game, Message, Position};
use crate::network::ids::NetworkID;
use crate::protocol::ClientPlayPacket;
use crate::server::Server;
use crate::translation::TranslationManager;
use crate::world::ChunkLoadData;
use std::cell::RefCell;
use std::ops::Deref;
use std::sync::Arc;
mod entity_action;
mod entity_animation;
mod interaction;
mod inventory;
mod movement;
pub fn handle_packet(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: ClientPlayPacket,
) -> anyhow::Result<()> {
    let pref = game.ecs.entity(player)?;
    match packet {
        ClientPlayPacket::PlayerPositionAndLook(p) => {
            let prev_position = *pref.get::<Position>()?;
            movement::handle_player_position_and_look(server, pref, p)?;
            player_moved(player, game, prev_position)?;
        }
        ClientPlayPacket::PlayerMovement(p) => {
            let prev_position = *pref.get::<Position>()?;
            movement::handle_player_movement(pref, p)?;
            player_moved(player, game, prev_position)?;
        }
        ClientPlayPacket::PlayerPosition(p) => {
            let prev_position = *pref.get::<Position>()?;
            movement::handle_player_position(server, pref, p)?;
            player_moved(player, game, prev_position)?;
        }
        ClientPlayPacket::PlayerLook(p) => {
            let prev_position = *pref.get::<Position>()?;
            movement::handle_player_look(server, pref, p)?;
            player_moved(player, game, prev_position)?;
        }
        ClientPlayPacket::ChatMessage(p) => {
            handle_chat_message(game, server, player, p)?;
        }
        ClientPlayPacket::Disconnect(p) => {
            let name = pref.get::<Username>()?;
            log::info!("{} disconnected: {}", name.0, p.reason);
            drop(name);
            let id = pref.get::<NetworkID>()?.deref().clone();
            let client = server.clients.get_mut(&id).expect("Player has no client?");
            client.set_disconnected(true);
        }
        ClientPlayPacket::EntityAction(p) => {
            entity_action::handle_entity_action(game, player, p)?;
        }
        ClientPlayPacket::Animation(p) => {
            entity_animation::handle_animation(server, pref, p)?;
        }
        ClientPlayPacket::HoldingChange(p) => {
            interaction::handle_held_item_change(server, pref, p)?;
        }
        ClientPlayPacket::CreativeInventoryAction(p) => {
            inventory::handle_creative_inventory_action(pref, p, server)?;
        }
        ClientPlayPacket::WindowClick(p) => {
            inventory::handle_click_window(server, pref, p)?;
        }
        ClientPlayPacket::PlayerDigging(p) => {
            interaction::handle_player_digging(game, server, p, player)?;
        }
        ClientPlayPacket::PlayerBlockPlacement(p) => {
            interaction::handle_player_block_placement(game, server, p, player)?;
        }
        ClientPlayPacket::Respawn(_) => {
            let netid = *game.ecs.get::<NetworkID>(player)?.deref();
            game.ecs
            .insert_entity_event(player, PlayerSpawnEvent)
            .unwrap();
            if let Err(_) = game.ecs.remove::<Dead>(player) {

            }
            let world = game.worlds.get(&game.ecs.get::<CurrentWorldInfo>(player)?.world_id).unwrap();
            let pref = game.ecs.entity(player)?;
            server.clients.get(&netid).unwrap().notify_respawn(&pref, world.level_dat.world_seed)?;
        }
        ClientPlayPacket::UpdateSign(p) => {
            interaction::handle_update_sign(game, server, player, p)?;
        }
        _ => {}
    }
    Ok(())
}

fn handle_chat_message(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: crate::protocol::packets::client::ChatMessage,
) -> SysResult {
    if packet.message.0.starts_with('/') {
        let name = game.ecs.get::<Username>(player)?.0.clone();
        let mut message = packet.message.0;
        log::info!("{} issued server command {}", name, message);
        message.remove(0);
        let res = game.execute_command(server, &message, player);
        if let Ok(c) = res {
            let player = game.ecs.entity(player)?;
            let mut chatbox = player.get_mut::<Chatbox>()?;
            if let Some(message) = crate::commands::code_to_message(c) {
                chatbox.send_message(message.into());
            }
        } else if let Err(e) = res {
            let player = game.ecs.entity(player)?;
            let mut chatbox = player.get_mut::<Chatbox>()?;
            chatbox.send_message("§cAn internal error occured.".into());
            log::error!("Command error: {:?}", e);
        }
    } else {
        let player = game.ecs.entity(player)?;
        let name = player.get::<Username>()?;
        let translation = game.objects.get::<TranslationManager>().unwrap();
        let format = translation.translate("chat.type.text", Some(vec![name.0.clone(), packet.message.0]));
        drop(name);
        let message = ChatMessage::new(format);
        for (_, chatbox) in game.ecs.query::<&mut Chatbox>().iter() {
            chatbox.send_message(message.clone());
        }
    }
    Ok(())
}


fn player_moved(player: Entity, game: &mut Game, prev_position: Position) -> SysResult {
    let position = *game.ecs.get::<Position>(player)?;
    let aabb = *game.ecs.get::<AABBSize>(player)?;
    let world = game.worlds.get(&position.world).unwrap();
    let mut current_collisions = world.get_collisions(&aabb, &position);
    let prev_collisions = world.get_collisions(&aabb, &prev_position);
    current_collisions.retain(|v| {
        !prev_collisions.contains(v)
    });
    for collision in current_collisions {
        collision.0.on_collide(game, collision.2, collision.1, player)?;
    }
    Ok(())
}