use std::ops::Deref;

use hecs::Entity;

use crate::{
    ecs::{
        entities::player::{CurrentWorldInfo, HotbarSlot, SLOT_HOTBAR_OFFSET, Gamemode},
        systems::SysResult,
        EntityRef,
    },
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{inventory::Inventory, inventory_slot::InventorySlot, window::Window},
    network::ids::NetworkID,
    protocol::packets::{
        client::{HoldingChange, PlayerBlockPlacement, PlayerDigging},
        DiggingStatus, Face, SoundEffectKind,
    },
    server::Server,
};

//f eather license in FEATHER_LICENSE.md
pub fn handle_held_item_change(
    server: &mut Server,
    player: EntityRef,
    packet: HoldingChange,
) -> SysResult {
    let new_id = packet.slot_id as usize;
    let mut slot = player.get_mut::<HotbarSlot>()?;

    log::trace!("Got player slot change from {} to {}", slot.get(), new_id);

    slot.set(new_id)?;
    drop(slot);
    server.broadcast_equipment_change(&player)?;
    Ok(())
}
/// Handles the Player Digging packet sent for the following
/// actions:
/// * Breaking blocks.
/// * Dropping items.
/// * Shooting arrows.
/// * Eating.
/// * Swapping items between the main and off hand.
pub fn handle_player_digging(
    game: &mut Game,
    server: &mut Server,
    packet: PlayerDigging,
    player: Entity,
) -> SysResult {
    let world = game.ecs.entity(player)?.get::<CurrentWorldInfo>()?.world_id;
    let gamemode = game.ecs.entity(player)?.get::<Gamemode>()?.deref().clone();
    log::trace!("Got player digging with status {:?}", packet.status);
    let pos = BlockPosition::new(packet.x, packet.y.into(), packet.z);
    match packet.status {
        DiggingStatus::StartedDigging => {
            if gamemode.id() == Gamemode::Creative.id() {
                let block = match game.block(pos, world) {
                    Some(b) => b.b_type,
                    None => {
                        return Ok(());
                    }
                };
                let _success = game.break_block(pos, world);
                server.broadcast_effect(SoundEffectKind::BlockBreak, pos, block as i32);
            }
            Ok(())
        }
        DiggingStatus::FinishedDigging => {
            let block = match game.block(pos, world) {
                Some(b) => b.b_type,
                None => {
                    return Ok(());
                }
            };
            let _success = game.break_block(pos, world);
            server.broadcast_effect(SoundEffectKind::BlockBreak, pos, block as i32);
            Ok(())
        }
        _ => Ok(()),
    }
}
/// Handles the player block placement packet.
pub fn handle_player_block_placement(
    game: &mut Game,
    server: &mut Server,
    packet: PlayerBlockPlacement,
    player: Entity,
) -> SysResult {
    if matches!(packet.direction, Face::Invalid) {
        return Ok(());
    }
    let world = game.ecs.get::<CurrentWorldInfo>(player).unwrap().world_id;
    let block_kind = {
        let result = game.block(packet.pos, world);
        match result {
            Some(block) => block.b_type,
            None => {
                let client_id = game.ecs.get::<NetworkID>(player).unwrap();

                let client = server.clients.get(&client_id).unwrap();

                client.disconnect("Attempted to interact with an unloaded block!");

                anyhow::bail!(
                    "Player attempted to interact with an unloaded block. {:?}",
                    packet
                )
            }
        }
    };
    if let InventorySlot::Filled(item) = packet.block_or_item {
        // Handle this as a block placement
        let event = BlockPlacementEvent {
            held_item: item,
            location: packet.pos,
            face: packet.direction,
            world,
        };

        let entity_ref = game.ecs.entity(player)?;
        if entity_ref.get::<Gamemode>()?.id() == Gamemode::Survival.id() {
            let inventory = entity_ref.get_mut::<Window>()?;
            let hotbar_slot = entity_ref.get::<HotbarSlot>()?;
            let client_id = entity_ref.get::<NetworkID>()?;
            let mut slot = inventory.item(SLOT_HOTBAR_OFFSET + hotbar_slot.get())?;
            slot.try_take(1);
            let client = server.clients.get(&client_id).unwrap();
            drop(slot);
            client.send_window_items(&inventory);
            drop(inventory);
            server.broadcast_equipment_change(&entity_ref)?;
        }
        game.ecs.insert_entity_event(player, event)?;
    }
    Ok(())
}
