// Feather license in FEATHER_LICENSE.md
//! Sends entity-related packets to clients.
//! Spawn packets, position updates, equipment, animations, etc.

use crate::{game::{Game, Position}, ecs::systems::{SystemExecutor, SysResult}, server::Server, network::{ids::NetworkID, metadata::Metadata}, events::SneakEvent};

use super::{PreviousPosition, metadata::{META_INDEX_POSE, EntityBitMask, EntityMetadata, META_INDEX_ENTITY_BITMASK}};


pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems
        .group::<Server>()
        .add_system(send_entity_movement)
        .add_system(send_entity_sneak_metadata);
}

/// Sends entity movement packets.
fn send_entity_movement(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (&position, prev_position, &network_id)) in game
        .ecs
        .query::<(
            &Position,
            &mut PreviousPosition,
            &NetworkID,
        )>()
        .iter()
    {
        if position != prev_position.0 {
            server.broadcast_nearby_with(position, |client| {
                client.update_entity_position(
                    network_id,
                    position,
                    *prev_position,
                );
            });
            prev_position.0 = position;
        }
    }
    Ok(())
}

/// Sends [SendEntityMetadata](protocol::packets::server::play::SendEntityMetadata) packet for when an entity is sneaking.
fn send_entity_sneak_metadata(game: &mut Game, server: &mut Server) -> SysResult {
    for (_, (&position, &SneakEvent { is_sneaking }, &network_id)) in game
        .ecs
        .query::<(&Position, &SneakEvent, &NetworkID)>()
        .iter()
    {
        let mut metadata = Metadata::new();
        let mut bit_mask = EntityBitMask::empty();

        // The Entity can sneak and sprint at the same time, what happens is that when it stops sneaking you immediately start running again.
        bit_mask.set(EntityBitMask::CROUCHED, is_sneaking);
        metadata.insert_byte_idx(bit_mask.bits(), META_INDEX_ENTITY_BITMASK);
        server.broadcast_nearby_with(position, |client| {
            client.send_entity_metadata(network_id, metadata.clone());
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