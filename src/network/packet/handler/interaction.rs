use std::{
    f32::consts::PI,
    f64::consts::PI as DoublePI,
    ops::{Deref, Mul},
};

use anyhow::bail;
use hecs::Entity;
use rand::Rng;

use crate::{
    aabb::{AABBPool, AABBSize},
    block_entity::{BlockEntity, BlockEntityLoader, SignData},
    configuration::CONFIGURATION,
    ecs::{
        entities::{
            item::{ItemEntity, ItemEntityBuilder},
            living::Health,
            player::{Gamemode, HitCooldown, HotbarSlot, Username, SLOT_HOTBAR_OFFSET, ItemInUse},
        },
        systems::{world::block::update::BlockUpdateManager, SysResult},
        EntityRef,
    },
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, DamageType, Game, Position},
    item::{
        inventory::Inventory,
        inventory_slot::InventorySlot,
        item::{block::ActionResult, BlockUseTarget, ItemRegistry},
        stack::{ItemStack, ItemStackType},
        window::Window,
    },
    network::ids::NetworkID,
    physics::Physics,
    protocol::packets::{
        client::{HoldingChange, PlayerBlockPlacement, PlayerDigging, UpdateSign, UseEntity},
        DiggingStatus, Face, SoundEffectKind,
    },
    server::Server,
    world::chunks::BlockState,
};

//f eather license in FEATHER_LICENSE.md
pub fn handle_held_item_change(
    server: &mut Server,
    player: EntityRef,
    packet: HoldingChange,
) -> SysResult {
    let new_id = packet.slot_id as usize;
    let mut slot = player.get_mut::<HotbarSlot>()?;
    let world = player.get::<Position>()?.world;
    log::trace!("Got player slot change from {} to {}", slot.get(), new_id);

    slot.set(new_id)?;
    drop(slot);
    server.broadcast_equipment_change(&player, world)?;
    Ok(())
}
pub fn drop_item(game: &mut Game, item: ItemStack, position: Position) -> SysResult {
    let mut itempos = position;
    itempos.on_ground = false;
    itempos.y += 1.;
    itempos.y += 0.22;
    let mut item = ItemEntityBuilder::build(game, itempos, item);
    let mut physics = item
        .get_mut::<Physics>()
        .ok_or(anyhow::anyhow!("No physics?"))?;
    let mut f = 0.3;
    let mut f1 = 0.;
    let mut velocity = physics.get_velocity_mut();
    velocity.x =
        (-((position.yaw / 180.) * PI).sin() * ((position.pitch / 180.) * PI).cos() * f) as f64;
    velocity.z =
        (((((position.yaw / 180.) * PI).cos()) * (position.pitch / 180. * PI).cos()) * f) as f64;
    velocity.y = ((-((position.pitch / 180.) * PI).sin()) * f + 0.1) as f64;
    f = 0.02;
    f1 = rand::thread_rng().gen::<f32>() * std::f32::consts::PI * 2.0;
    f *= rand::thread_rng().gen::<f32>();
    velocity.x += (f1 as f64).cos() * f as f64;
    velocity.y +=
        ((rand::thread_rng().gen::<f32>() - rand::thread_rng().gen::<f32>()) * 0.1) as f64;
    velocity.z += (f1 as f64).sin() * f as f64;
    game.spawn_entity(item);
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
    let world = game.ecs.entity(player)?.get::<Position>()?.world;
    let gamemode = *game.ecs.get::<Gamemode>(player)?.deref();
    log::trace!("Got player digging with status {:?}", packet.status);
    let pos = BlockPosition::new(packet.x, packet.y.into(), packet.z, world);
    let res = match packet.status {
        DiggingStatus::StartedDigging => {
            if pos.within_border(CONFIGURATION.world_border)
                && game.ecs.get::<Position>(player)?.distance(&(pos.into())) < 6.0
            {
                let block = match game.block(pos, world) {
                    Some(b) => b,
                    None => {
                        return Ok(());
                    }
                };
                if let Some(block_type) = ItemRegistry::global().get_block(block.b_type) {
                    let creative = gamemode.id() == Gamemode::Creative.id();
                    if creative || block_type.hardness() == 1 {
                        block_type.on_break(game, server, player, pos, packet.face.clone(), world);
                        let _success = game.break_block(pos, world);
                        if !creative {
                            let entity_ref = game.ecs.entity(player)?;
                            let client_id = *entity_ref.get::<NetworkID>()?;
                            let inventory = entity_ref.get_mut::<Window>()?.inner().clone();
                            let hotbar_slot = *entity_ref.get::<HotbarSlot>()?;
                            let slot_id = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
                            let mut slot = inventory.item(slot_id)?;
                            let taken = slot.clone();
                            if let InventorySlot::Filled(taken) = taken.clone() {
                                match taken.item() {
                                    ItemStackType::Item(i) => {
                                        drop(entity_ref);
                                        i.on_dig_with(game, server, player, &mut slot, slot_id, BlockUseTarget { position: pos, world: pos.world, face: packet.face.clone() })?;
                                    },
                                    ItemStackType::Block(_) => (),
                                }
                            }
                            drop(slot);
                            let entity_ref = game.ecs.entity(player)?;
                            let client = server.clients.get(&client_id).unwrap();
                            client.send_window_items(&*entity_ref.get::<Window>()?);
                            let drops = block_type.dropped_items(block, taken);
                            for drop in drops {
                                let mut pos: Position = pos.into();
                                pos.x += 0.5;
                                pos.y += 0.5;
                                let builder = ItemEntityBuilder::build(game, pos, drop);
                                game.spawn_entity(builder);
                            }
                        }
                        let client_id = *game.ecs.get::<NetworkID>(player)?;
                        server.broadcast_nearby_with(pos.into(), |cl| {
                            if cl.id == client_id && !creative {
                                return;
                            }
                            cl.send_effect(pos, SoundEffectKind::BlockBreak, block.b_type as i32);
                        });
                    }
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Err"))
            }
        }
        DiggingStatus::FinishedDigging => {
            if pos.within_border(CONFIGURATION.world_border)
                && game.ecs.get::<Position>(player)?.distance(&(pos.into())) < 6.0
            {
                let block = match game.block(pos, world) {
                    Some(b) => b,
                    None => {
                        return Ok(());
                    }
                };
                if let Some(block_type) = ItemRegistry::global().get_block(block.b_type) {
                    let mut creative = true;
                    if gamemode.id() == Gamemode::Survival.id() {
                        creative = false;
                        let entity_ref = game.ecs.entity(player)?;
                        let client_id = *entity_ref.get::<NetworkID>()?;
                        let inventory = entity_ref.get_mut::<Window>()?.inner().clone();
                        let hotbar_slot = *entity_ref.get::<HotbarSlot>()?;
                        let slot_id = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
                        let mut slot = inventory.item(slot_id)?;
                        let taken = slot.clone();
                        if let InventorySlot::Filled(taken) = taken.clone() {
                            match taken.item() {
                                ItemStackType::Item(i) => {
                                    drop(entity_ref);
                                    i.on_dig_with(game, server, player, &mut slot, slot_id, BlockUseTarget { position: pos, world: pos.world, face: packet.face.clone() })?;
                                },
                                ItemStackType::Block(_) => (),
                            }
                        }
                        drop(slot);
                        let entity_ref = game.ecs.entity(player)?;
                        let client = server.clients.get(&client_id).unwrap();
                        client.send_window_items(&*entity_ref.get::<Window>()?);
                        let drops = block_type.dropped_items(block, taken);
                        for drop in drops {
                            let mut pos: Position = pos.into();
                            pos.x += 0.5;
                            pos.y += 0.5;
                            let builder = ItemEntityBuilder::build(game, pos, drop);
                            game.spawn_entity(builder);
                        }
                    }
                    let client_id = *game.ecs.get::<NetworkID>(player)?;
                    block_type.on_break(game, server, player, pos, packet.face, world);
                    let _success = game.break_block(pos, world);
                    server.broadcast_nearby_with(pos.into(), |cl| {
                        if cl.id == client_id && !creative {
                            return;
                        }
                        cl.send_effect(pos, SoundEffectKind::BlockBreak, block.b_type as i32);
                    });
                    if creative {}
                }
                Ok(())
            } else {
                Err(anyhow::anyhow!("Outside of border"))
            }
        }
        DiggingStatus::DropItem => {
            let entity_ref = game.ecs.entity(player)?;
            let client_id = entity_ref.get::<NetworkID>()?;
            let client = server.clients.get(&client_id).unwrap();
            let inventory = entity_ref.get_mut::<Window>()?;
            let hotbar_slot = entity_ref.get::<HotbarSlot>()?;
            let mut slot = inventory.item(SLOT_HOTBAR_OFFSET + hotbar_slot.get())?;
            let taken = slot.try_take(1);
            drop(slot);
            client.send_window_items(&inventory);
            drop(inventory);
            drop(client_id);
            drop(hotbar_slot);
            let world = entity_ref.get::<Position>()?.world;
            server.broadcast_equipment_change(&entity_ref, world)?;
            match taken {
                InventorySlot::Filled(item) => {
                    let pos = *entity_ref.get::<Position>()?;
                    drop_item(game, item, pos)?;
                }
                InventorySlot::Empty => (),
            }
            Ok(())
        }
        DiggingStatus::ShootArrow => {
            let entity_ref = game.ecs.entity(player)?;
            let client_id = *entity_ref.get::<NetworkID>()?;
            let client = server.clients.get(&client_id).unwrap();
            let inventory = entity_ref.get_mut::<Window>()?.inner().clone();
            let hotbar_slot = *entity_ref.get::<HotbarSlot>()?;
            let slot_id = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
            let mut slot = inventory.item(slot_id)?;
            let mut item = entity_ref.get_mut::<ItemInUse>()?;
            let item2 = item.clone();
            item.0 = InventorySlot::Empty;
            item.1 = 0;
            drop(entity_ref);
            drop(item);
            if let InventorySlot::Filled(item) = item2.0 {
                match item.item() {
                    ItemStackType::Item(i) => {
                        i.on_stop_using(game, server, player, slot, slot_id)?;
                    },
                    ItemStackType::Block(_) => todo!(),
                }
            }
            Ok(())
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
            let client = server
                .clients
                .get(&*game.ecs.get::<NetworkID>(player)?)
                .unwrap();
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
    mut packet: PlayerBlockPlacement,
    player: Entity,
) -> SysResult {
    if matches!(packet.direction, Face::Invalid) {
        let e_ref = game.ecs.entity(player)?;
        if let Some(i) = packet.block_or_item.item_kind() {
            if let ItemStackType::Item(i) = i {
                let window = e_ref.get::<Window>()?.inner().clone();
                let hotbar_slot = e_ref.get::<HotbarSlot>()?;
                let slot = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
                let item = window.item(slot)?;
                drop(e_ref);
                drop(hotbar_slot);
                i.on_use(game, server, item, slot, player, None)?;
            }
        }
        return Ok(());
    }
    let world = game.ecs.get::<Position>(player)?.world;
    packet.pos.world = world;
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
            match block_type.interacted_with(world, game, server, packet.pos, block, player)? {
                ActionResult::SUCCESS => {
                    return Ok(());
                }
                _ => (),
            }
        }
    }
    let e_ref = game.ecs.entity(player)?;
    let window = e_ref.get_mut::<Window>()?;
    let hotbar_slot = e_ref.get::<HotbarSlot>()?;
    let slot = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
    let real_item = window.item(slot)?.clone();
    drop(e_ref);
    drop(window);
    drop(hotbar_slot);
    if let InventorySlot::Filled(item) = &packet.block_or_item {
        if let InventorySlot::Filled(real_item) = real_item {
            let item2 = real_item.clone();
            let (item2, _) = item2.take(1);
            let item2 = item2.unwrap();
            if real_item == item.clone() || item2 == item.clone() {
                // Handle this as a block placement
                let event = match item.item() {
                    ItemStackType::Item(i) => {
                        let e_ref = game.ecs.entity(player)?;
                        let window = e_ref.get::<Window>()?.inner().clone();
                        let hotbar_slot = e_ref.get::<HotbarSlot>()?;
                        let slot = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
                        let item = window.item(slot)?;
                        drop(e_ref);
                        drop(hotbar_slot);
                        i.on_use(
                            game,
                            server,
                            item,
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
                        if let Ok(_) =
                            validate_block_place(game, player, packet.direction.offset(packet.pos))
                        {
                            if let Some(blk) =
                                game.block(packet.direction.offset(packet.pos), world)
                            {
                                if blk.registry_type()?.can_place_over() {
                                    if b.can_place_on(
                                        world,
                                        game,
                                        packet.pos,
                                        packet.direction.clone(),
                                    ) {
                                        let mut e = b.place(
                                            game,
                                            player,
                                            item.clone(),
                                            packet.pos,
                                            packet.direction.clone(),
                                            world,
                                        );
                                        if let Some(val) = &e {
                                            let mut pool = AABBPool::default();
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
                        server.broadcast_equipment_change(&entity_ref, world)?;
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
            } else {
                let entity_ref = game.ecs.entity(player)?;
                let client_id = entity_ref.get::<NetworkID>()?;
                let client = server.clients.get(&client_id).unwrap();
                let window = entity_ref.get_mut::<Window>()?;
                client.send_window_items(&window);
            }
        } else {
            let entity_ref = game.ecs.entity(player)?;
            let client_id = entity_ref.get::<NetworkID>()?;
            let client = server.clients.get(&client_id).unwrap();
            let window = entity_ref.get_mut::<Window>()?;
            client.send_window_items(&window);
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
    let world = game.ecs.get::<Position>(player)?.world;
    if packet.text1.0.len() > 15
        || packet.text2.0.len() > 15
        || packet.text3.0.len() > 15
        || packet.text4.0.len() > 15
    {
        bail!("Text too long!");
    }
    let pos = BlockPosition::new(packet.x, packet.y as i32, packet.z, world);
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
            world,
            entity_ref.get::<BlockEntityLoader>()?.deref().clone(),
            &entity_ref,
        );
    }
    Ok(())
}

pub fn handle_use_entity(
    game: &mut Game,
    server: &mut Server,
    player: Entity,
    packet: UseEntity,
) -> SysResult {
    let target = NetworkID(packet.target);

    if let Some(entity) = game.entity_by_network_id(target) {
        if packet.left_click && !is_creative(game, entity) {
            let mut cooldown = game.ecs.get_mut::<HitCooldown>(player)?;
            if cooldown.0 == 0 {
                let name = game.ecs.get::<Username>(player)?.0.clone();
                let mut health = game.ecs.get_mut::<Health>(entity)?;
                health.damage(1, DamageType::Player { damager: name });
                let pos = *game.ecs.get::<Position>(player)?;
                if let Ok(mut velocity) = game.ecs.get_mut::<Physics>(entity) {
                    let c1 = -(((pos.yaw * PI) / 180.).sin()) * 0.3;
                    let c2 = ((pos.yaw * PI) / 180.).cos() * 0.3;
                    velocity.add_velocity(c1.into(), 0.3, c2.into());
                }
                cooldown.0 = 7;
            }
        }
    }
    Ok(())
}

fn is_creative(game: &mut Game, entity: Entity) -> bool {
    let res: Box<dyn Fn() -> anyhow::Result<bool>> =
        Box::new(move || Ok(*game.ecs.entity(entity)?.get::<Gamemode>()? == Gamemode::Creative));
    res().unwrap_or(false)
}
