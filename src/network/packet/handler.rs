use hecs::Entity;

use crate::configuration::CONFIGURATION;
use crate::ecs::EntityRef;
use crate::ecs::entities::player::{ChunkLoadQueue, CurrentWorldInfo};
use crate::game::{BlockPosition, ChunkCoords, DamageType, Game, ItemStack, Message, Position};
use crate::network::ids::NetworkID;
use crate::protocol::ClientPlayPacket;
use crate::server::Server;
use crate::world::ChunkLoadData;
use std::cell::RefCell;
use std::sync::Arc;
pub fn handle_packet(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: ClientPlayPacket,
) -> anyhow::Result<()> {
    Ok(())
}