use std::ops::Deref;

use rand::RngCore;

use crate::{
    ecs::{
        entities::{
            item::{ItemEntity, ItemEntityData, Life},
            player::{HotbarSlot, Player, SLOT_HOTBAR_OFFSET}, living::Dead,
        },
        systems::{SysResult, SystemExecutor},
    },
    entities::{PreviousPosition, SpawnPacketSender},
    game::{BlockPosition, Game, Position},
    item::{inventory_slot::InventorySlot, item::ItemRegistry, window::Window},
    network::ids::NetworkID,
    physics::Physics,
    server::Server, events::{EntityRemoveEvent, EntityDeathEvent},
};

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    s.group::<Server>()
        .add_system(pickup_items);
    s.add_system(Physics::system)
        .add_system(epic_system)
        .add_system(increment_life).add_system(kill_old_items);
}
fn increment_life(game: &mut Game) -> SysResult {
    for (_, life) in game.ecs.query::<&mut Life>().iter() {
        life.0 += 1;
    }
    Ok(())
}

fn epic_system(game: &mut Game) -> SysResult {
    let mut items = Vec::new();
    for (entity, (_, _, _)) in game
        .ecs
        .query::<(&ItemEntity, &mut Physics, &Position)>()
        .iter()
    {
        items.push(entity);
    }
    for entity in items {
        let entity_ref = game.ecs.entity(entity)?;
        let mut fakephysics = entity_ref.get_mut::<Physics>()?.deref().clone();
        let pos = *entity_ref.get::<Position>()?;
        log::info!("ITem at {:?}", pos);
        fakephysics.add_velocity(0., -0.03, 0.);
        drop(entity_ref);
        fakephysics.move_entity(game, entity, *fakephysics.get_velocity())?;
        let entity_ref = game.ecs.entity(entity)?;
        let mut physics = entity_ref.get_mut::<Physics>()?;
        *physics = fakephysics;
        let mut f = 0.98;
        if pos.on_ground {
            f = 0.58;
            let pos: BlockPosition = pos.into();
            let id = game.block_id_at(pos.offset(0, -1, 0));
            if let Some(id) = ItemRegistry::global().get_block(id) {
                f = id.slipperiness() * 0.98;
            }
        }
        let velocity = physics.get_velocity_mut();
        velocity.x *= f;
        velocity.y *= 0.9800000190734863;
        velocity.z *= f;
        if pos.on_ground {
            velocity.y *= -0.5;
        }
    }
    Ok(())
}
// TODO: make better
fn merge_items(game: &mut Game, server: &mut Server) -> SysResult {
    let mut item_entities = Vec::new();
    for (entity, (_, pos)) in game.ecs.query::<(&ItemEntity, &Position)>().iter() {
        item_entities.push((entity, *pos));
    }
    item_entities.retain(|(e, p)| {
        game.ecs.get::<EntityRemoveEvent>(*e).is_err()
    });
    let mut to_remove = Vec::new();
    let mut to_update = Vec::new();
    for (entity, pos) in item_entities.iter() {
        if to_remove.contains(entity) {
            continue;
        }
        let mut our_data = game.ecs.entity(*entity)?.get::<ItemEntityData>()?.deref().clone();
        let our_id = *game.ecs.get::<NetworkID>(*entity)?;
        for (other_entity, other_pos) in item_entities.iter() {
            if to_remove.contains(other_entity) {
                continue;
            }
            if our_id == *game.ecs.get::<NetworkID>(*other_entity)? {
            } else if entity != other_entity && pos.distance(other_pos) < 1.0 {
                let other_entity_data = game.ecs.entity(*other_entity)?.get::<ItemEntityData>()?;
                let ood = other_entity_data.deref().clone();
                drop(other_entity_data);
                if our_data.0.merge_with(ood.0).is_ok() {
                    to_remove.push(*other_entity);
                    to_update.push(*entity);
                }
            }
        }
        *game.ecs.entity(*entity)?.get_mut::<ItemEntityData>()? = our_data;
    }
    for entity in to_remove.iter() {
        game.remove_entity(*entity)?;
    }
    for entity in to_update {
        if to_remove.contains(&entity) {
            continue;
        }
        let entity = game.ecs.entity(entity)?;
        if entity.get::<EntityRemoveEvent>().is_err() {
            let id = *entity.get::<NetworkID>()?;
            let pos = *entity.get::<Position>()?;
            let sps = entity.get::<SpawnPacketSender>()?;
            let sch = game.scheduler.clone();
            let ticks = game.ticks;
            server.broadcast_nearby_with(pos, |cl| {
                cl.unload_entity(id);
                sps.send(sch.clone(), ticks, &entity, cl).unwrap();
            });
        }
    }
    Ok(())
}
fn kill_old_items(game: &mut Game) -> SysResult {
    let mut to_kill = Vec::new();
    for (entity, (_, life)) in game.ecs.query::<(&ItemEntity, &Life)>().iter() {
        if life.0 > 6000 {
            to_kill.push(entity);
        }
    }
    for entity in to_kill {
        game.remove_entity(entity)?;
    }
    Ok(())
}
fn pickup_items(game: &mut Game, server: &mut Server) -> SysResult {
    let mut item_entities = Vec::new();
    for (entity, (_, pos, life)) in game.ecs.query::<(&ItemEntity, &Position, &Life)>().iter() {
        if life.0 > 5 {
            item_entities.push((entity, *pos));
        }
    }
    item_entities.retain(|(e, _)| {
        game.ecs.get::<EntityDeathEvent>(*e).is_err()
    });
    let mut to_despawn = Vec::new();
    for (player, (_, pos, &net_id)) in game.ecs.query::<(&Player, &Position, &NetworkID)>().iter() {
        for (entity, item_pos) in item_entities.iter() {
            if pos.world == item_pos.world && pos.distance(item_pos) < 1.1 {
                to_despawn.push((*entity, player, net_id));
            }
        }
        item_entities.retain(|(e, _)| {
            let mut res = true;
            for (entity, _, _) in to_despawn.iter() {
                if *entity == *e {
                    res = false;
                    break;
                }
            }
            res
        });
    }
    for (entity, player_entity, plr_id) in to_despawn {
        let client = server.clients.get(&plr_id).unwrap();
        let entity_ref = game.ecs.entity(player_entity)?;
        if entity_ref.get::<Dead>().is_err() {
            let mut inventory = entity_ref.get_mut::<Window>()?;
            let hotbar_slot = entity_ref.get::<HotbarSlot>()?;
            let world = entity_ref.get::<Position>()?.world;
            inventory.insert_item(InventorySlot::Filled(
                game.ecs.get::<ItemEntityData>(entity)?.0.clone(),
            ))?;
            client.send_window_items(&inventory);
            drop(inventory);
            server.broadcast_equipment_change(&entity_ref)?;
            drop(hotbar_slot);
            drop(entity_ref);
            let id = *game.ecs.get::<NetworkID>(entity)?;
            let pos = *game.ecs.get::<Position>(entity)?;
            server.broadcast_nearby_with(pos, |cl| {
                cl.send_collect_item(id, plr_id);
            });
            game.remove_entity(entity)?;
        }
    }
    Ok(())
}
