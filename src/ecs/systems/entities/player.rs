use std::ops::Deref;

use crate::{
    ecs::{
        entities::{player::{Player, Username, Chatbox, Gamemode, PreviousGamemode, OffgroundHeight, CurrentWorldInfo}, living::{PreviousHealth, Hunger, Health, Dead, PreviousHunger, Regenerator}},
        systems::{SystemExecutor, SysResult},
    },
    game::{Game, Position, DamageType},
    network::ids::NetworkID,
    server::Server, protocol::{packets::{server::ChatMessage, EntityStatusKind}, io::String16}, events::{EntityDeathEvent, PlayerSpawnEvent}, item::{window::Window, item::ItemRegistry}, entities::SpawnPacketSender, translation::TranslationManager, aabb::AABBSize,
};

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s.add_system(check_fall_damage).add_system(regenerate);
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
        //         *pref.get_mut::<Position>()? = Position::from_pos(324., 75., -472.);
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
fn regenerate(game: &mut Game) -> SysResult {
    for (e, (health, hunger, regenerator)) in game.ecs.query::<(&mut Health, &mut Hunger, &mut Regenerator)>().iter() {
        if !game.ecs.get::<Dead>(e).is_ok() {
            if regenerator.0 == 0 {
                if health.0 < 20 {
                    if hunger.get_points(1) {
                        health.0 += 2;
                    }
                }
                regenerator.0 = 80;
            } else {
                regenerator.0 -= 1;
            }
        }
    }
    Ok(())
}
fn update_health(game: &mut Game, server: &mut Server) -> SysResult {
    let mut make_dead = Vec::new();
    let mut hurt = Vec::new();
    for (entity, (health, prev_health, hunger, prev_hunger, id)) in game.ecs.query::<(&Health, &mut PreviousHealth, &Hunger, &mut PreviousHunger, &NetworkID)>().iter() {
        if health.0 != prev_health.0.0 || hunger.0 != prev_hunger.0.0 {
            let client = server.clients.get(id).unwrap();
            client.set_health(health, hunger);
            if health.0 < prev_health.0.0 {
                hurt.push(entity);
            }
            prev_health.0 = health.clone();
            prev_hunger.0 = hunger.clone();
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
        let health = entity_ref.get::<Health>()?;
        let pos = entity_ref.get::<Position>()?;
        let id = entity_ref.get::<NetworkID>()?;
        if let Ok(name) = entity_ref.get::<Username>() {
            game.broadcast_chat(format!("{} {}", name.0, health.1.string()));
        }
        server.broadcast_entity_status(*pos, *id, EntityStatusKind::EntityDead);
    }
    Ok(())
}

fn check_fall_damage(game: &mut Game) -> SysResult {
    //log::info!("Running");
    for (entity, (health, fall_start, pos, world, bounding_box)) in game.ecs.query::<(&mut Health, &OffgroundHeight, &Position, &CurrentWorldInfo, &AABBSize)>().iter() {
        //log::info!("Found entity");
        if pos.on_ground {
            //log::info!("On ground");
            if fall_start.1 as f64 > pos.y {
                let height = fall_start.1 as f64 - pos.y;
                //log::info!("Height: {}", height);
                let mut do_dmg = true;
                if let Ok(mode) = game.ecs.get::<Gamemode>(entity) {
                    if *mode == Gamemode::Creative {
                        do_dmg = false;
                    }
                }
                if let Some(world) = game.worlds.get(&world.world_id) {
                    // TODO check if absorbs fall
                    if world.collides_with(bounding_box, pos, ItemRegistry::global().get_block(9).unwrap()) {
                        do_dmg = false;
                    }
                    if world.collides_with(bounding_box, pos, ItemRegistry::global().get_block(8).unwrap()) {
                        do_dmg = false;
                    }
                    for block in world.get_collisions(bounding_box, pos) {
                        if block.0.absorbs_fall() {
                            do_dmg = false;
                        }
                    }
                }
                if height > 0.0 && do_dmg {
                    let fall_damage = (height - 3.0).max(0.0);
                    health.damage(fall_damage.round() as i16, DamageType::Fall);
                }
            }
        }
    }
    Ok(())
}