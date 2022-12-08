use std::{ops::Deref, rc::Rc, cell::RefCell};

use hecs::EntityBuilder;

use crate::{
    block_entity::{BlockEntity, BlockEntityLoader},
    ecs::{
        entities::{player::Username, item::ItemEntityData, falling_block::FallingBlockEntityData},
        systems::{SysResult, SystemExecutor},
        EntityRef,
    },
    game::{Game, Position, Scheduler},
    network::{ids::NetworkID, metadata::Metadata},
    server::{Client, Server}, status_effects::StatusEffectsManager, item::inventory_slot::InventorySlot, protocol::packets::{ObjectVehicleKind, EnumMobType},
};

// feather license in FEATHER_LICENSE.md
pub mod entity_updates;
pub mod metadata;
pub mod spawn_packet;
/// Initial state of an entity passed
/// to `Game::create_entity_builder`.
#[derive(Debug)]
pub enum EntityInit {
    Player,
    BlockEntity,
    Item,
    FallingBlock,
    Mob
}

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    entity_updates::register(game, systems);
    spawn_packet::register(game, systems);
}
/// Component that sends the spawn packet for an entity
/// using its components.
pub struct SpawnPacketSender(fn(Rc<RefCell<Scheduler>>, u128, &EntityRef, &Client) -> SysResult);

impl SpawnPacketSender {
    pub fn send(&self, scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
        let res = (self.0)(scheduler, ticks, entity, client);
        if let Ok(username) = entity.get::<Username>() {
            if let Ok(_) = entity.get::<Position>() {
                if let Ok(_) = entity.get::<NetworkID>() {
                    if res.is_ok() && client.username() != username.0 {
                        client.send_exact_entity_position(
                            *entity.get::<NetworkID>()?,
                            *entity.get::<Position>()?,
                        );
                    }
                }
            }
        }
        res
    }
}

/// Stores the [`Position`] of an entity on
/// the previous tick. Used to determine
/// when to send movement updates.
#[derive(Copy, Clone, Debug)]
pub struct PreviousPosition(pub Position);

pub fn add_entity_components(builder: &mut EntityBuilder, init: &EntityInit) {
    if !builder.has::<NetworkID>() {
        builder.add(NetworkID::new());
    }

    // can't panic because this is only called after both position and onground is added to all entities.
    // Position is added in the caller of this function and on_ground is added in the
    // build default function. All entity builder functions call the build default function.
    let prev_position = *builder.get::<Position>().unwrap();

    builder.add(PreviousPosition(prev_position));
    add_spawn_packet(builder, init);
}

fn add_spawn_packet(builder: &mut EntityBuilder, init: &EntityInit) {
    // TODO: object entities spawned with Spawn Entity
    // (minecarts, items, ...)
    let spawn_packet = match init {
        EntityInit::Player => spawn_player,
        EntityInit::BlockEntity => spawn_block_entity,
        EntityInit::Item => spawn_item_entity,
        EntityInit::FallingBlock => spawn_falling_block,
        EntityInit::Mob => spawn_mob,
        _ => spawn_living_entity,
    };
    builder.add(SpawnPacketSender(spawn_packet));
}
fn spawn_falling_block(scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
    let network_id = *entity.get::<NetworkID>()?;
    let pos = *entity.get::<Position>()?;
    let block_type = entity.get::<FallingBlockEntityData>()?;
    let kind = match *block_type {
        FallingBlockEntityData::Gravel => ObjectVehicleKind::FallingGravel,
        FallingBlockEntityData::Sand => ObjectVehicleKind::FallingSand,
    };
    client.spawn_object_vehicle(network_id, kind, pos);
    Ok(())
}
fn spawn_player(scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
    let network_id = *entity.get::<NetworkID>()?;
    let pos = *entity.get::<Position>()?;
    let name = &*entity.get::<Username>()?;
    let status = entity.get::<StatusEffectsManager>()?;
    for effect in status.get_effects().values() {
        effect.show_client(entity, client)?;
    }
    drop(status);
    client.send_player(network_id, name, pos);
    client.send_entity_equipment(entity)?;
    Ok(())
}
fn spawn_item_entity(scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
    let net_id = *entity.get::<NetworkID>()?;
    let pos = *entity.get::<Position>()?;
    let data = entity.get::<ItemEntityData>()?.deref().clone(); 
    client.spawn_dropped_item(net_id, pos, InventorySlot::Filled(data.0));
    Ok(())
}
fn spawn_block_entity(scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
    log::info!("Spawning be");
    let mut sch = scheduler.borrow_mut();
    let entity = entity.entity();
    let client = client.id;
    sch.schedule_task(ticks + 1, move |game| {
        if let Ok(entity) = game.ecs.entity(entity) {
            let s = game.objects.get::<Server>().unwrap();
            if let Some(client) = s.clients.get(&client) {
                if let Ok(loader) = entity.get::<BlockEntityLoader>() {
                    log::info!("is a loader");
                    loader.load(client, &entity);
                }
            }
        }
        None
    });
    Ok(())
}
fn spawn_mob(scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
    let network_id = *entity.get::<NetworkID>()?;
    let pos = *entity.get::<Position>()?;
    let kind = *entity.get::<EnumMobType>()?;
    let meta = &*entity.get::<Metadata>()?;

    client.spawn_mob(network_id, kind, pos, meta.clone());
    Ok(()) 
}
fn spawn_living_entity(scheduler: Rc<RefCell<Scheduler>>, ticks: u128, entity: &EntityRef, client: &Client) -> SysResult {
    unimplemented!();
    /*     let network_id = *entity.get::<NetworkId>()?;
    let uuid = *entity.get::<Uuid>()?;
    let pos = *entity.get::<Position>()?;
    let kind = *entity.get::<EntityKind>()?;

    client.send_living_entity(network_id, uuid, pos, kind);
    Ok(()) */
}
