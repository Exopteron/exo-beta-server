use std::collections::HashMap;

use crate::{game::{ChunkCoords, Game}, network::ids::NetworkID, ecs::systems::{SystemExecutor, SysResult}, server::Server, events::{EntityRemoveEvent, ViewUpdateEvent}};

use super::view::View;

// feather license in FEATHER_LICENSE.md

/// Data structure to query which clients should
/// receive updates from a given chunk, fast.
#[derive(Default)]
pub struct ChunkSubscriptions {
    chunks: HashMap<ChunkCoords, Vec<NetworkID>>,
}

impl ChunkSubscriptions {
    pub fn subscriptions_for(&self, chunk: ChunkCoords) -> &[NetworkID] {
        self.chunks
            .get(&chunk)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }
}

pub fn register(systems: &mut SystemExecutor<Game>) {
    systems
        .group::<Server>()
        .add_system(update_chunk_subscriptions);
}

fn update_chunk_subscriptions(game: &mut Game, server: &mut Server) -> SysResult {
    // Update players whose views have changed
    for (_, (event, &client_id)) in game.ecs.query::<(&ViewUpdateEvent, &NetworkID)>().iter() {
        for new_chunk in event.new_view.difference(event.old_view) {
            server
                .chunk_subscriptions
                .chunks
                .entry(new_chunk)
                .or_default()
                .push(client_id);
        }
        for old_chunk in event.old_view.difference(event.new_view) {
            remove_subscription(server, old_chunk, client_id);
        }
    }

    // Update players that have left
    for (_, (_event, &client_id, &view)) in game
        .ecs
        .query::<(&EntityRemoveEvent, &NetworkID, &View)>()
        .iter()
    {
        for chunk in view.iter() {
            remove_subscription(server, chunk, client_id);
        }
    }

    Ok(())
}

fn remove_subscription(server: &mut Server, chunk: ChunkCoords, client_id: NetworkID) {
    if let Some(vec) = server.chunk_subscriptions.chunks.get_mut(&chunk) {
        vec_remove_item(vec, &client_id);

        if vec.is_empty() {
            server.chunk_subscriptions.chunks.remove(&chunk);
        }
    }
}

/// Swap-removes an item from a vector by equality.
pub fn vec_remove_item<T: PartialEq>(vec: &mut Vec<T>, item: &T) {
    let index = vec.iter().position(|x| x == item);
    if let Some(index) = index {
        vec.swap_remove(index);
    }
}
