use crate::game::items::crafting::*;
use crate::game::items::ItemRegistry;
use crate::game::PlayerRef;
use crate::game::{BlockPosition, DamageType, Game, ItemStack, Message, Position};
use crate::network::ids::EntityID;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::server::Server;
use std::cell::RefCell;
use std::sync::Arc;
pub fn handle_packet(
    game: &mut Game,
    server: &mut Server,
    mut player: Arc<PlayerRef>,
    packet: ClientPacket,
) -> anyhow::Result<()> {
    match packet {
        ClientPacket::PlayerPacket(packet) => {
            if player.is_dead() {
                return Ok(());
            }
            if packet.on_ground == false && player.get_checking_fall() {
                //log::debug!("Off ground");
                player.set_offground_height(player.get_position_clone().y as f32);
                player.set_checking_fall(false);
            } else if packet.on_ground == true {
                player.set_checking_fall(true);
            }
            let mut pos = player.get_position();
            pos.on_ground = packet.on_ground; // player.position.
            let mut pos = pos.clone();
            player.set_last_position(pos);
            player.set_position(pos);
        }
        ClientPacket::PlayerLookPacket(packet) => {
            if player.is_dead() {
                return Ok(());
            }
            if packet.on_ground == false && player.get_checking_fall() {
                //log::debug!("Off ground");
                player.set_offground_height(player.get_position_clone().y as f32);
                player.set_checking_fall(false);
            } else if packet.on_ground == true {
                player.set_checking_fall(true);
            }
            let mut pos = player.get_position();
            pos.yaw = packet.yaw;
            pos.pitch = packet.pitch;
            pos.on_ground = packet.on_ground;
            let mut pos = pos.clone();
            player.set_last_position(pos);
            player.set_position(pos);
        }
        ClientPacket::PlayerPositionPacket(packet) => {
            if player.is_dead() {
                return Ok(());
            }
            if packet.on_ground == false && player.get_checking_fall() {
                //log::debug!("Off ground");
                player.set_offground_height(player.get_position_clone().y as f32);
                player.set_checking_fall(false);
            } else if packet.on_ground == true {
                player.set_checking_fall(true);
            }
            let mut bad_move = false;
            for y in 0..1 {
                if let Some(block) = game.world.get_block(packet.x.floor() as i32, packet.y.floor() as i32, packet.z.floor() as i32) {
                    if let Some(registry_block) = ItemRegistry::global().get_item(block.b_type as i16) {
                        if let Some(registry_block) = registry_block.get_item().as_block() {
                            if registry_block.is_solid() {
                                bad_move = true;
                            }
                        }
                    }
                }
            }
            if !bad_move {
                let mut pos = player.get_position();
                pos.x = packet.x;
                pos.y = packet.y;
                pos.stance = packet.stance;
                pos.z = packet.z;
                pos.on_ground = packet.on_ground;
                let pos = pos.clone();
                let pos = pos.clone();
                player.set_last_position(pos);
                player.set_position(pos);
            } else {
                let pos = player.get_position();
                //log::info!("Denied. TPing to {:?}", pos);
                player.write_packet(ServerPacket::PlayerPositionAndLook { x: pos.x, stance: pos.stance, y: pos.y, z: pos.z, yaw: pos.yaw, pitch: pos.pitch, on_ground: pos.on_ground });
                player.set_offground_height(0.);
            }
        }
        ClientPacket::PlayerPositionAndLookPacket(packet) => {
            if player.is_dead() {
                return Ok(());
            }
            if packet.on_ground == false && player.get_checking_fall() {
                //log::debug!("Off ground");
                player.set_offground_height(player.get_position_clone().y as f32);
                player.set_checking_fall(false);
            } else if packet.on_ground == true {
                player.set_checking_fall(true);
            }
            let mut bad_move = false;
            for y in 0..1 {
                if let Some(block) = game.world.get_block(packet.x.floor() as i32, packet.y.floor() as i32, packet.z.floor() as i32) {
                    if let Some(registry_block) = ItemRegistry::global().get_item(block.b_type as i16) {
                        if let Some(registry_block) = registry_block.get_item().as_block() {
                            if registry_block.is_solid() {
                                bad_move = true;
                            }
                        }
                    }
                }
            }
            if !bad_move {
                let mut pos = player.get_position();
                pos.yaw = packet.yaw;
                pos.pitch = packet.pitch;
                pos.x = packet.x;
                pos.y = packet.y;
                pos.stance = packet.stance;
                pos.z = packet.z;
                pos.on_ground = packet.on_ground;
                let pos = pos.clone();
                player.set_last_position(pos);
                player.set_position(pos);
            } else {
                let pos = player.get_position();
                //log::info!("Denied. TPing to {:?}", pos);
                player.write_packet(ServerPacket::PlayerPositionAndLook { x: pos.x, stance: pos.stance, y: pos.y, z: pos.z, yaw: pos.yaw, pitch: pos.pitch, on_ground: pos.on_ground });
                player.set_offground_height(0.);
            }
        }
        ClientPacket::EntityAction(packet) => match packet.action {
            1 => {
                player.set_crouching(true);
            }
            2 => {
                player.set_crouching(false);
            }
            _ => {}
        },
        ClientPacket::ChatMessage(mut message) => {
            if message.message.len() > 256 {
                return Ok(());
            }
            use crate::game::entities::*;
            if message.message == "panicmeballs" {
                panic!("Panic!");
                //game.spawn_entity(Box::new(crate::game::entities::item_entity::ItemEntity::new(player.get_position_clone(), game.ticks, ItemStack::new(1, 0, 1))));
            }
            if message.message == "makeaslime" {
                game.spawn_entity(Box::new(
                    slime_entity::SlimeEntity::new(
                        player.get_position_clone(),
                        game.ticks,
                    ),
                ));
            }
/*             if message.message == "chunksave" {
                game.world.to_file("world");
            } */
/*             if message.message == "chunkload" {
                game.world = crate::chunks::World::from_file("world").unwrap();
            } */
            /*             let mut inv = player.get_inventory();
            let mut slot = inv
                .get_slot(39)
                .expect("Player doesn't have expected slot!"); /*
                                                                              .inventory
                                                               .items
                                                               .get_mut(&40)
                                                               */
            slot.id = 5;
            slot.count = 64;
            let mut slot = inv
                .get_slot(40)
                .expect("Player doesn't have expected slot!"); /*
                                                                              .inventory
                                                               .items
                                                               .get_mut(&40)
                                                               */
            slot.id = 50;
            slot.count = 64;
            let mut slot = inv
                .get_slot(41) /*
                                .inventory
                    .items
                    .get_mut(&41)
                */
                .expect("Player doesn't have expected slot!");
            slot.id = 3;
            slot.count = 64;
            let mut slot = inv
                .get_slot(42) /*
                                .inventory
                    .items
                    .get_mut(&42)
                */
                .expect("Player doesn't have expected slot!");
            slot.id = 285;
            slot.count = 1;
            drop(inv); */
            //player.sync_inventory();
            if message.message.starts_with("/") {
                message.message.remove(0);
                log::debug!(
                    "{} issued server command /{}",
                    player.get_username(),
                    message.message
                );
                use std::ops::DerefMut;
                //log::debug!("A");
                let res =
                    game.execute_command(&mut player, &message.message)?;
                //log::debug!("B");
                match res {
                    0 => {}
                    4 => {
                        player.send_message(Message::new(&format!("Unknown command.")));
                    }
                    res => {
                        player
                            .send_message(Message::new(&format!("Command returned code {}.", res)));
                    }
                }
            } else {
                //log::debug!("sx");
                let message = message.message;
                //let message = message.replace("&", "§");
                let message = Message::new(&format!("<{}> {}", player.get_username(), message));
                log::info!("[Server task] {}", message.message);
                //log::debug!("bx");
                for (id, player_iter) in game.players.0.borrow().clone() {
                    if id == player.get_id() {
                        player.send_message(message.clone());
                    } else {
                        let mut player = player_iter;
                        player.send_message(message.clone());
                    }
                }
                //log::debug!("nx");
            }
        }
        ClientPacket::Respawn(_) => {
            if !player.is_dead() {
                player.disconnect("Sent respawn packet when alive.".to_string());
                return Ok(());
            }
            player.set_health(20);
            let mut pos = player.get_position();
            pos.x = 3.0;
            pos.y = 20.0;
            pos.z = 5.0;
            let mut pos = pos.clone();
            player.set_last_position(pos);
            player.set_position(pos);
            player.write_packet(ServerPacket::Respawn {
                world: player.get_world(),
            });
            //let id = player.id.0;
            player.set_dead(false);
            game.hide_player(&player.unwrap().unwrap())?;
            //game.broadcast_to_loaded(&player, ServerPacket::DestroyEntity { eid: id })?;
            //game.broadcast_packet(ServerPacket::EntityStatus { eid: player.id.0, entity_status: 0x00 })?;
        }
        ClientPacket::Animation(packet) => {
            if packet.animate == 1 {
                let id = player.get_id().clone();
                let name = player.get_username();
                let list = game.players.0.borrow().clone();
                for list2 in list {
                    if !list2.1.can_borrow() {
                        continue;
                    }
                    let mut player_new = list2.1.unwrap().unwrap();
                    if let Some(_) = player_new.rendered_players.get(&(id, name.clone())) {
                        player_new.write(ServerPacket::Animation {
                            eid: id.0,
                            animate: 1,
                        });
                    } else {
                        //log::info!("Player {} does not have ({}, {}) IN THEIR RENDERD PLAYE.R", player_new.username, player.get_id().0, player.get_username());
                        continue;
                    }
                }
                /*                 for i in 0..len {
                    if i as i32 == player.get_id().0 {
                        continue;
                    }
                    let list = game.players.0.borrow();
                    /*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
                    let list2 =
                        if let Some(plr) = list.get(&crate::network::ids::EntityID(i as i32)) {
                            plr.clone()
                        } else {
                            continue;
                        };
                    let mut player_new = list2.unwrap().unwrap();
                    drop(list);
                    if let Some(_) = player_new
                        .rendered_players
                        .get(&(player.get_id(), player.get_username()))
                    {
                        player_new.write(ServerPacket::Animation {
                            eid: player.get_id().0,
                            animate: 1,
                        });
                    } else {
                        log::info!("Player {} does not have ({}, {}) IN THEIR RENDERD PLAYE.R", player_new.username, player.get_id().0, player.get_username());
                        continue;
                    }
                } */
            }
        }
        ClientPacket::Disconnect(_) => {
            player.unwrap().unwrap().remove(None);
        }
        ClientPacket::HoldingChange(packet) => {
            player.set_held_slot(packet.slot_id);
        }
        ClientPacket::WindowClick(packet) => {
            if let Err(e) = window_click_handler(packet, player.clone(), game) {
                log::debug!(
                    "Error handling packet from user {}: {:?}",
                    player.get_username(),
                    e
                );
            }
        }
        ClientPacket::WindowClick(mut packet) => {
            use std::ops::DerefMut;
            let mut player = player.unwrap().unwrap();
            let mut player = player.deref_mut();
            let registry = ItemRegistry::global();
            log::debug!("Window id: {}", packet.window_id);
            if packet.window_id == 0 {
                if packet.item_id != -1 {
                    /*                     if registry.get_item(packet.item_id).is_none() {
                        //invslot = ItemStack::default();
                        player.sync_inventory();
                        //crate::systems::sync_inv_force(game, server, player)?;
                        player.write(ServerPacket::Transaction {
                            window_id: 0,
                            action_number: packet.action_number,
                            accepted: false,
                        });
                        return Ok(());
                    } */
                    let item = ItemStack::new(
                        packet.item_id,
                        packet.item_uses.unwrap(),
                        packet.item_count.unwrap(),
                    );
                    if player
                        .inventory
                        .items
                        .get(&(packet.slot as i8))
                        .expect("Slot doesn't exist!")
                        != &item
                    {
                        log::debug!("Declined! Tried to get {}", packet.slot);
                        player.sync_inventory();
                        //crate::systems::sync_inv_force(game, server, player)?;
                        player.write(ServerPacket::Transaction {
                            window_id: 0,
                            action_number: packet.action_number,
                            accepted: false,
                        });
                        return Ok(());
                    }
                    let invslot = player
                        .inventory
                        .items
                        .get_mut(&(packet.slot as i8))
                        .expect("Slot doesn't exist!");
                    if invslot.count == 0 || registry.get_item(invslot.id).is_none() {
                        *invslot = ItemStack::default();
                        player.sync_inventory();
                        //crate::systems::sync_inv_force(game, server, player)?;
                        player.write(ServerPacket::SetSlot {
                            window_id: -1,
                            slot: -1,
                            item_id: -1,
                            item_count: None,
                            item_uses: None,
                        });
                        player.write(ServerPacket::Transaction {
                            window_id: 0,
                            action_number: packet.action_number,
                            accepted: false,
                        });
                        return Ok(());
                    }
                    if (5..9).contains(&packet.slot) {
                        player.held_item_changed = true;
                    }
                    if invslot.id != 0 {
                        if (*player).current_cursored_item.is_some()
                            && invslot.id == player.current_cursored_item.as_ref().unwrap().id
                            && packet.right_click == 0
                        {
                            if packet.slot != 0 {
                                //*invslot = ItemStack::default();
                                //log::debug!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
                                if invslot.count as u64
                                    + player.current_cursored_item.clone().unwrap().count.max(1)
                                        as u64
                                    > registry.get_item(item.id).unwrap().get_item().stack_size()
                                        as u64
                                {
                                    let thing = invslot.count
                                        + player
                                            .current_cursored_item
                                            .clone()
                                            .unwrap()
                                            .count
                                            .max(1);
                                    let thing = thing
                                        - registry
                                            .get_item(item.id)
                                            .unwrap()
                                            .get_item()
                                            .stack_size()
                                            as i8;
                                    player.current_cursored_item.as_mut().unwrap().count = thing;
                                    invslot.count =
                                        registry.get_item(item.id).unwrap().get_item().stack_size()
                                            as i8;
                                } else {
                                    invslot.count +=
                                        player.current_cursored_item.clone().unwrap().count.max(1);
                                    player.current_cursored_item = None;
                                }
                            } else {
                                if let Some(mut curcurid) = player.current_cursored_item {
                                    log::debug!("Is some");
                                    if curcurid.id == invslot.id
                                        && curcurid.damage == invslot.damage
                                    {
                                        log::debug!("Got here!");
                                        if (curcurid.count as u64 + invslot.count as u64)
                                            <= registry
                                                .get_item(invslot.id)
                                                .unwrap()
                                                .get_item()
                                                .stack_size()
                                                as u64
                                        {
                                            curcurid.count += invslot.count;
                                            invslot.count = 0;
                                            log::debug!("New: {:?} {:?}", curcurid, invslot);
                                            player.current_cursored_item = Some(curcurid);
                                        }
                                    }
                                }
                                log::debug!("Doing this!");
                            }
                        } else if packet.right_click == 1 {
                            log::debug!("Right click!");
                            if let Some(mut curcurid) = player.current_cursored_item {
                                if curcurid.count > 0 {
                                    if invslot.damage == curcurid.damage
                                        && invslot.id == curcurid.id
                                    {
                                        if invslot.count as u64 + 1
                                            < registry
                                                .get_item(item.id)
                                                .unwrap()
                                                .get_item()
                                                .stack_size()
                                                as u64
                                        {
                                            invslot.count += 1;
                                            curcurid.count -= 1;
                                            player.current_cursored_item = Some(curcurid);
                                        }
                                    }
                                }
                            }
                        } else {
                            if player.current_cursored_item.is_some() {
                                *invslot = player.current_cursored_item.as_ref().unwrap().clone();
                            } else {
                                *invslot = ItemStack::default();
                            }
                            player.current_cursored_item = Some(item.clone());
                            /*                             player.write(ServerPacket::Transaction { window_id: 0, action_number: packet.action_number, accepted: false });
                            return Ok(()); */
                        }
                    } else {
                        *invslot = ItemStack::default();
                    }
                    if packet.slot == 0 {
                        log::debug!("Slot zro");
                        for i in 0..5 {
                            let slot = player.inventory.items.get_mut(&i).unwrap();
                            if i != 0 {
                                if slot.count - 1 <= 0 || slot.id == 0 {
                                    slot.id = 0;
                                    slot.count = 0;
                                    slot.damage = 0;
                                    continue;
                                }
                                slot.count -= 1;
                            }
                            drop(slot);
                            player.sync_inventory();
                            //*player.inventory.items.get_mut(&i).unwrap() = ItemStack::default();
                        }
                        let mut grid = Grid::default();
                        grid[0][0] = if let Some(a) = player.inventory.items.get(&1) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        grid[1][0] = if let Some(a) = player.inventory.items.get(&2) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        grid[0][1] = if let Some(a) = player.inventory.items.get(&3) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        grid[1][1] = if let Some(a) = player.inventory.items.get(&4) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        log::debug!("Grid:\n{:?}", grid);
                        if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                            log::debug!("Got it!");
                            *player.inventory.items.get_mut(&0).unwrap() = out;
                        };
                    }
                } else {
                    if player.current_cursored_item.is_none()
                        || (player.inventory.items.get(&(packet.slot as i8)).unwrap().id
                            != player.current_cursored_item.clone().unwrap().id
                            && player.inventory.items.get(&(packet.slot as i8)).unwrap().id != 0)
                    {
                        player.sync_inventory();
                        //crate::systems::sync_inv_force(game, server, player)?;
                        player.write(ServerPacket::Transaction {
                            window_id: 0,
                            action_number: packet.action_number,
                            accepted: false,
                        });
                        return Ok(());
                    }
                    let mut curcurid = player.current_cursored_item.clone().unwrap();
                    use crate::game::items::*;
                    if (5..9).contains(&packet.slot) {
                        if let Some(item) = registry.get_item(curcurid.id) {
                            let tool_type = if let Some(tool_type) = item.get_item().get_tool_type()
                            {
                                tool_type
                            } else {
                                return Ok(());
                            };
                            match packet.slot {
                                5 => match tool_type {
                                    ToolType::HELMET => {}
                                    _ => {
                                        player.write(ServerPacket::Transaction {
                                            window_id: 0,
                                            action_number: packet.action_number,
                                            accepted: false,
                                        });
                                        return Ok(());
                                    }
                                },
                                6 => match tool_type {
                                    ToolType::CHESTPLATE => {}
                                    _ => {
                                        player.write(ServerPacket::Transaction {
                                            window_id: 0,
                                            action_number: packet.action_number,
                                            accepted: false,
                                        });
                                        return Ok(());
                                    }
                                },
                                7 => match tool_type {
                                    ToolType::LEGGINGS => {}
                                    _ => {
                                        player.write(ServerPacket::Transaction {
                                            window_id: 0,
                                            action_number: packet.action_number,
                                            accepted: false,
                                        });
                                        return Ok(());
                                    }
                                },
                                8 => match tool_type {
                                    ToolType::BOOTS => {}
                                    _ => {
                                        player.write(ServerPacket::Transaction {
                                            window_id: 0,
                                            action_number: packet.action_number,
                                            accepted: false,
                                        });
                                        return Ok(());
                                    }
                                },
                                _ => {
                                    player.write(ServerPacket::Transaction {
                                        window_id: 0,
                                        action_number: packet.action_number,
                                        accepted: false,
                                    });
                                    return Ok(());
                                }
                            }
                            /*                             if !item.get_item().get_tool_type() ==  {
                                //crate::systems::sync_inv_force(game, server, player)?;
                                player.write(ServerPacket::Transaction {
                                    window_id: 0,
                                    action_number: packet.action_number,
                                    accepted: false,
                                });
                                return Ok(());
                            } */
                            player.held_item_changed = true;
                        }
                    }
                    let mut invslot = player
                        .inventory
                        .items
                        .get_mut(&(packet.slot as i8))
                        .expect("Slot doesn't exist!");
                    if invslot.id == curcurid.id {
                        invslot.count += curcurid.count;
                    }
                    if packet.right_click == 1 {
                        if curcurid.count <= 0 {
                            player.sync_inventory();
                            //crate::systems::sync_inv_force(game, server, player)?;
                            player.write(ServerPacket::Transaction {
                                window_id: 0,
                                action_number: packet.action_number,
                                accepted: false,
                            });
                            return Ok(());
                        }
                        *invslot = curcurid.clone();
                        invslot.count = 1;
                        curcurid.count -= 1;
                        if curcurid.count >= 1 {
                            player.current_cursored_item = Some(curcurid);
                        } else {
                            player.current_cursored_item = None;
                        }
                    } else {
                        if invslot.id != 0 {
                            //player.current_cursored_item = Some(invslot.clone());
                        } else {
                            player.current_cursored_item = None;
                        }
                        player.current_cursored_item = None;
                        *invslot = curcurid.clone();
                    }
                    if (1..5).contains(&packet.slot) {
                        log::debug!("Contains.");
                        let mut grid = Grid::default();
                        grid[0][0] = if let Some(a) = player.inventory.items.get(&1) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        grid[1][0] = if let Some(a) = player.inventory.items.get(&2) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        grid[0][1] = if let Some(a) = player.inventory.items.get(&3) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        grid[1][1] = if let Some(a) = player.inventory.items.get(&4) {
                            let a = a.clone();
                            if a.id == 0 {
                                None
                            } else {
                                Some(a)
                            }
                        } else {
                            None
                        };
                        log::debug!("Grid:\n{:?}", grid);
                        if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                            log::debug!("Got it!");
                            *player.inventory.items.get_mut(&0).unwrap() = out;
                        };
                    }
                    log::debug!("Slot {}", packet.slot);
                }
                //player.last_transaction_id = packet.action_number;
                log::debug!(
                    "Client sent {:?}, {:?}, {:?} {}",
                    packet.item_id,
                    packet.item_count,
                    packet.item_uses,
                    packet.slot
                );
                player.write(ServerPacket::Transaction {
                    window_id: 0,
                    action_number: packet.action_number,
                    accepted: true,
                });
                return Ok(());
            } else {
                if let Some(window) = player.open_inventories.get_mut(&packet.window_id) {
                    let wandow = window.inventory.clone();
                    let mut inventory = &mut *wandow.borrow_mut();
                    let mut inv_type: Option<i8> = None;
                    if inventory.items.len() < packet.slot as usize {
                        log::debug!("switching to plr inv");
                        packet.slot -= 1;
                        //packet.slot -= inventory.items.len() as i16;
                        inventory = &mut player.inventory;
                    } else {
                        inv_type = Some(window.inventory_type);
                        log::debug!("Is some! {} {}", inventory.items.len(), packet.slot);
                    }
                    if packet.item_id != -1 {
                        /*                     if registry.get_item(packet.item_id).is_none() {
                            //invslot = ItemStack::default();
                            player.sync_inventory();
                            //crate::systems::sync_inv_force(game, server, player)?;
                            player.write(ServerPacket::Transaction {
                                window_id: 0,
                                action_number: packet.action_number,
                                accepted: false,
                            });
                            return Ok(());
                        } */
                        let item = ItemStack::new(
                            packet.item_id,
                            packet.item_uses.unwrap(),
                            packet.item_count.unwrap(),
                        );
                        log::debug!("Got 2 here");
                        if inventory
                            .items
                            .get(&(packet.slot as i8))
                            .expect("Slot doesn't exist!")
                            != &item
                        {
                            log::debug!("Declined! Tried to get {}", packet.slot);
                            player.sync_inventory();
                            //crate::systems::sync_inv_force(game, server, player)?;
                            player.write(ServerPacket::Transaction {
                                window_id: 0,
                                action_number: packet.action_number,
                                accepted: false,
                            });
                            return Ok(());
                        }
                        let invslot = inventory
                            .items
                            .get_mut(&(packet.slot as i8))
                            .expect("Slot doesn't exist!");
                        if invslot.count == 0 || registry.get_item(invslot.id).is_none() {
                            log::debug!("Is bad");
                            *invslot = ItemStack::default();
                            player.sync_inventory();
                            //crate::systems::sync_inv_force(game, server, player)?;
                            player.write(ServerPacket::SetSlot {
                                window_id: -1,
                                slot: -1,
                                item_id: -1,
                                item_count: None,
                                item_uses: None,
                            });
                            player.write(ServerPacket::Transaction {
                                window_id: 0,
                                action_number: packet.action_number,
                                accepted: false,
                            });
                            return Ok(());
                        }
                        if invslot.id != 0 {
                            if (*player).current_cursored_item.is_some()
                                && invslot.id == player.current_cursored_item.as_ref().unwrap().id
                                && packet.right_click == 0
                            {
                                if packet.slot != 0 {
                                    //*invslot = ItemStack::default();
                                    //log::debug!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
                                    if invslot.count
                                        + player.current_cursored_item.clone().unwrap().count.max(1)
                                        > registry
                                            .get_item(item.id)
                                            .unwrap()
                                            .get_item()
                                            .stack_size()
                                            as i8
                                    {
                                        let thing = invslot.count
                                            + player
                                                .current_cursored_item
                                                .clone()
                                                .unwrap()
                                                .count
                                                .max(1);
                                        let thing = thing
                                            - registry
                                                .get_item(item.id)
                                                .unwrap()
                                                .get_item()
                                                .stack_size()
                                                as i8;
                                        player.current_cursored_item.as_mut().unwrap().count =
                                            thing;
                                        invslot.count = registry
                                            .get_item(item.id)
                                            .unwrap()
                                            .get_item()
                                            .stack_size()
                                            as i8;
                                    } else {
                                        invslot.count += player
                                            .current_cursored_item
                                            .clone()
                                            .unwrap()
                                            .count
                                            .max(1);
                                        player.current_cursored_item = None;
                                    }
                                } else {
                                    if let Some(mut curcurid) = player.current_cursored_item {
                                        log::debug!("Is some");
                                        if curcurid.id == invslot.id
                                            && curcurid.damage == invslot.damage
                                        {
                                            log::debug!("Got here!");
                                            if (curcurid.count as u64 + invslot.count as u64)
                                                <= registry
                                                    .get_item(invslot.id)
                                                    .unwrap()
                                                    .get_item()
                                                    .stack_size()
                                                    as u64
                                            {
                                                curcurid.count += invslot.count;
                                                invslot.count = 0;
                                                log::debug!("New: {:?} {:?}", curcurid, invslot);
                                                player.current_cursored_item = Some(curcurid);
                                            }
                                        }
                                    }
                                    log::debug!("Doing this!");
                                }
                            } else if packet.right_click == 1 {
                                log::debug!("Right click!");
                                if let Some(mut curcurid) = player.current_cursored_item {
                                    if curcurid.count > 0 {
                                        if invslot.damage == curcurid.damage
                                            && invslot.id == curcurid.id
                                        {
                                            if invslot.count as u64 + 1
                                                < registry
                                                    .get_item(item.id)
                                                    .unwrap()
                                                    .get_item()
                                                    .stack_size()
                                                    as u64
                                            {
                                                invslot.count += 1;
                                                curcurid.count -= 1;
                                                player.current_cursored_item = Some(curcurid);
                                            }
                                        }
                                    }
                                }
                            } else {
                                if player.current_cursored_item.is_some() {
                                    *invslot =
                                        player.current_cursored_item.as_ref().unwrap().clone();
                                } else {
                                    *invslot = ItemStack::default();
                                }
                                player.current_cursored_item = Some(item.clone());
                                /*                             player.write(ServerPacket::Transaction { window_id: 0, action_number: packet.action_number, accepted: false });
                                return Ok(()); */
                            }
                        } else {
                            *invslot = ItemStack::default();
                        }
                        if inv_type == Some(1) {
                            log::debug!("Balls 2");
                            if packet.slot == 0 {
                                let mut grid = Grid::default();
                                grid[0][0] = if let Some(a) = inventory.items.get(&1) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[1][0] = if let Some(a) = inventory.items.get(&2) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[2][0] = if let Some(a) = inventory.items.get(&3) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[0][1] = if let Some(a) = inventory.items.get(&4) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[1][1] = if let Some(a) = inventory.items.get(&5) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[2][1] = if let Some(a) = inventory.items.get(&6) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[0][2] = if let Some(a) = inventory.items.get(&7) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[1][2] = if let Some(a) = inventory.items.get(&8) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                grid[2][2] = if let Some(a) = inventory.items.get(&9) {
                                    let a = a.clone();
                                    if a.id == 0 {
                                        None
                                    } else {
                                        Some(a)
                                    }
                                } else {
                                    None
                                };
                                //log::debug!("Grid: {:?}", grid);
                                if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                                    //log::debug!("Got it!");
                                    *inventory.items.get_mut(&0).unwrap() = out;
                                };
                                //log::debug!("Slot zro");
                                for i in 0..10 {
                                    //log::debug!("This is running");
                                    let slot = player.inventory.items.get_mut(&i).unwrap();
                                    if i > 0 {
                                        if slot.count - 1 <= 0 {
                                            //log::debug!("Clearing slot!");
                                            slot.id = 0;
                                            slot.count = 0;
                                            slot.damage = 0;
                                            continue;
                                        }
                                        slot.count -= 1;
                                    } else {
                                        slot.id = 0;
                                        slot.count = 0;
                                        slot.damage = 0;
                                        continue;
                                    }
                                }
                            }
                        }
                    } else {
                        log::debug!("Slot: {:?}", packet.slot);
                        if player.current_cursored_item.is_none()
                            || (inventory.items.get(&(packet.slot as i8)).unwrap().id
                                != player.current_cursored_item.clone().unwrap().id
                                && inventory.items.get(&(packet.slot as i8)).unwrap().id != 0)
                        {
                            log::debug!(
                                "Stopping here {} {}",
                                player.current_cursored_item.is_none(),
                                inventory.items.get(&(packet.slot as i8)).unwrap().id != 0
                            );
                            if player.current_cursored_item.is_some() {
                                log::debug!(
                                    "and this {}",
                                    inventory.items.get(&(packet.slot as i8)).unwrap().id
                                        != player.current_cursored_item.clone().unwrap().id
                                );
                            }
                            player.sync_inventory();
                            //crate::systems::sync_inv_force(game, server, player)?;
                            player.write(ServerPacket::Transaction {
                                window_id: packet.window_id,
                                action_number: packet.action_number,
                                accepted: false,
                            });
                            return Ok(());
                        }
                        let mut curcurid = player.current_cursored_item.clone().unwrap();
                        /*                         if (5..9).contains(&packet.slot) {
                            if let Some(item) = registry.get_item(curcurid.id) {
                                if !item.get_item().wearable() {
                                    //crate::systems::sync_inv_force(game, server, player)?;
                                    player.write(ServerPacket::Transaction {
                                        window_id: 0,
                                        action_number: packet.action_number,
                                        accepted: false,
                                    });
                                    return Ok(());
                                }
                                player.held_item_changed = true;
                            }
                        } */
                        let mut invslot = inventory
                            .items
                            .get_mut(&(packet.slot as i8))
                            .expect("Slot doesn't exist!");
                        if invslot.id == curcurid.id {
                            invslot.count += curcurid.count;
                        }
                        if packet.right_click == 1 {
                            if curcurid.count <= 0 {
                                player.sync_inventory();
                                //crate::systems::sync_inv_force(game, server, player)?;
                                player.write(ServerPacket::Transaction {
                                    window_id: 0,
                                    action_number: packet.action_number,
                                    accepted: false,
                                });
                                return Ok(());
                            }
                            *invslot = curcurid.clone();
                            invslot.count = 1;
                            curcurid.count -= 1;
                            if curcurid.count >= 1 {
                                player.current_cursored_item = Some(curcurid);
                            } else {
                                player.current_cursored_item = None;
                            }
                        } else {
                            if invslot.id != 0 {
                                //player.current_cursored_item = Some(invslot.clone());
                            } else {
                                player.current_cursored_item = None;
                            }
                            player.current_cursored_item = None;
                            log::debug!("Setting slot");
                            *invslot = curcurid.clone();
                        }
                        if (1..10).contains(&packet.slot) {
                            let mut grid = Grid::default();
                            grid[0][0] = if let Some(a) = inventory.items.get(&1) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[1][0] = if let Some(a) = inventory.items.get(&2) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[2][0] = if let Some(a) = inventory.items.get(&3) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[0][1] = if let Some(a) = inventory.items.get(&4) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[1][1] = if let Some(a) = inventory.items.get(&5) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[2][1] = if let Some(a) = inventory.items.get(&6) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[0][2] = if let Some(a) = inventory.items.get(&7) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[1][2] = if let Some(a) = inventory.items.get(&8) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            grid[2][2] = if let Some(a) = inventory.items.get(&9) {
                                let a = a.clone();
                                if a.id == 0 {
                                    None
                                } else {
                                    Some(a)
                                }
                            } else {
                                None
                            };
                            log::debug!("Grid: {:?}", grid);
                            if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                                log::debug!("Got it!");
                                *inventory.items.get_mut(&0).unwrap() = out;
                            };
                        }
                        log::debug!("Slot {}", packet.slot);
                    }
                    //player.last_transaction_id = packet.action_number;
                    log::debug!(
                        "Client sent {:?}, {:?}, {:?}",
                        packet.item_id,
                        packet.item_count,
                        packet.item_uses
                    );
                    player.write(ServerPacket::Transaction {
                        window_id: 0,
                        action_number: packet.action_number,
                        accepted: true,
                    });
                    log::debug!("Balls");
                    return Ok(());
                }
            }
        }
        // ClientPacket::WindowClick(packet) => {
        //     let mut player = player.unwrap().unwrap();
        //     if packet.window_id == 0 {
        //         if packet.item_id != -1 {
        //             let item = ItemStack::new(
        //                 packet.item_id,
        //                 packet.item_uses.unwrap(),
        //                 packet.item_count.unwrap(),
        //             );
        //             if *player.inventory.items.get(&(packet.slot as i8))
        //                 .expect("Slot doesn't exist!")
        //                 != item
        //             {
        //                 log::debug!("Declined! Tried to get {}", packet.slot);
        //                 player.sync_inventory();
        //                 //crate::systems::sync_inv_force(game, server, player)?;
        //                 player.write(ServerPacket::Transaction {
        //                     window_id: 0,
        //                     action_number: packet.action_number,
        //                     accepted: false,
        //                 });
        //                 return Ok(());
        //             }
        //             //let mut inv = &mut player.inventory.items;
        //             let curcuritem = player.current_cursored_item.clone();
        //             let mut invslot = player.inventory.items.get_mut(&(packet.slot as i8))
        //                 .expect("Slot doesn't exist!");
        //             if invslot.count == 0 {
        //                 //player.set_inventory_slot(packet.slot as i8, ItemStack::default());
        //                 //player.sync_inventory();
        //                 //crate::systems::sync_inv_force(game, server, player)?;
        //                 player.write(ServerPacket::Transaction {
        //                     window_id: 0,
        //                     action_number: packet.action_number,
        //                     accepted: false,
        //                 });
        //                 return Ok(());
        //             }
        //             if invslot.id != 0 {
        //                 if curcuritem.is_some()
        //                     && invslot.id == curcuritem.as_ref().unwrap().id
        //                     && packet.right_click == 0
        //                 {
        //                     //*invslot = ItemStack::default();
        //                     //log::debug!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
        //                     invslot.count +=
        //                         curcuritem.as_ref().unwrap().count.max(1);
        //                     player.current_cursored_item = None;
        //                 } else {
        //                     *player.inventory.items.get_mut(&(packet.slot as i8)).unwrap() = ItemStack::default();
        //                     player.current_cursored_item = Some(item.clone());
        //                     /*                             player.write(ServerPacket::Transaction { window_id: 0, action_number: packet.action_number, accepted: false });
        //                     return Ok(()); */
        //                 }
        //             } else {
        //                 *player.inventory.items.get_mut(&(packet.slot as i8)).unwrap() = ItemStack::default();
        //             }
        //         } else {
        //             //let mut inv = player.inventory.items;
        //             let curcuritem = player.current_cursored_item.clone();
        //             let slot = player.inventory.items.get_mut(&(packet.slot as i8)).unwrap();
        //             if curcuritem.is_none()
        //                 || (slot.id
        //                     != curcuritem.clone().unwrap().id
        //                     && slot.id != 0)
        //             {
        //                 player.sync_inventory();
        //                 //crate::systems::sync_inv_force(game, server, player)?;
        //                 player.write(ServerPacket::Transaction {
        //                     window_id: 0,
        //                     action_number: packet.action_number,
        //                     accepted: false,
        //                 });
        //                 return Ok(());
        //             }
        //             let mut curcurid = curcuritem.clone().unwrap();
        //             let mut invslot = slot;
        //             if invslot.id == curcurid.id {
        //                 invslot.count += curcurid.count;
        //             }
        //             if packet.right_click == 1 {
        //                 if curcurid.count <= 0 {
        //                     //player.sync_inventory();
        //                     //crate::systems::sync_inv_force(game, server, player)?;
        //                     player.write(ServerPacket::Transaction {
        //                         window_id: 0,
        //                         action_number: packet.action_number,
        //                         accepted: false,
        //                     });
        //                     return Ok(());
        //                 }
        //                 if !(invslot.id == curcurid.id || invslot.id == 0x00) {
        //                     player.write(ServerPacket::Transaction {
        //                         window_id: 0,
        //                         action_number: packet.action_number,
        //                         accepted: false,
        //                     });
        //                     return Ok(());
        //                 }
        //                 //player.set_inventory_slot(packet.slot as i8, curcurid.clone());
        //                 //*invslot = curcurid.clone();
        //                 invslot.count = 1;
        //                 curcurid.count -= 1;
        //                 invslot.id = curcurid.id;
        //                 if curcurid.count >= 1 {
        //                     player.current_cursored_item = Some(curcurid);
        //                 } else {
        //                     player.current_cursored_item = None;
        //                 }
        //             } else {
        //                 *player.inventory.items.get_mut(&(packet.slot as i8)).unwrap() = curcurid.clone();
        //                 player.current_cursored_item= None;
        //             }
        //         }
        //         //player.last_transaction_id = packet.action_number;
        //         log::debug!(
        //             "Client sent {:?}, {:?}, {:?}",
        //             packet.item_id,
        //             packet.item_count,
        //             packet.item_uses
        //         );
        //         player.write(ServerPacket::Transaction {
        //             window_id: 0,
        //             action_number: packet.action_number,
        //             accepted: true,
        //         });
        //         return Ok(());
        //     }
        // }
        // TODO don't use unwrap on the player
        ClientPacket::UseEntity(packet) => {
            let interval = std::time::Duration::from_millis(350);
            let mut player = player.unwrap().unwrap();
            if packet.left_click {
                if player.since_last_attack + interval > std::time::Instant::now() {
                    return Ok(());
                }
                player.since_last_attack = std::time::Instant::now();
                //game.broadcast_packet(ServerPacket::SoundEffect { effect_id: 1001, x: player.position.x as i32, y: player.position.y as i8, z: player.position.z as i32, sound_data: 0 })?;
                let plrs = game.players.0.borrow();
                let plr = plrs.get(&EntityID(packet.target)).clone();
                if let Some(plr) = plr {
                    let gamerule = game.gamerules.rules.get("pvp-enabled").unwrap();
                    if let crate::game::gamerule::GameruleValue::Boolean(value) = gamerule {
                        if !value {
                            return Ok(());
                        }
                    } else {
                        panic!("PVP Gamerule is not a boolean!");
                    }
                    if plr.get_position().distance(&player.position) < 6.0 {
                        let registry = ItemRegistry::global();
                        let mut dmg = 1;
                        if let Some(hand) = player.get_item_in_hand_mut() {
                            if let Some(item) = registry.get_item(hand.id as i16) {
                                if let Some(max_dmg) = item.get_item().max_uses() {
                                    hand.damage += 2;
                                    if hand.damage as u64 > max_dmg {
                                        hand.reset();
                                    }     
                                    player.sync_inventory();
                                }
                                if let Some(damage) = item.get_item().damage() {
                                    dmg = damage;
                                }
                            }
                        }
                        let mut plr = plr.unwrap().unwrap();
                        if plr.dead {
                            return Ok(());
                        }
                        plr.damage(
                            DamageType::Player {
                                damager: player.username.clone(),
                            },
                            dmg,
                            Some(&mut player),
                        );
                        use std::ops::Mul;
                        let arr = player.position.get_direction().mul(1980.0).to_array();
                        let x = arr[0];
                        let y = arr[1];
                        let z = arr[2];
                        //log::debug!("Adding velocity {} {} {}", x, y, z);
                        plr.add_velocity(
                            x as i16,
                            (((y as i64) + 2043).min(i16::MAX as i64)) as i16,
                            z as i16,
                        );
                        //plr.add_velocity(0, 1343, 0);
                        /*                         plr.health -= 1;
                        let id = plr.id.0;
                        player.write(ServerPacket::Animation { eid: id, animate: 2});
                        player.write(ServerPacket::EntityStatus { eid: id, entity_status: 2 });
                        for (_, player) in &*plrs {
                            let mut player = if let Ok(plr) = player.try_borrow_mut() {
                                plr
                            } else {
                                continue;
                            };
                            player.write(ServerPacket::EntityStatus { eid: plr.id.0, entity_status: 2 });
                            player.write(ServerPacket::Animation { eid: plr.id.0, animate: 2});
                        } */
                    }
                } else {
                    drop(plrs);
                    let entities = game.entities.borrow().clone();
                    if let Some(entity) = entities.get(&EntityID(packet.target)) {
                        let mut entity = entity.borrow_mut();
                        let mut dmg = 1;
                        if entity.get_position().distance(&player.position) < 6.0 {
                            if entity.is_dead() {
                                return Ok(());
                            }
                            let registry = ItemRegistry::global();
                            if let Some(hand) = player.get_item_in_hand_mut() {
                                if let Some(item) = registry.get_item(hand.id as i16) {
                                    if let Some(max_dmg) = item.get_item().max_uses() {
                                        hand.damage += 2;
                                        if hand.damage as u64 > max_dmg {
                                            hand.reset();
                                        }     
                                        player.sync_inventory();
                                    }
                                    if let Some(damage) = item.get_item().damage() {
                                        dmg = damage;
                                    }
                                }
                            }
                            let arr = player.position.get_direction().mul(3.0).to_array();
                            entity.add_velocity(arr);
                            drop(player);
                            entity.damage(game, dmg);
                            use std::ops::Mul;
                        }
                    }
                }
            }
        }
        ClientPacket::PlayerBlockPlacement(mut packet) => {
            if packet.block_or_item_id >= 0 {
                let item = ItemStack::new(
                    packet.block_or_item_id,
                    packet.damage.unwrap(),
                    packet.amount.unwrap() + 1,
                );
                let item_2 = ItemStack::new(
                    packet.block_or_item_id,
                    packet.damage.unwrap(),
                    packet.amount.unwrap(),
                );
                let mut held = player.get_item_in_hand();
                let mut sync: bool = false;
                if item != *held && item_2 != *held {
                    log::debug!("Not, comparing {:?} to {:?}", item, *held);
                    sync = true;
                }
                drop(held);
                if sync {
                    player.sync_inventory();
                    return Ok(());
                }
                // packet.y -= 1;
                /*                 match packet.direction {
                    0 => {
                        packet.y -= 1;
                    }
                    1 => {
                        packet.y += 1;
                    }
                    2 => {
                        packet.z -= 1;
                    }
                    3 => {
                        packet.z += 1;
                    }
                    4 => {
                        packet.x -= 1;
                    }
                    5 => {
                        packet.x += 1;
                    }
                    _ => {
                        return Ok(());
                    }
                } */
                let mut exopacket = packet.clone();
                match exopacket.direction {
                    0 => {
                        exopacket.y -= 1;
                    }
                    1 => {
                        exopacket.y = match exopacket.y.checked_add(1) {
                            Some(num) => num,
                            None => {
                                return Ok(());
                            }
                        }
                    }
                    2 => {
                        exopacket.z -= 1;
                    }
                    3 => {
                        exopacket.z += 1;
                    }
                    4 => {
                        exopacket.x -= 1;
                    }
                    5 => {
                        exopacket.x += 1;
                    }
                    x => {
                        log::debug!("Fal {}", x);
                        //return false;
                    }
                }
                let registry = ItemRegistry::global();
                if let Some(block) = game.world.get_block(packet.x, packet.y as i32, packet.z) {
                    log::debug!("Block: {:?}", block);
                    if let Some(i) = registry.get_item(block.b_type as i16) {
                        if let Some(i) = i.get_item().as_block() {
                            if i.on_right_click(game, &mut exopacket, player.clone()) == false {
                                return Ok(());
                            }
                        }
                    }
                }
                if let Some(i) = registry.get_item(item.id) {
                    i.get_item().on_use(game, packet, player)?;
                } else {
                    let mut held = player.get_item_in_hand();
                    held.id = 0;
                    held.count = 0;
                    drop(held);
                    player.sync_inventory();
                    return Ok(());
                }
            } else {
                let registry = ItemRegistry::global();
                if let Some(block) = game.world.get_block(packet.x, packet.y as i32, packet.z) {
                    log::debug!("Block: {:?}", block);
                    if let Some(i) = registry.get_item(block.b_type as i16) {
                        if let Some(i) = i.get_item().as_block() {
                            if i.on_right_click(game, &mut packet, player.clone()) == false {
                                return Ok(());
                            }
                        }
                    }
                }
            }
        }
        ClientPacket::PlayerBlockPlacement(mut packet) => {
            let mut player = player.unwrap().unwrap();
            let mut success = false;
            if packet.block_or_item_id >= 0 {
                if packet.x == -1 && packet.y == -1 && packet.z == -1 {
                    return Ok(());
                }
                let item = ItemStack::new(
                    packet.block_or_item_id,
                    packet.damage.unwrap(),
                    packet.amount.unwrap() - 1,
                );
                let item_2 = ItemStack::new(
                    packet.block_or_item_id,
                    packet.damage.unwrap(),
                    packet.amount.unwrap(),
                );
                let held = player.get_item_in_hand_mut().unwrap();
                let despawn;
                if item != *held && item_2 != *held {
                    log::debug!("Not, comparing {:?} to {:?}", item, *held);
                    despawn = true;
                    return Ok(());
                } else {
                    despawn = false;
                }
                log::debug!("Block: {:?} {:?} {:?}", packet.x, packet.y, packet.z);
                held.count -= 1;
                if despawn {
                    //player.sync_inventory();
                    crate::systems::sync_inv_force(game, server, &mut player)?;
                }
                packet.y -= 1;
                match packet.direction {
                    0 => {
                        packet.y -= 1;
                    }
                    1 => {
                        packet.y += 1;
                    }
                    2 => {
                        packet.z -= 1;
                    }
                    3 => {
                        packet.z += 1;
                    }
                    4 => {
                        packet.x -= 1;
                    }
                    5 => {
                        packet.x += 1;
                    }
                    _ => {
                        return Ok(());
                    }
                }
                let block = if let Some(blk) =
                    game.world
                        .get_block(packet.x, (packet.y + 0) as i32, packet.z)
                {
                    blk
                } else {
                    return Ok(());
                };
                let mut pos = player.position.clone();
                let held = player.get_item_in_hand_mut().unwrap();
                for user in game.players.0.borrow().iter() {
                    /*                     let mut pos = user.1.try_borrow();
                    if pos.is_err() {
                        continue;
                    } */
                    //let mut pos = pos;
                    if pos.contains_block(crate::game::BlockPosition {
                        x: packet.x,
                        y: (packet.y + 1) as i32,
                        z: packet.z,
                    }) {
                        held.count += 1;
                        player.write(ServerPacket::BlockChange {
                            x: packet.x,
                            y: packet.y + 1,
                            z: packet.z,
                            block_type: block.get_type() as i8,
                            block_metadata: block.b_metadata as i8,
                        });
                        return Ok(());
                    }
                }
                if pos.contains_block(crate::game::BlockPosition {
                    x: packet.x,
                    y: (packet.y + 1) as i32,
                    z: packet.z,
                }) {
                    held.count += 1;
                    player.write(ServerPacket::BlockChange {
                        x: packet.x,
                        y: packet.y + 1,
                        z: packet.z,
                        block_type: block.get_type() as i8,
                        block_metadata: block.b_metadata as i8,
                    });
                    return Ok(());
                }
                //let mut pos = crate::world::World::pos_to_index(packet.x, packet.y as i32, packet.z as i32);
                log::debug!(
                    "Setting at X: {} Y: {} Z: {}",
                    packet.x,
                    packet.y as i32,
                    packet.z
                );
                if block.get_type() == 0 {
                    //player.write(ServerPacket::BlockChange { x: packet.x, y: packet.y, z: packet.z, block_type: item.id as i8, block_metadata: 0x00 });
                    log::debug!("Setting block.");
                    block.set_type(item.id as u8);
                    game.block_updates.push(crate::game::Block {
                        position: crate::game::BlockPosition {
                            x: packet.x,
                            y: (packet.y + 1) as i32,
                            z: packet.z,
                        },
                        block: block.clone(),
                    });
                    success = true;
                } else {
                    player.write(ServerPacket::BlockChange {
                        x: packet.x,
                        y: packet.y + 1,
                        z: packet.z,
                        block_type: block.get_type() as i8,
                        block_metadata: block.b_metadata as i8,
                    })
                }
                if !success {}
            }
        }
        // TODO more usage of unwrap on the player im lazy
        ClientPacket::PlayerDigging(packet) => {
            let mut player = player.unwrap().unwrap();
            match packet.status {
                0 => {
                    player.mining_block.block = BlockPosition {
                        x: packet.x,
                        y: packet.y as i32,
                        z: packet.z as i32,
                    };
                    player.mining_block.face = packet.face;
                    //log::debug!("Got");
                    let block = if let Some(blk) =
                        game.world
                            .get_block(packet.x, (packet.y - 0) as i32, packet.z)
                    {
                        blk
                    } else {
                        return Ok(());
                    };
                    let orig_type = block.b_type.clone();
                    //log::debug!("Or ig type {}", orig_type);
                    let registry = ItemRegistry::global();
                    if let Some(item) = registry.get_item(orig_type as i16) {
                        if let None = item.get_item().as_block() {
                            return Ok(());
                        }
                    } else {
                        return Ok(());
                    }
                    // TODO Instabreak instead of this hard-coded check
                    if orig_type != 50 {
                        //log::debug!("Denied");
                        player.write(ServerPacket::BlockChange {
                            x: packet.x,
                            y: packet.y + 0,
                            z: packet.z,
                            block_type: block.get_type() as i8,
                            block_metadata: block.b_metadata as i8,
                        });
                        return Ok(());
                    }
                    block.set_type(0);
                    game.block_updates.push(crate::game::Block {
                        position: crate::game::BlockPosition {
                            x: packet.x,
                            y: (packet.y + 0) as i32,
                            z: packet.z,
                        },
                        block: block.clone(),
                    });
                    log::debug!("orig_type: {}", orig_type);
                    game.broadcast_to_loaded(
                        &player,
                        ServerPacket::SoundEffect {
                            effect_id: 2001,
                            x: packet.x,
                            y: packet.y,
                            z: packet.z,
                            sound_data: orig_type as i32,
                        },
                    )?;
                    player.sync_inventory();
                    let registry = ItemRegistry::global();
                    if let Some(item) = registry.get_item(orig_type as i16) {
                        if let Some(block) = item.get_item().as_block() {
                            let tool = player.get_item_in_hand().clone();
                            if let Some(drop) = block.on_break(game, packet.clone(), player, tool) {
                                game.spawn_entity(Box::new(
                                    crate::game::entities::item_entity::ItemEntity::new(
                                        Position::from_pos(
                                            packet.x as f64,
                                            (packet.y as f64) + 1.0,
                                            packet.z as f64,
                                        ),
                                        game.ticks,
                                        drop,
                                        None,
                                    ),
                                ));
                            }
                        }
                    }
                }
                2 => {
                    log::debug!("Got pos {} {} {}", packet.x, packet.y, packet.z);
                    let block = if let Some(blk) =
                        game.world
                            .get_block(packet.x, (packet.y - 0) as i32, packet.z)
                    {
                        blk
                    } else {
                        return Ok(());
                    };
                    let orig_type = block.b_type.clone();
                    //log::debug!("Or ig type {}", orig_type);
                    let registry = ItemRegistry::global();
                    if let Some(item) = registry.get_item(orig_type as i16) {
                        if let None = item.get_item().as_block() {
                            return Ok(());
                        }
                    } else {
                        return Ok(());
                    }
                    // TODO Instabreak instead of this hard-coded check
                    if player.mining_block.block
                        != (BlockPosition {
                            x: packet.x,
                            y: packet.y as i32,
                            z: packet.z,
                        })
                        && orig_type != 50
                    {
                        //log::debug!("Denied");
                        player.write(ServerPacket::BlockChange {
                            x: packet.x,
                            y: packet.y + 0,
                            z: packet.z,
                            block_type: block.get_type() as i8,
                            block_metadata: block.b_metadata as i8,
                        });
                        return Ok(());
                    }
                    /*                     if player.mining_block.face != packet.face {
                        player.write(ServerPacket::BlockChange {
                            x: packet.x,
                            y: packet.y + 1,
                            z: packet.z,
                            block_type: block.get_type() as i8,
                            block_metadata: block.b_metadata as i8,
                        });
                        return Ok(());
                    } */
                    /*                     let btype = block.b_type;
                    drop(block);
                    let registry = ItemRegistry::global();
                    if let Some(bl) = registry.get_item(btype as i16) {
                        if bl.get_item().is_block() {
                            crate::game::items::block::AsBlock::as_block(&bl.get_item()).unwrap().on_break(game, player, BlockPosition { x: packet.x, y: (packet.y - 1) as i32, z: packet.z}, packet, orig_type as i32)?;
                        }
                    } */
                    block.set_type(0);
                    game.block_updates.push(crate::game::Block {
                        position: crate::game::BlockPosition {
                            x: packet.x,
                            y: (packet.y + 0) as i32,
                            z: packet.z,
                        },
                        block: block.clone(),
                    });
                    log::debug!("orig_type: {}", orig_type);
                    game.broadcast_to_loaded(
                        &player,
                        ServerPacket::SoundEffect {
                            effect_id: 2001,
                            x: packet.x,
                            y: packet.y,
                            z: packet.z,
                            sound_data: orig_type as i32,
                        },
                    )?;
                    let registry = ItemRegistry::global();
                    if let Some(hand) = player.get_item_in_hand_mut() {
                        if let Some(item) = registry.get_item(hand.id as i16) {
                            if let Some(max_dmg) = item.get_item().max_uses() {
                                hand.damage += 1;
                                if hand.damage as u64 > max_dmg {
                                    hand.reset();
                                }     
                                player.sync_inventory();
                            }
                        }
                    }
                    if let Some(item) = registry.get_item(orig_type as i16) {
                        if let Some(block) = item.get_item().as_block() {
                            let tool = player.get_item_in_hand().clone();
                            if let Some(drop) = block.on_break(game, packet.clone(), player, tool) {
                                game.spawn_entity(Box::new(
                                    crate::game::entities::item_entity::ItemEntity::new(
                                        Position::from_pos(
                                            packet.x as f64,
                                            (packet.y as f64) + 1.0,
                                            packet.z as f64,
                                        ),
                                        game.ticks,
                                        drop,
                                        None,
                                    ),
                                ));
                            }
                        }
                    }
                }
                4 => {
                    use std::ops::Mul;
                    let item = player.get_item_in_hand().clone();
                    if item.id != 0 {
                        *player.get_item_in_hand() = ItemStack::default();
                        let mut pos = player.position.clone();
                        let add = player.position.get_direction().mul(2.0).to_array();
                        pos.y += 1.5;
                        game.spawn_entity(Box::new(
                            crate::game::entities::item_entity::ItemEntity::new(
                                pos, game.ticks, item, None,
                            ),
                        ));
                    }
                }
                _ => {}
            }
        }
        ClientPacket::CloseWindow(packet) => {
            let mut player = player.unwrap().unwrap();
            player.open_inventories.remove(&packet.window_id);
        }
        _ => {}
    }
    Ok(())
}
use crate::game::*;
pub enum WClick {
    Inventory(Window),
    Player(Arc<PlayerRef>),
}
use std::fmt::{self, *};
impl fmt::Display for WClick {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            WClick::Inventory(_) => write!(f, "WClick::Inventory"),
            WClick::Player(_) => write!(f, "WClick::Player"),
        }
    }
}
impl WClick {
    pub fn get_inventory(&self) -> std::cell::RefMut<'_, Inventory> {
        match self {
            WClick::Inventory(window) => window.inventory.borrow_mut(),
            WClick::Player(player) => player.get_inventory(),
        }
    }
}
fn window_click_handler(
    mut packet: crate::network::packet::WindowClick,
    player: Arc<PlayerRef>,
    game: &mut Game,
) -> anyhow::Result<()> {
    let registry = ItemRegistry::global();
    let mut inventory: WClick;
    let mut window_id = 0;
    let mut window_type = 0;
    if packet.window_id != 0 {
        if let Some(inv) = player.get_open_inventories().get(&packet.window_id) {
            window_type = inv.inventory_type;
            inventory = WClick::Inventory(inv.clone());
        } else {
            return Err(anyhow::anyhow!(
                "tried to access a window that doesn't exist!"
            ));
        }
        if packet.slot as usize > inventory.get_inventory().items.len() - 1 {
            packet.slot -= 1;
            inventory = WClick::Player(player.clone());
        }
    } else {
        window_id = packet.window_id;
        inventory = WClick::Player(player.clone());
    }
    log::debug!("Window: {}", inventory);
    let closure: anyhow::Result<()> = {
        // If picking up from a slot
        if packet.item_id != -1 {
            if window_type == 0 && packet.slot == 0 {
                let mut inv = inventory.get_inventory();
                log::debug!("Inv: {:?}", inv.items);
                let mut grid = Grid::default();
                grid[0][0] = if let Some(a) = inv.items.get(&1) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[1][0] = if let Some(a) = inv.items.get(&2) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[0][1] = if let Some(a) = inv.items.get(&3) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[1][1] = if let Some(a) = inv.items.get(&4) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                log::debug!("Grid balls:\n{:?}", grid);
                if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                    log::debug!("Got it!");
                    *inv.items.get_mut(&0).unwrap() = out;
                };
                for i in 1..5 {
                    if let Some(item) = inv.items.get_mut(&i) {
                        if item.count - 1 > 0 {
                            log::debug!("Reducing");
                            item.count -= 1;
                        } else {
                            item.count = 0;
                            item.id = 0;
                            item.damage = 0;
                        }
                    }
                }
            } else if window_type == 1 && packet.slot == 0 {
                let mut inv = inventory.get_inventory();
                let mut grid = Grid::default();
                grid[0][0] = if let Some(a) = inv.items.get(&1) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[1][0] = if let Some(a) = inv.items.get(&2) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[2][0] = if let Some(a) = inv.items.get(&3) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[0][1] = if let Some(a) = inv.items.get(&4) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[1][1] = if let Some(a) = inv.items.get(&5) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[2][1] = if let Some(a) = inv.items.get(&6) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[0][2] = if let Some(a) = inv.items.get(&7) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[1][2] = if let Some(a) = inv.items.get(&8) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                grid[2][2] = if let Some(a) = inv.items.get(&9) {
                    let a = a.clone();
                    if a.id == 0 {
                        None
                    } else {
                        Some(a)
                    }
                } else {
                    None
                };
                log::info!("Grid:\n{:?}", grid);
                if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                    log::debug!("Got it!");
                    *inv.items.get_mut(&0).unwrap() = out;
                };
                for i in 1..10 {
                    if let Some(item) = inv.items.get_mut(&i) {
                        if item.count - 1 > 0 {
                            log::debug!("Reducing");
                            item.count -= 1;
                        } else {
                            item.count = 0;
                            item.id = 0;
                            item.damage = 0;
                        }
                    }
                }
            }
            let cur = player.get_current_cursored_item_mut().clone();
            let mut is_some = cur.is_some();
            let is_some_real = is_some.clone();
            if is_some {
                is_some = cur.unwrap().id != 0;
            }
            let mut recv_item = item_packet_validator(&packet, player.clone(), &inventory)?;
            let mut inv = inventory.get_inventory();
            let slot = inv
                .get_slot(packet.slot as i8)
                .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
            if !is_some || packet.right_click == 1 {
                if packet.right_click == 0 {
                    if !packet.shift {
                        slot.reset();
                        drop(inv);
                        *player.get_current_cursored_item_mut() = Some(recv_item);
                    } else {
                        log::debug!("Shift click");
                        let mut slot = slot.clone();
                        if window_type == 1 {
                            inv = player.get_inventory();
                        }
                        for item in inv.items.iter_mut() {
                            if *item.0 as i16 == packet.slot {
                                continue;
                            }
                            let mut item = item.1;
                            if item.id == slot.id && item.damage == slot.damage {
                                let max_stack_size =
                                    registry.get_item(slot.id).unwrap().get_item().stack_size()
                                        as u64;
                                if item.count as u64 + slot.count as u64 > max_stack_size {
                                    for i in 0..max_stack_size - item.count as u64 {
                                        item.count += 1;
                                        slot.count -= 1;
                                    }
                                } else {
                                    item.count += slot.count;
                                    slot.count = 0;
                                }
                            }
                            if slot.count == 0 {
                                slot.id = 0;
                                slot.damage = 0;
                                break;
                            }
                        }
                        if slot.id != 0 {
                            let mut offset = 0;
                            if window_type == 0 {
                                offset = 9;
                            } else if window_type == 1 {
                                //inv = player.get_inventory();
                                offset = 9;
                            }
                            for i in offset..inv.items.len() {
                                let item = inv.items.get_mut(&(i as i8)).unwrap();
                                if item.id == 0 {
                                    *item = slot;
                                    slot.reset();
                                }
                            }
                            /*                             for item in inv.items.iter_mut() {
                                if (0..offset).contains(item.0) {
                                    continue;
                                }
                                let item = item.1;
                            } */
                        }
                        let slot_2 = inv
                            .get_slot(packet.slot as i8)
                            .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                        *slot_2 = slot;
                    }
                } else {
                    if !is_some {
                        let num = equal_half(slot.count as usize);
                        slot.count = num.1 as i8;
                        drop(inv);
                        recv_item.count = num.0 as i8;
                        *player.get_current_cursored_item_mut() = Some(recv_item);
                        log::debug!(
                            "Current cursored item: {:?}",
                            player.get_current_cursored_item_mut()
                        );
                    } else {
                        let slot = slot.clone();
                        drop(inv);
                        log::debug!("NumNutz");
                        let mut cursor2 = player.get_current_cursored_item_mut();
                        let cursor = cursor2.as_mut().unwrap();
                        if slot.id != cursor.id || slot.damage != cursor.damage {
                            return Err(anyhow::anyhow!("Tried to stack two different items!"));
                        }
                        let max_stack_size = ItemRegistry::global()
                            .get_item(slot.id)
                            .unwrap()
                            .get_item()
                            .stack_size();
                        if slot.count as u64 + 1 > max_stack_size as u64 {
                            player.sync_inventory();
                            return Err(anyhow::anyhow!("Tried to stack too large!"));
                        } else {
                            if cursor.count - 1 == 0 {
                                cursor.id = 0;
                                cursor.damage = 0;
                            }
                            cursor.count -= 1;
                            drop(cursor2);
                            let mut inv = inventory.get_inventory();
                            log::debug!("Slot: {:?}", packet.slot);
                            let slot = inv
                                .get_slot(packet.slot as i8)
                                .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                            slot.count += 1;
                            drop(inv);
                        }
                    }
                }
            } else if is_some {
                let slot = slot.clone();
                drop(inv);
                log::debug!("Else");
                if player.get_current_cursored_item_mut().is_none() {
                    return Err(anyhow::anyhow!("No item in cursor!"));
                }
                let cursored = player.get_current_cursored_item_mut().clone().unwrap();
                if (slot.id != cursored.id || slot.damage != cursored.damage) && packet.slot == 0 && window_type < 3 {
                    let mut inv = inventory.get_inventory();
                    for i in 1..10 {
                        if let Some(item) = inv.items.get_mut(&i) {
                            if item.count > 0 {
                                log::debug!("Increasing");
                                item.count += 1;
                            } else {
                                item.count = 0;
                                item.id = 0;
                                item.damage = 0;
                            }
                        }
                    }
                    return Ok(());
                } else if slot.id != cursored.id || slot.damage != cursored.damage {
                    let mut inv = inventory.get_inventory();
                    //log::debug!("Slot: {:?}", packet.slot);
                    let slot = inv
                        .get_slot(packet.slot as i8)
                        .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                    let cur2 = slot.clone();
                    *slot = cursored;
                    drop(inv);
                    *player.get_current_cursored_item_mut().as_mut().unwrap() = cur2;
                    //return Err(anyhow::anyhow!("Tried to stack two different items!"));
                } else {
                    let max_stack_size = ItemRegistry::global()
                        .get_item(slot.id)
                        .unwrap()
                        .get_item()
                        .stack_size();
                    if cursored.count as u64 + slot.count as u64 > max_stack_size as u64 {
                        log::debug!("Sugmanut");
                        for _ in 0..max_stack_size as u64 - slot.count as u64 {
                            player
                                .get_current_cursored_item_mut()
                                .as_mut()
                                .unwrap()
                                .count -= 1;
                            let mut inv = inventory.get_inventory();
                            //log::debug!("Slot: {:?}", packet.slot);
                            let slot = inv
                                .get_slot(packet.slot as i8)
                                .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                            slot.count += 1;
                            drop(inv);
                        }
                        //return Err(anyhow::anyhow!("Tried to stack too large!"));
                    } else if packet.slot != 0 && window_type < 3 {
                        let mut inv = inventory.get_inventory();
                        log::debug!("Slot: {:?}", packet.slot);
                        let slot = inv
                            .get_slot(packet.slot as i8)
                            .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                        slot.count += cursored.count;
                        drop(inv);
                        player
                            .get_current_cursored_item_mut()
                            .as_mut()
                            .unwrap()
                            .reset();
                    } else {
                        let mut inv = inventory.get_inventory();
                        log::debug!("Slot: {:?}", packet.slot);
                        let mut cursored = cursored;
                        let slot = inv
                            .get_slot(packet.slot as i8)
                            .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                        cursored.count += slot.count;
                        slot.reset();
                        drop(inv);
                        *player.get_current_cursored_item_mut().as_mut().unwrap() = cursored;
                    }
                }
            }
        } else {
            // If putting in a slot
            log::debug!("Putting in slot");
            if packet.slot > 0 {
                if player.get_current_cursored_item_mut().is_none() {
                    return Err(anyhow::anyhow!("No item in cursor!"));
                }
                if player.get_current_cursored_item_mut().unwrap().id == 0 {
                    return Err(anyhow::anyhow!("Cursor is empty!"));
                }
                if !in_registry(player.get_current_cursored_item_mut().as_ref().unwrap().id) {
                    return Err(anyhow::anyhow!("Item is not in registry!"));
                }
                let cursored = player.get_current_cursored_item_mut().unwrap().clone();
                let mut inv = inventory.get_inventory();
                log::debug!("Slot: {:?}", packet.slot);
                let slot = inv
                    .get_slot(packet.slot as i8)
                    .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?
                    .clone();
                drop(inv);
                let reg_item = ItemRegistry::global().get_item(cursored.id).unwrap();
                if let WClick::Player(_) = inventory {
                    if packet.slot == 0 {
                        return Err(anyhow::anyhow!("can't put items in crafting output!"));
                    }
                    use crate::game::items::ToolType;
                    if (5..9).contains(&packet.slot) {
                        if let Some(tool_type) = reg_item.get_item().get_tool_type() {
                            match tool_type {
                                ToolType::HELMET => {
                                    if packet.slot != 5 {
                                        return Err(anyhow::anyhow!("non-armor in armor slot!"));
                                    }
                                }
                                ToolType::CHESTPLATE => {
                                    if packet.slot != 6 {
                                        return Err(anyhow::anyhow!("non-armor in armor slot!"));
                                    }
                                }
                                ToolType::LEGGINGS => {
                                    if packet.slot != 7 {
                                        return Err(anyhow::anyhow!("non-armor in armor slot!"));
                                    }
                                }
                                ToolType::BOOTS => {
                                    if packet.slot != 8 {
                                        return Err(anyhow::anyhow!("non-armor in armor slot!"));
                                    }
                                }
                                _ => {
                                    return Err(anyhow::anyhow!("non-armor in armor slot!"));
                                }
                            }
                        } else {
                            return Err(anyhow::anyhow!("non-armor in armor slot!"));
                        }
                    }
                }
                if packet.right_click == 0 {
                    log::debug!("Slot: {:?}", slot);
                    player
                        .get_current_cursored_item_mut()
                        .as_mut()
                        .unwrap()
                        .reset();
                    let mut inv = inventory.get_inventory();
                    log::debug!("Slot: {:?}", packet.slot);
                    let slot = inv
                        .get_slot(packet.slot as i8)
                        .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                    *slot = cursored;
                    drop(inv);
                } else {
                    log::debug!("ELSE");
                    if slot.id == 0 {
                        log::debug!("Slot: {:?}", slot);
                        let mut cursor2 = player.get_current_cursored_item_mut();
                        let cursor = cursor2.as_mut().unwrap();
                        if cursor.count - 1 == 0 {
                            cursor.id = 0;
                            cursor.damage = 0;
                            //return Err(anyhow::anyhow!("Not enough items left!"));
                        }
                        cursor.count -= 1;
                        drop(cursor);
                        drop(cursor2);
                        let mut inv = inventory.get_inventory();
                        log::debug!("Slot: {:?}", packet.slot);
                        let slot = inv
                            .get_slot(packet.slot as i8)
                            .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
                        slot.id = cursored.id;
                        slot.count = 1;
                        drop(inv);
                    } else {
                    }
                }
                if (1..5).contains(&packet.slot) && window_type == 0 {
                    let mut inv = inventory.get_inventory();
                    log::debug!("Inv: {:?}", inv.items);
                    let mut grid = Grid::default();
                    grid[0][0] = if let Some(a) = inv.items.get(&1) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[1][0] = if let Some(a) = inv.items.get(&2) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[0][1] = if let Some(a) = inv.items.get(&3) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[1][1] = if let Some(a) = inv.items.get(&4) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    log::info!("Grid:\n{:?}", grid);
                    if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                        log::debug!("Got it!");
                        *inv.items.get_mut(&0).unwrap() = out;
                    };
                /*                     for i in 1..4 {
                    if let Some(item) = inv.items.get_mut(&i) {
                        if item.count > 0 {
                            item.count -= 1;
                        } else {
                            item.count = 0;
                            item.id = 0;
                            item.damage = 0;
                        }
                    }
                } */
                } else if (1..10).contains(&packet.slot) && window_type == 1 {
                    let mut inv = inventory.get_inventory();
                    let mut grid = Grid::default();
                    grid[0][0] = if let Some(a) = inv.items.get(&1) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[1][0] = if let Some(a) = inv.items.get(&2) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[2][0] = if let Some(a) = inv.items.get(&3) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[0][1] = if let Some(a) = inv.items.get(&4) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[1][1] = if let Some(a) = inv.items.get(&5) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[2][1] = if let Some(a) = inv.items.get(&6) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[0][2] = if let Some(a) = inv.items.get(&7) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[1][2] = if let Some(a) = inv.items.get(&8) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    grid[2][2] = if let Some(a) = inv.items.get(&9) {
                        let a = a.clone();
                        if a.id == 0 {
                            None
                        } else {
                            Some(a)
                        }
                    } else {
                        None
                    };
                    log::info!("Grid 2:\n{:?}", grid);
                    if let Some(out) = registry.get_solver_ref().solve(&mut grid) {
                        log::debug!("Got it!");
                        *inv.items.get_mut(&0).unwrap() = out;
                    };
                }
            } else {
                // TODO item drop
                log::debug!("Drop item here");
            }
        }
        Ok(())
    };
    if let Ok(_) = closure {
        player.write_packet(ServerPacket::Transaction {
            window_id: packet.window_id,
            action_number: packet.action_number,
            accepted: true,
        });
        return Ok(());
    } else if let Err(e) = closure {
        player.write_packet(ServerPacket::Transaction {
            window_id: packet.window_id,
            action_number: packet.action_number,
            accepted: false,
        });
        return Err(e);
    }
    Ok(())
}
pub fn item_packet_validator(
    packet: &crate::network::packet::WindowClick,
    player: Arc<PlayerRef>,
    inventory: &WClick,
) -> anyhow::Result<ItemStack> {
    if packet.item_count == Some(0) {
        return Err(anyhow::anyhow!("pick up an item of count 0!"));
    }
    if packet.item_count.is_none() {
        return Err(anyhow::anyhow!("count is none!"));
    }
    if packet.item_uses.is_none() {
        return Err(anyhow::anyhow!("damage is none!"));
    }
    let mut inv = inventory.get_inventory();
    let recv_item = ItemStack::new(
        packet.item_id,
        packet.item_uses.unwrap() as i16,
        packet.item_count.unwrap() as i8,
    );
    let slot = inv
        .get_slot(packet.slot as i8)
        .ok_or(anyhow::anyhow!("inventory slot does not exist!"))?;
    if *slot != recv_item {
        let slot = slot.clone();
        drop(inv);
        player.clear_cursor();
        player.sync_inventory();
        return Err(anyhow::anyhow!(
            "item is incorrect! expected {:?}, got {:?}!",
            slot,
            recv_item
        ));
    }
    if let None = ItemRegistry::global().get_item(slot.id) {
        slot.reset();
        drop(inv);
        player.clear_cursor();
        return Err(anyhow::anyhow!("item does not exist!"));
    }
    Ok(recv_item)
}
fn in_registry(id: i16) -> bool {
    ItemRegistry::global().get_item(id).is_some()
}
fn equal_half(input: usize) -> (usize, usize) {
    let x = input as f64 / 2.0;
    let x1: usize;
    let x2: usize;
    if x % 1.0 != 0.0 {
        x1 = (x - 0.5) as usize;
        x2 = (x + 0.5) as usize;
    } else {
        x1 = x as usize;
        x2 = x as usize;
    }
    (x1, x2)
}
