//! Sends and unloads entities and chunks for a client.
//!
//! The entities and chunks visible to each client are
//! determined based on the player's [`common::view::View`].

use std::collections::HashMap;

use hecs::Entity;

use crate::{ecs::{systems::{SystemExecutor, SysResult}, entities::player::CurrentWorldInfo}, game::{Game, ChunkCoords, Position}, server::{Server, Client}, events::{ViewUpdateEvent, ChunkLoadEvent}, network::ids::NetworkID};


pub fn register(systems: &mut SystemExecutor<Game>) {
    systems
        .group::<Server>()
        .add_system(send_new_chunks)
        .add_system(send_loaded_chunks);
}

/// Stores the players waiting on chunks that are currently being loaded.
#[derive(Default)]
pub struct WaitingChunks(HashMap<ChunkCoords, Vec<Entity>>);

impl WaitingChunks {
    pub fn drain_players_waiting_for(&mut self, chunk: ChunkCoords) -> Vec<Entity> {
        self.0.remove(&chunk).unwrap_or_default()
    }

    pub fn insert(&mut self, player: Entity, chunk: ChunkCoords) {
        self.0.entry(chunk).or_default().push(player);
    }
}

fn send_new_chunks(game: &mut Game, server: &mut Server) -> SysResult {
    for (player, (&client_id, event, &position, &current_world)) in game
        .ecs
        .query::<(&NetworkID, &ViewUpdateEvent, &Position, &CurrentWorldInfo)>()
        .iter()
    {
        // As ecs removes the client one tick after it gets removed here, it can
        // happen that a client is still listed in the ecs but actually removed here so
        // we need to check if the client is actually still there.
        if let Some(client) = server.clients.get(&client_id) {
            //client.update_own_chunk(event.new_view.center());
            update_chunks(
                game,
                player,
                client,
                event,
                &current_world,
                position,
                &mut server.waiting_chunks,
            )?;
        }
    }
    Ok(())
}

fn update_chunks(
    game: &Game,
    player: Entity,
    client: &Client,
    event: &ViewUpdateEvent,
    current_world: &CurrentWorldInfo,
    position: Position,
    waiting_chunks: &mut WaitingChunks,
) -> SysResult {
    // Send chunks that are in the new view but not the old view.
    for &pos in &event.new_chunks {
        let world = game.worlds.get(&current_world.world_id).expect("World does not exist?");
        if let Some(chunk) = world.chunk_map.chunk_handle_at(pos) {
            client.send_chunk(&chunk);
        } else {
            //log::info!("Waiting for chunk");
            waiting_chunks.insert(player, pos);
        }
    }

    // Unsend the chunks that are in the old view but not the new view.
    for &pos in &event.old_chunks {
        client.unload_chunk(pos);
    }
    //log::info!("Spawning from here at {:?}!", position);
    spawn_client_if_needed(client, position);

    Ok(())
}

/// Sends newly loaded chunks to players currently
/// waiting for those chunks to load.
fn send_loaded_chunks(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, event) in game.ecs.query::<&ChunkLoadEvent>().iter() {
        for player in server
            .waiting_chunks
            .drain_players_waiting_for(event.position)
        {
            if let Ok(client_id) = game.ecs.get::<NetworkID>(player) {
                if let Some(client) = server.clients.get(&client_id) {
                    client.send_chunk(&event.chunk);
                    let possy = *game.ecs.get::<Position>(player)?;
                    //log::info!("Spawning at {:?}!", possy);

                    spawn_client_if_needed(client, possy);
                }
            }
        }
    }
    Ok(())
}

fn spawn_client_if_needed(client: &Client, pos: Position) {
    if !client.knows_own_position() && client.known_chunks() >= 9 * 9 {
        log::info!("Sent all chunks to {}; now spawning at {:?}", client.username(), pos);
        client.update_own_position(pos);
    }
}
