// Feather license in FEATHER_LICENSE.md
//! Sends entity-related packets to clients.
//! Spawn packets, position updates, equipment, animations, etc.

use std::ops::Deref;

use crate::{game::{Game, Position}, ecs::{systems::{SystemExecutor, SysResult}}, server::Server, network::{ids::NetworkID, metadata::Metadata}, events::{SneakEvent, ViewUpdateEvent, ChangeWorldEvent}, world::view::View};

use super::{PreviousPosition, metadata::{META_INDEX_POSE, EntityBitMask, EntityMetadata, META_INDEX_ENTITY_BITMASK}};


pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems
        .group::<Server>()
        .add_system(send_entity_movement)
        .add_system(send_entity_sneak_metadata);
}

/// Sends entity movement packets.
fn send_entity_movement(game: &mut Game, server: &mut Server) -> SysResult {
    let mut to_switch_dim = Vec::new();
    for (entity, (&position, prev_position, &network_id)) in game
        .ecs
        .query::<(
            &Position,
            &mut PreviousPosition,
            &NetworkID,
        )>()
        .iter()
    {
        let mut switched = false;
        if position.world != prev_position.0.world {
            to_switch_dim.push((entity, prev_position.0.world));
            //previous_world_info.1 = previous_world_info.0;
            prev_position.0.world = position.world;
            switched = true;
        }
        if position != prev_position.0 {
            server.broadcast_nearby_with(position,  |client| {
                client.update_entity_position(
                    network_id,
                    position,
                    *prev_position,
                );
            });
            prev_position.0 = position;
        }
    }
    for (entity, old_world) in to_switch_dim {
        let eref = game.ecs.entity(entity)?;
        let world_id = eref.get::<Position>()?.world;
        let client = server.clients.get(&*eref.get::<NetworkID>()?).unwrap();
        client.notify_respawn(&eref, game.worlds.get(&world_id).unwrap().level_dat.lock().world_seed)?;
        drop(eref);
        game.schedule_next_tick(move |game| {
            let eref = game.ecs.entity(entity).ok()?;
            let view = eref.get::<View>().ok()?;
            let oldview = *view;
            let newview = View::empty(world_id);
            drop(view);
            game.ecs.insert_entity_event(entity, ViewUpdateEvent::new(oldview, newview)).ok()?;
            game.ecs.insert_entity_event(entity, ChangeWorldEvent { old_dim: old_world, new_dim: world_id }).ok()?;
            None
        });
    }
    Ok(())
}

/// Sends [SendEntityMetadata](protocol::packets::server::play::SendEntityMetadata) packet for when an entity is sneaking.
fn send_entity_sneak_metadata(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (&position, &SneakEvent { is_sneaking }, &network_id, metadata)) in game
        .ecs
        .query::<(&Position, &SneakEvent, &NetworkID, &mut Metadata)>()
        .iter()
    {
        // The Entity can sneak and sprint at the same time, what happens is that when it stops sneaking you immediately start running again.
        metadata.flags.set(EntityBitMask::CROUCHED, is_sneaking);
        server.broadcast_nearby_with(position, |client| {
            client.send_entity_metadata(false, network_id, metadata.clone());
        });
    }
    Ok(())
}

/* /// Sends [SendEntityMetadata](protocol::packets::server::play::SendEntityMetadata) packet for when an entity is sprinting.
fn send_entity_sprint_metadata(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (&position, &SprintEvent { is_sprinting }, &network_id)) in game
        .ecs
        .query::<(&Position, &SprintEvent, &NetworkID)>()
        .iter()
    {
        let mut metadata = EntityMetadata::entity_base();
        let mut bit_mask = EntityBitMask::empty();

        bit_mask.set(EntityBitMask::SPRINTING, is_sprinting);
        metadata.set(META_INDEX_ENTITY_BITMASK, bit_mask.bits());

        server.broadcast_nearby_with(position, |client| {
            client.send_entity_metadata(network_id, metadata.clone());
        });
    }
    Ok(())
}
 */