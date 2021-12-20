// feather license in FEATHER_LICENSE.md
//! Sends tablist info to clients via the Player Info packet.

use crate::{server::Server, game::Game, ecs::entities::player::{Player, Username}, events::{EntityRemoveEvent, PlayerJoinEvent}, network::ids::NetworkID};

use super::{SystemExecutor, SysResult};


pub fn register(systems: &mut SystemExecutor<Game>) {
    systems
        .group::<Server>()
        .add_system(remove_tablist_players)
        .add_system(add_tablist_players);
}

fn remove_tablist_players(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (_event, _player, name)) in game
        .ecs
        .query::<(&EntityRemoveEvent, &Player, &Username)>()
        .iter()
    {
        server.broadcast_with(|client| client.remove_tablist_player(name.0.clone()));
    }
    Ok(())
}

fn add_tablist_players(game: &mut Game, server: &mut Server) -> SysResult {
    for (player, (_, &network_id, name)) in game
        .ecs
        .query::<(
            &PlayerJoinEvent,
            &NetworkID,
            &Username,
        )>()
        .iter()
    {
        // Add this player to other players' tablists
        server.broadcast_with(|client| {
            client.add_tablist_player(name.0.to_string(), 0);
        });

        // Add other players to this player's tablist
        for (other_player, name) in game
            .ecs
            .query::<&Username>()
            .iter()
        {
            if let Some(client) = server.clients.get(&network_id) {
                if other_player != player {
                    client.add_tablist_player(name.0.to_string(), 0);
                }
            }
        }
    }
    Ok(())
}
