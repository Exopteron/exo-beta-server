use std::ops::Deref;

use anyhow::bail;
use hecs::Entity;

use crate::{
    aabb::{AABBPool, AABBSize},
    block_entity::{BlockEntity, BlockEntityLoader, SignData},
    ecs::{
        entities::player::{CurrentWorldInfo, Gamemode, HotbarSlot, SLOT_HOTBAR_OFFSET},
        systems::{SysResult, world::block::update::BlockUpdateManager},
        EntityRef,
    },
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        inventory::Inventory,
        inventory_slot::InventorySlot,
        item::{block::ActionResult, BlockUseTarget, ItemRegistry},
        stack::ItemStackType,
        window::Window,
    },
    network::ids::NetworkID,
    protocol::packets::{
        client::{HoldingChange, PlayerBlockPlacement, PlayerDigging, UpdateSign},
        DiggingStatus, Face, SoundEffectKind,
    },
    server::Server,
    world::chunks::BlockState, configuration::CONFIGURATION,
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
    let res = match packet.status {
        DiggingStatus::StartedDigging => {
            if gamemode.id() == Gamemode::Creative.id() {
                if pos.within_border(CONFIGURATION.world_border) {
                    let block = match game.block(pos, world) {
                        Some(b) => b.b_type,
                        None => {
                            return Ok(());
                        }
                    };
                    if let Some(block_type) = ItemRegistry::global().get_block(block) {
                        block_type.on_break(game, server, player, pos, packet.face, world);
                        let _success = game.break_block(pos, world);
                        server.broadcast_effect(SoundEffectKind::BlockBreak, pos, block as i32);
                    }
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("Outside of border"))
                }
            } else {
                Ok(())
            }
        }
        DiggingStatus::FinishedDigging => {
            if pos.within_border(CONFIGURATION.world_border) {
                let block = match game.block(pos, world) {
                    Some(b) => b.b_type,
                    None => {
                        return Ok(());
                    }
                };
                if let Some(block_type) = ItemRegistry::global().get_block(block) {
                    block_type.on_break(game, server, player, pos, packet.face, world);
                    let _success = game.break_block(pos, world);
                    server.broadcast_effect(SoundEffectKind::BlockBreak, pos, block as i32);
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Outside of border"))
            }
        }
        _ => Ok(()),
    };
    match res {
        Ok(_) => (),
        Err(_) => {
            let block = match game.block(pos, world) {
                Some(b) => b,
                None => {
                    return Ok(());
                }
            };
            let client = server.clients.get(&*game.ecs.get::<NetworkID>(player)?).unwrap();
            client.send_block_change(pos, block);
        }
    }
    Ok(())
}
fn validate_block_place(
    game: &mut Game,
    player: Entity,
    position: BlockPosition,
) -> anyhow::Result<()> {
    if !position.within_border(CONFIGURATION.world_border) {
        bail!("Outside of border");
    }
    if game
        .ecs
        .get::<Position>(player)?
        .distance(&(position.into()))
        > 6.0
    {
        bail!("Too far away from place");
    }
    Ok(())
}
/// Handles the player block placement packet.
pub fn handle_player_block_placement(
    game: &mut Game,
    server: &mut Server,
    packet: PlayerBlockPlacement,
    player: Entity,
) -> SysResult {
    if matches!(packet.direction, Face::Invalid) {
        let e_ref = game.ecs.entity(player)?;
        if let Some(i) = packet.block_or_item.item_kind() {
            if let ItemStackType::Item(i) = i {
                let window = e_ref.get_mut::<Window>()?;
                let hotbar_slot = e_ref.get::<HotbarSlot>()?;
                let slot = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
                drop(e_ref);
                drop(window);
                drop(hotbar_slot);
                i.on_use(game, server, slot, player, None)?;
            }
        }
        return Ok(());
    }
    let world = game.ecs.get::<CurrentWorldInfo>(player).unwrap().world_id;
    let block_kind = {
        let result = game.block(packet.direction.offset(packet.pos), world);
        match result {
            Some(block) => block,
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
    if let Some(block) = game.block(packet.pos, world) {
        if let Some(block_type) = ItemRegistry::global().get_block(block.b_type) {
            match block_type.interacted_with(world, game, server, packet.pos, block, player) {
                ActionResult::SUCCESS => {
                    return Ok(());
                }
                _ => (),
            }
        }
    }
    if let InventorySlot::Filled(item) = &packet.block_or_item {
        // Handle this as a block placement
        // TODO don't trust the player. But we can for now :D
        let event = match item.item() {
            ItemStackType::Item(i) => {
                let e_ref = game.ecs.entity(player)?;
                let window = e_ref.get_mut::<Window>()?;
                let hotbar_slot = e_ref.get::<HotbarSlot>()?;
                let slot = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
                drop(e_ref);
                drop(window);
                drop(hotbar_slot);
                i.on_use(
                    game,
                    server,
                    slot,
                    player,
                    Some(BlockUseTarget {
                        position: packet.pos,
                        face: packet.direction.clone(),
                        world,
                    }),
                )?;
                return Ok(());
            }
            ItemStackType::Block(b) => {
                if let Ok(_) = validate_block_place(game, player, packet.direction.offset(packet.pos)) {
                    if let Some(blk) = game.block(packet.direction.offset(packet.pos), world) {
                        if blk.registry_type()?.can_place_over() {
                            if b.can_place_on(world, game, packet.pos, packet.direction.clone()) {
                                let mut e = b.place(
                                    game,
                                    player,
                                    item.clone(),
                                    packet.pos,
                                    packet.direction.clone(),
                                    world,
                                );
                                if let Some(val) = &e {
                                    let mut pool = AABBPool::new();
                                    let block_bounding_box = b.collision_box(
                                        BlockState::new(
                                            val.held_item.id() as u8,
                                            val.held_item.damage_taken() as u8,
                                        ),
                                        packet.direction.offset(packet.pos),
                                    );
                                    for (_, (position, bounding_box)) in
                                        game.ecs.query::<(&Position, &AABBSize)>().iter()
                                    {
                                        pool.add(bounding_box.get(position));
                                    }
                                    if let Some(block_bounding_box) = block_bounding_box {
                                        if pool.intersects(&block_bounding_box).len() != 0 {
                                            e = None;
                                        }
                                    }
                                }
                                e
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        if let Some(event) = event {
            let entity_ref = game.ecs.entity(player)?;
            if entity_ref.get::<Gamemode>()?.id() == Gamemode::Survival.id() {
                let client_id = entity_ref.get::<NetworkID>()?;
                let client = server.clients.get(&client_id).unwrap();
                let inventory = entity_ref.get_mut::<Window>()?;
                let hotbar_slot = entity_ref.get::<HotbarSlot>()?;
                let mut slot = inventory.item(SLOT_HOTBAR_OFFSET + hotbar_slot.get())?;
                slot.try_take(1);
                drop(slot);
                client.send_window_items(&inventory);
                drop(inventory);
                server.broadcast_equipment_change(&entity_ref)?;
            }
            let id = match event.held_item.item() {
                ItemStackType::Item(_) => return Ok(()),
                ItemStackType::Block(b) => b.id(),
            };
            let block = BlockState {
                b_type: id,
                b_metadata: event.held_item.damage_taken() as u8,
                b_light: 0,
                b_skylight: 15,
            };
            game.set_block(event.location, block, world);
            let mut update_manager = game.objects.get_mut::<BlockUpdateManager>()?;
            update_manager.add((event.location, world));
        } else {
            let entity_ref = game.ecs.entity(player)?;
            let client_id = entity_ref.get::<NetworkID>()?;
            let client = server.clients.get(&client_id).unwrap();
            client.send_block_change(
                packet.direction.offset(packet.pos),
                BlockState::new(block_kind.b_type, block_kind.b_metadata),
            );
        }
    }
    Ok(())
}
pub fn handle_update_sign(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: UpdateSign,
) -> SysResult {
    if packet.text1.0.len() > 15
        || packet.text2.0.len() > 15
        || packet.text3.0.len() > 15
        || packet.text4.0.len() > 15
    {
        bail!("Text too long!");
    }
    let pos = BlockPosition::new(packet.x, packet.y as i32, packet.z);
    let mut to_sync = Vec::new();
    for (entity, (sign_data, block_entity)) in
        game.ecs.query::<(&mut SignData, &BlockEntity)>().iter()
    {
        if block_entity.0 == pos {
            sign_data.0[0] = packet.text1.0.clone();
            sign_data.0[1] = packet.text2.0.clone();
            sign_data.0[2] = packet.text3.0.clone();
            sign_data.0[3] = packet.text4.0.clone();
        }
        to_sync.push(entity);
    }
    for entity in to_sync {
        let entity_ref = game.ecs.entity(entity)?;
        server.sync_block_entity(
            *entity_ref.get::<Position>()?,
            entity_ref.get::<BlockEntityLoader>()?.deref().clone(),
            &entity_ref,
        );
    }
    Ok(())
}
