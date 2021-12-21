use std::ops::Deref;

use crate::{
    ecs::{
        entities::{player::{Player, Username, Chatbox, Gamemode, PreviousGamemode}, living::{PreviousHealth, Hunger, Health, Dead}},
        systems::{SystemExecutor, SysResult},
    },
    game::{Game, Position},
    network::ids::NetworkID,
    server::Server, protocol::{packets::{server::ChatMessage, EntityStatusKind}, io::String16}, events::{EntityDeathEvent, PlayerSpawnEvent}, item::window::Window, entities::SpawnPacketSender, translation::TranslationManager,
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
    }).add_system(update_gamemode).add_system(update_health).add_system(spawn_listener);
}
fn broadcast_player_leave(game: &Game, username: &Username) {
    let translation = game.objects.get::<TranslationManager>().unwrap();
    game.broadcast_chat(translation.translate("multiplayer.player.left", Some(vec![username.0.clone()])));
}

fn update_gamemode(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (gamemode, prev_gamemode, id)) in game.ecs.query::<(&Gamemode, &mut PreviousGamemode, &NetworkID)>().iter() {
        if gamemode.id() != prev_gamemode.id() {
            let client = server.clients.get(id).unwrap();
            client.set_gamemode(*gamemode);
            prev_gamemode.0 = Some(*gamemode);
        }
    }
    Ok(())
}
fn spawn_listener(game: &mut Game, server: &mut Server) -> SysResult {
    let mut entities_respawned = Vec::new();
    for (entity, (_, networkid, window)) in game.ecs.query::<(&PlayerSpawnEvent, &NetworkID, &Window)>().iter() {
        let client = server.clients.get(networkid).unwrap();
        client.send_window_items(window);
        entities_respawned.push(entity);
    }
    for entity in entities_respawned {
        let pref = game.ecs.entity(entity)?;
        let netid = pref.get::<NetworkID>()?.deref().clone();
        pref.get_mut::<Health>()?.0 = 20;
        *pref.get_mut::<Position>()? = Position::from_pos(0., 75., 0.);
        let spawnpacket = pref.get::<SpawnPacketSender>()?;
        server.broadcast_nearby_with(*pref.get::<Position>()?, |cl| {
            if cl.id != netid {
                cl.unload_entity(netid);
                if let Err(e) = spawnpacket.send(&pref, cl) {
                    log::error!("Error sending spawn packet after respawn to user: {:?}", e);
                }
            }
        });
    }
    Ok(())
}
fn update_health(game: &mut Game, server: &mut Server) -> SysResult {
    let mut make_dead = Vec::new();
    let mut hurt = Vec::new();
    for (entity, (health, prev_health, hunger, id)) in game.ecs.query::<(&Health, &mut PreviousHealth, &Hunger, &NetworkID)>().iter() {
        if health.0 != prev_health.0.0 {
            let client = server.clients.get(id).unwrap();
            client.set_health(health, hunger);
            if health.0 < prev_health.0.0 {
                hurt.push(entity);
            }
            prev_health.0 = health.clone();
            if health.0 <= 0 {
                make_dead.push(entity);
            }
        }
    }
    for hurt in hurt {
        let entity_ref = game.ecs.entity(hurt)?;
        let pos = entity_ref.get::<Position>()?;
        let id = entity_ref.get::<NetworkID>()?;
        server.broadcast_entity_status(*pos, *id, EntityStatusKind::EntityHurt);
    }
    for dead in make_dead {
        game.ecs.insert_entity_event(dead, EntityDeathEvent)?;
        game.ecs.insert(dead, Dead)?;
        let entity_ref = game.ecs.entity(dead)?;
        let pos = entity_ref.get::<Position>()?;
        let id = entity_ref.get::<NetworkID>()?;
        server.broadcast_entity_status(*pos, *id, EntityStatusKind::EntityDead);
    }
    Ok(())
}