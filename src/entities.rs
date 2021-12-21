use hecs::EntityBuilder;

use crate::{ecs::{EntityRef, systems::{SysResult, SystemExecutor}, entities::player::Username}, server::Client, game::{Position, Game}, network::ids::NetworkID};

// feather license in FEATHER_LICENSE.md
pub mod spawn_packet;
pub mod entity_updates;
pub mod metadata;
/// Initial state of an entity passed
/// to `Game::create_entity_builder`.
#[derive(Debug)]
pub enum EntityInit {
    Player
}

pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    entity_updates::register(game, systems);
    spawn_packet::register(game, systems);
}
/// Component that sends the spawn packet for an entity
/// using its components.
pub struct SpawnPacketSender(fn(&EntityRef, &Client) -> SysResult);

impl SpawnPacketSender {
    pub fn send(&self, entity: &EntityRef, client: &Client) -> SysResult {
        let res = (self.0)(entity, client);
        if res.is_ok() {
            if client.username() != entity.get::<Username>()?.0 {
                client.send_exact_entity_position(*entity.get::<NetworkID>()?, *entity.get::<Position>()?);
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

    builder
        .add(PreviousPosition(prev_position));
    add_spawn_packet(builder, init);
}

fn add_spawn_packet(builder: &mut EntityBuilder, init: &EntityInit) {
    // TODO: object entities spawned with Spawn Entity
    // (minecarts, items, ...)
    let spawn_packet = match init {
        EntityInit::Player => spawn_player,
        _ => spawn_living_entity,
    };
    builder.add(SpawnPacketSender(spawn_packet));
}

fn spawn_player(entity: &EntityRef, client: &Client) -> SysResult {
    let network_id = *entity.get::<NetworkID>()?;
    let pos = *entity.get::<Position>()?;
    let name = &*entity.get::<Username>()?;
    client.send_player(network_id, name, pos);
    client.send_entity_equipment(entity)?;
    Ok(())
}

fn spawn_living_entity(entity: &EntityRef, client: &Client) -> SysResult {
    unimplemented!();
/*     let network_id = *entity.get::<NetworkId>()?;
    let uuid = *entity.get::<Uuid>()?;
    let pos = *entity.get::<Position>()?;
    let kind = *entity.get::<EntityKind>()?;

    client.send_living_entity(network_id, uuid, pos, kind);
    Ok(()) */
}
