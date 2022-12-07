use std::ops::Deref;

use crate::{
    aabb::AABBSize,
    ecs::{
        entities::{
            item::ItemEntityBuilder,
            living::{Dead, Health, Hunger, PreviousHealth, PreviousHunger, Regenerator},
            player::{
                Chatbox, Gamemode, HitCooldown, HotbarSlot, ItemInUse, Player,
                PreviousGamemode, Username, SLOT_HOTBAR_OFFSET, Sleeping,
            },
        },
        systems::{SysResult, SystemExecutor},
    },
    entities::SpawnPacketSender,
    events::{ChangeWorldEvent, EntityDeathEvent, PlayerSpawnEvent},
    game::{DamageType, Game, Position},
    item::{
        inventory::{reference::Area, Inventory},
        inventory_slot::InventorySlot,
        item::ItemRegistry,
        window::Window, self,
    },
    network::{ids::NetworkID, metadata::Metadata},
    physics::Physics,
    player_dat::PlayerDat,
    protocol::{
        io::String16,
        packets::{server::ChatMessage, EntityStatusKind, client::Animation, EntityAnimationType},
    },
    server::Server,
    status_effects::StatusEffectsManager,
    translation::TranslationManager, sleep::SleepManager,
};

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s
        .add_system(regenerate)
        .add_system(hit_cooldown);
    s.group::<Server>()
        .add_system(|game, server| {
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
                let entity_ref = game.ecs.entity(entity)?;
                let world = game.worlds.get(&0).unwrap();
                let mut pd_dir = world.world_dir.clone();
                let mut username = entity_ref.get::<Username>()?.0.clone();
                username = username.replace("\\", "");
                username = username.replace("/", "");
                username = username.replace("..", "");
                pd_dir.push("players/".to_owned() + &username + ".dat");
                PlayerDat::from_entity(&entity_ref)?.to_file(pd_dir)?;
                drop(entity_ref);
                game.remove_entity(entity)?;
            }
            Ok(())
        })
        .add_system(update_gamemode)
        .add_system(update_health)
        .add_system(spawn_listener)
        .add_system(notify_pos)
        .add_system(velocity)
        .add_system(item_use_ticker)
        .add_system(update_metadata)
        .add_system(update_sleeping);
}
fn update_sleeping(game: &mut Game, server: &mut Server) -> SysResult {
    let mut num_sleeping = 0;

    let is_night = (item::default::bed::NIGHT_START..item::default::bed::NIGHT_END).contains(&game.worlds.get(&0).unwrap().level_dat.lock().time);

    for (_, (_, id, sleeping, pos)) in game.ecs.query::<(&Player, &NetworkID, &mut Sleeping, &mut Position)>().iter() {
        if let Some(client) = server.clients.get(id) {
            if sleeping.changed() {
                if sleeping.is_sleeping() {
                    server.broadcast_nearby_with(*pos, |client| {
                        client.use_bed(*id, sleeping.bed_coords().unwrap(), true);
                    });
                } else {
                    server.broadcast_nearby_with(*pos, |cl| {
                        cl.send_entity_animation(*id, EntityAnimationType::LeaveBed);
                    });
                    client.wake_up_sleeping();
                }
                sleeping.reset_changed();
            }
            if sleeping.is_sleeping() && is_night {
                num_sleeping += 1;
            } else if sleeping.is_sleeping() {
                server.broadcast_nearby_with(*pos, |cl| {
                    cl.send_entity_animation(*id, EntityAnimationType::LeaveBed);
                });
                client.wake_up_sleeping();
                sleeping.unset_sleeping();
            }
        }
    }
    if num_sleeping == server.player_count.get() && is_night {
        let mut mgr = game.objects.get_mut::<SleepManager>()?;
        if mgr.update() {
            for (_, (_, id, sleeping, pos)) in game.ecs.query::<(&Player, &NetworkID, &mut Sleeping, &mut Position)>().iter() {
                let client = server.clients.get(id).unwrap();
                sleeping.unset_sleeping();
                client.wake_up_sleeping();
            }
            game.worlds.get_mut(&0).unwrap().level_dat.lock().time = 1000_i64;
        }
    } else {
        let _ = game.objects.get_mut::<SleepManager>().map(|mut v| v.reset());
    }
    Ok(())
}

fn update_metadata(game: &mut Game, server: &mut Server) -> SysResult {
    for (entity, (id, position, meta)) in game.ecs.query::<(&NetworkID, &Position, &mut Metadata)>().iter() {
        if meta.dirty {
            server.broadcast_nearby_with(*position, |client| {
                client.send_entity_metadata(true, *id, meta.clone());
            });
            meta.dirty = false;
        }
    }
    Ok(())
}
fn broadcast_player_leave(game: &Game, username: &Username) {
    let translation = game.objects.get::<TranslationManager>().unwrap();
    game.broadcast_chat(
        format!("§e{}", translation.translate("multiplayer.player.left", Some(vec![username.0.clone()]))),
    );
}

fn update_gamemode(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (gamemode, prev_gamemode, id)) in game
        .ecs
        .query::<(&Gamemode, &mut PreviousGamemode, &NetworkID)>()
        .iter()
    {
        if gamemode.id() != prev_gamemode.id() {
            let client = server.clients.get(id).unwrap();
            client.set_gamemode(*gamemode);
            prev_gamemode.0 = Some(*gamemode);
        }
    }
    Ok(())
}
fn item_use_ticker(game: &mut Game, server: &mut Server) -> SysResult {
    let mut entities = Vec::new();
    for (entity, _) in game.ecs.query::<&ItemInUse>().iter() {
        entities.push(entity);
    }
    for entity in entities {
        let mut ticker = game.ecs.get_mut::<ItemInUse>(entity)?;
        if ticker.0.is_filled() && ticker.1 == 0 {
            let item = ticker.0.take_all();
            if let InventorySlot::Filled(item) = item {
                let entity = game.ecs.entity(entity)?;
                let id = *entity.get::<NetworkID>()?;
                let hotbar_slot = entity.get::<HotbarSlot>()?.get();
                let inventory = entity.get::<Window>()?.inner().clone();
                let slot_id = SLOT_HOTBAR_OFFSET + hotbar_slot;
                let slot = inventory.item(slot_id)?;
                drop(ticker);
                let real_entity = entity.entity();
                drop(entity);
                match item.item() {
                    crate::item::stack::ItemStackType::Item(item) => {
                        item.on_eat(game, server, real_entity, slot, slot_id)?;
                    }
                    crate::item::stack::ItemStackType::Block(_) => (),
                }
            }
        }
    }
    for (_, item_in_use) in game.ecs.query::<&mut ItemInUse>().iter() {
        if !item_in_use.0.is_empty() && item_in_use.1 > 0 {
            item_in_use.1 -= 1;
        }
    }
    Ok(())
}
fn spawn_listener(game: &mut Game, server: &mut Server) -> SysResult {
    let mut entities_respawned = Vec::new();
    for (entity, (event, networkid, window)) in game
        .ecs
        .query::<(&PlayerSpawnEvent, &NetworkID, &Window)>()
        .iter()
    {
        let client = server.clients.get(networkid).unwrap();
        client.send_window_items(window);
        entities_respawned.push((entity, event.0));
    }
    for (entity, first) in entities_respawned {
        let pref = game.ecs.entity(entity)?;
        let current_world = pref.get::<Position>()?.world;
        let mut spawn_point = game
            .worlds
            .get(&current_world)
            .unwrap()
            .level_dat
            .lock()
            .spawn_point;
        let netid = *pref.get::<NetworkID>()?;
        if !first {
            pref.get_mut::<Health>()?.0 = 20;
            pref.get_mut::<Hunger>()?.0 = 20;
            pref.get_mut::<StatusEffectsManager>()?.reset();
            *pref.get_mut::<Position>()? = spawn_point.into();
        }
        let spawnpacket = pref.get::<SpawnPacketSender>()?;
        server.broadcast_nearby_with(*pref.get::<Position>()?, |cl| {
            if cl.id != netid {
                cl.unload_entity(netid);
                if let Err(e) = spawnpacket.send(game.scheduler.clone(), game.ticks, &pref, cl) {
                    log::error!("Error sending spawn packet after respawn to user: {:?}", e);
                }
            }
        });
        drop(spawnpacket);
        if game.ecs.remove::<Dead>(entity).is_err() {

        }
    }
    Ok(())
}
fn notify_pos(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (event, id, pos)) in game
        .ecs
        .query::<(&ChangeWorldEvent, &NetworkID, &mut Position)>()
        .iter()
    {
        pos.world = event.new_dim;
        let mut newpos = pos.clone();
        newpos.world = event.old_dim;
        server.broadcast_nearby_with(newpos, |c| {
            if *id != c.id {
                log::info!("Telling {} to unload", c.username());
                c.unload_entity(*id);
            }
        });
        let client = server.clients.get(id).ok_or(anyhow::anyhow!("No client"))?;
        client.update_own_position(*pos);
    }
    Ok(())
}
fn regenerate(game: &mut Game) -> SysResult {
    for (e, (health, hunger, regenerator)) in game
        .ecs
        .query::<(&mut Health, &mut Hunger, &mut Regenerator)>()
        .iter()
    {
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
    for (entity, (health, prev_health, hunger, prev_hunger, id)) in game
        .ecs
        .query::<(
            &mut Health,
            &mut PreviousHealth,
            &Hunger,
            &mut PreviousHunger,
            &NetworkID,
        )>()
        .iter()
    {
        if health.0 != prev_health.0 .0 || hunger.0 != prev_hunger.0.0 || health.2 {
            if let Some(client) = server.clients.get(id) {
                client.set_health(health, hunger);
            }
            if health.0 < prev_health.0.0 || health.2 {
                health.2 = false;
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
        server.broadcast_entity_status(*pos, pos.world, *id, EntityStatusKind::EntityHurt);
    }
    for dead in make_dead {
        game.ecs.insert_entity_event(dead, EntityDeathEvent)?;
        game.ecs.insert(dead, Dead)?;
        let entity_ref = game.ecs.entity(dead)?;
        let pos = *entity_ref.get::<Position>()?;
        let inventory = entity_ref.get::<Window>()?.inner().clone();
        for item in inventory.to_vec() {
            if let InventorySlot::Filled(item) = item {
                let builder = ItemEntityBuilder::build(game, pos, item);
                game.spawn_entity(builder);
            }
        }
        let entity_ref = game.ecs.entity(dead)?;
        let mut real_inv = entity_ref.get_mut::<Inventory>()?;
        real_inv.clear();
        let health = entity_ref.get::<Health>()?;
        let pos = entity_ref.get::<Position>()?;
        let id = entity_ref.get::<NetworkID>()?;
        if let Ok(name) = entity_ref.get::<Username>() {
            game.broadcast_chat(format!("{} {}", name.0, health.1.string()));
        }
        log::info!("Sending dead");
        server.broadcast_entity_status(*pos, pos.world, *id, EntityStatusKind::EntityDead);
        drop(real_inv);
        drop(health);
        drop(pos);
        drop(id);
        if entity_ref.get::<Player>().is_err() {
            game.schedule_at(game.ticks + 20, move |game| {
                game.remove_entity(dead);
                None
            });
        }
    }
    Ok(())
}

// fn check_fall_damage(game: &mut Game) -> SysResult {
//     //log::info!("Running");
//     for (entity, (health, fall_start, pos, bounding_box)) in game
//         .ecs
//         .query::<(&mut Health, &OffgroundHeight, &Position, &AABBSize)>()
//         .iter()
//     {
//         //log::info!("Found entity");
//         if pos.on_ground {
//             //log::info!("On ground");
//             if fall_start.1 as f64 > pos.y {
//                 let height = fall_start.1 as f64 - pos.y;
//                 //log::info!("Height: {}", height);
//                 let mut do_dmg = true;
//                 if let Ok(mode) = game.ecs.get::<Gamemode>(entity) {
//                     if *mode == Gamemode::Creative {
//                         do_dmg = false;
//                     }
//                 }
//                 if let Some(world) = game.worlds.get(&pos.world) {
//                     // TODO check if absorbs fall
//                     if world.collides_with(
//                         bounding_box,
//                         pos,
//                         ItemRegistry::global().get_block(9).unwrap(),
//                     ) {
//                         do_dmg = false;
//                     }
//                     if world.collides_with(
//                         bounding_box,
//                         pos,
//                         ItemRegistry::global().get_block(8).unwrap(),
//                     ) {
//                         do_dmg = false;
//                     }
//                     for block in world.get_collisions(bounding_box, None, pos) {
//                         if block.0.absorbs_fall() {
//                             do_dmg = false;
//                         }
//                     }
//                 }
//                 if height > 0.0 && do_dmg {
//                     let fall_damage = (height - 3.0).max(0.0);
//                     health.damage(fall_damage.round() as i16, DamageType::Fall);
//                 }
//             }
//         }
//     }
//     Ok(())
// }
fn hit_cooldown(game: &mut Game) -> SysResult {
    for (_, hit_cooldown) in game.ecs.query::<&mut HitCooldown>().iter() {
        if hit_cooldown.0 > 0 {
            hit_cooldown.0 -= 1;
        }
    }
    Ok(())
}
fn velocity(game: &mut Game, server: &mut Server) -> SysResult {
    let mut entities = Vec::new();
    for (entity, (_, velocity, id)) in game
        .ecs
        .query::<(&Player, &mut Physics, &NetworkID)>()
        .iter()
    {
        let client = match server.clients.get(id) {
            Some(c) => c,
            None => continue,
        };
        if velocity.modified() {
            let velocity = velocity.get_velocity();
            if velocity.x + velocity.y + velocity.z != 0. {
                client.send_entity_velocity(*id, velocity.x, velocity.y, velocity.z);
            }
        }
        entities.push(entity);
    }
    for entity in entities {
        let mut velocity_f = game.ecs.get_mut::<Physics>(entity)?.deref().clone();
        //velocity_f.move_entity(game, entity, *velocity_f.get_velocity())?;
        let mut velocity = game.ecs.get_mut::<Physics>(entity)?;
        let pos = *game.ecs.get::<Position>(entity)?;
        *velocity = velocity_f;
        let velocity_vector = velocity.get_velocity_mut();
        if velocity_vector.x + velocity_vector.y + velocity_vector.z != 0. {
            velocity_vector.x *= 0.59;
            velocity_vector.y *= 0.59;
            velocity_vector.z *= 0.59;
            if velocity_vector.x < 0.00001 {
                velocity_vector.x = 0.;
            }
            if velocity_vector.y < 0.00001 {
                velocity_vector.y = 0.;
            }
            if velocity_vector.z < 0.00001 {
                velocity_vector.z = 0.;
            }
            velocity.set_modified(false);
        }
    }
    Ok(())
}
