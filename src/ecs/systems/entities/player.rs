use crate::{
    ecs::{
        entities::player::{Player, Username, Chatbox},
        systems::SystemExecutor,
    },
    game::Game,
    network::ids::NetworkID,
    server::Server, protocol::{packets::server::ChatMessage, io::String16},
};

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s.group::<Server>().add_system(|game, server| {
        let mut to_despawn = Vec::new();
        for (p, (_, id, name)) in game.ecs.query::<(&Player, &NetworkID, &Username)>().iter() {
            let client = server.clients.get(id).unwrap();
            if client.is_disconnected() {
                // TODO: broadcast disconnection event whatnot
                to_despawn.push(p);
                broadcast_player_leave(game, name);
                server.remove_client(*id);
                server.player_count.remove_player();
            }
        }
        for entity in to_despawn {
            game.remove_entity(entity)?;
        }
        Ok(())
    });
}
fn broadcast_player_leave(game: &Game, username: &Username) {
    game.broadcast_chat(format!("{} left the game.", username.0));
}