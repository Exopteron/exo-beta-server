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
    player: Arc<PlayerRef>,
    packet: ClientPacket,
) -> anyhow::Result<()> {
    match packet {
        ClientPacket::PlayerPacket(packet) => {
            if player.is_dead() {
                return Ok(());
            }
            if packet.on_ground == false && player.get_checking_fall() {
                //log::info!("Off ground");
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
                //log::info!("Off ground");
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
                //log::info!("Off ground");
                player.set_offground_height(player.get_position_clone().y as f32);
                player.set_checking_fall(false);
            } else if packet.on_ground == true {
                player.set_checking_fall(true);
            }
            let mut pos = player.get_position();
            pos.x = packet.x;
            pos.y = packet.y;
            pos.stance = packet.stance;
            pos.z = packet.z;
            pos.on_ground = packet.on_ground;
            let pos = pos.clone();
            //let mut pos = crate::game::Position::from_pos(3.0, 20.0, 5.0);
            /*             if pos.distance(&player.last_position) > 16.0 && player.last_position != pos
            {
                log::info!("Position: {:?}", pos);
                if pos.y < 0.0 || player.last_position.y < 0.0 {
                    player.last_position = pos;
                    pos = pos;
                }
                player.write(ServerPacket::PlayerPositionAndLook {
                    x: player.last_position.x,
                    y: player.last_position.y,
                    stance: player.last_position.stance,
                    z: player.last_position.z,
                    yaw: player.last_position.yaw,
                    pitch: player.last_position.pitch,
                    on_ground: player.last_position.on_ground,
                });
                pos = player.last_position;
                return Ok(());
            } */
            let mut pos = pos.clone();
            player.set_last_position(pos);
            player.set_position(pos);
        }
        ClientPacket::PlayerPositionAndLookPacket(packet) => {
            if player.is_dead() {
                return Ok(());
            }
            if packet.on_ground == false && player.get_checking_fall() {
                //log::info!("Off ground");
                player.set_offground_height(player.get_position_clone().y as f32);
                player.set_checking_fall(false);
            } else if packet.on_ground == true {
                player.set_checking_fall(true);
            }
            let mut pos = player.get_position();
            pos.yaw = packet.yaw;
            pos.pitch = packet.pitch;
            pos.x = packet.x;
            pos.y = packet.y;
            pos.stance = packet.stance;
            pos.z = packet.z;
            pos.on_ground = packet.on_ground;
            /*             let mut pos = crate::game::Position::from_pos(3.0, 20.0, 5.0);
            if pos.distance(&player.last_position) > 16.0 && player.last_position != pos
            {
                log::info!("Position: {:?}", pos);
                if pos.y < 0.0 || player.last_position.y < 0.0 {
                    player.last_position = pos;
                    pos = pos;
                }
                player.write(ServerPacket::PlayerPositionAndLook {
                    x: player.last_position.x,
                    y: player.last_position.y,
                    stance: player.last_position.stance,
                    z: player.last_position.z,
                    yaw: player.last_position.yaw,
                    pitch: player.last_position.pitch,
                    on_ground: player.last_position.on_ground,
                });
                pos = player.last_position;
                return Ok(());
            } */
            let mut pos = pos.clone();
            player.set_last_position(pos);
            player.set_position(pos);
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
            if message.message.len() > 64 {
                return Ok(());
            }
            if message.message == "balls" {
                //game.spawn_entity(Box::new(crate::game::entities::item_entity::ItemEntity::new(player.get_position_clone(), game.ticks, ItemStack::new(1, 0, 1))));
            }
            let mut inv = player.get_inventory();
            log::info!("Message: {}", message.message);
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
            drop(inv);
            //player.sync_inventory();
            if message.message.starts_with("/") {
                message.message.remove(0);
                use std::ops::DerefMut;
                //log::info!("A");
                let res =
                    game.execute_command(player.unwrap().unwrap().deref_mut(), &message.message)?;
                //log::info!("B");
                player.send_message(Message::new(&format!("Command returned code {}.", res)));
            } else {
                //log::info!("sx");
                let message = message.message;
                //let message = message.replace("&", "§");
                let message = Message::new(&format!("<{}> {}", player.get_username(), message));
                //log::info!("bx");
                for (id, player_iter) in game.players.0.borrow().clone() {
                    if id == player.get_id() {
                        player.send_message(message.clone());
                    } else {
                        let mut player = player_iter;
                        player.send_message(message.clone());
                    }
                }
                //log::info!("nx");
            }
        }
        ClientPacket::Respawn(_) => {
            if !player.is_dead() {
                player.disconnect("Sent respawn packet when alive.".to_string());
                return Ok(());
            }
            player.set_dead(false);
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
            game.hide_player(&player.unwrap().unwrap())?;
            //game.broadcast_to_loaded(&player, ServerPacket::DestroyEntity { eid: id })?;
            //game.broadcast_packet(ServerPacket::EntityStatus { eid: player.id.0, entity_status: 0x00 })?;
        }
        ClientPacket::Animation(packet) => {
            if packet.animate == 1 {
                let len = game.players.0.borrow().len().clone();
                for i in 0..len {
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
                        continue;
                    }
                }
            }
        }
        ClientPacket::Disconnect(_) => {
            player.unwrap().unwrap().remove();
        }
        ClientPacket::HoldingChange(packet) => {
            player.set_held_slot(packet.slot_id);
        }
        ClientPacket::WindowClick(mut packet) => {
            use std::ops::DerefMut;
            let mut player = player.unwrap().unwrap();
            let mut player = player.deref_mut();
            let registry = ItemRegistry::global();
            log::info!("Window id: {}", packet.window_id);
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
                        log::info!("Declined! Tried to get {}", packet.slot);
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
                            //*invslot = ItemStack::default();
                            //log::info!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
                            if invslot.count
                                + player.current_cursored_item.clone().unwrap().count.max(1)
                                > registry.get_item(item.id).unwrap().get_item().stack_size() as i8
                            {
                                let thing = invslot.count
                                    + player.current_cursored_item.clone().unwrap().count.max(1);
                                let thing = thing
                                    - registry.get_item(item.id).unwrap().get_item().stack_size()
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
                    if (5..9).contains(&packet.slot) {
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
                }
                //player.last_transaction_id = packet.action_number;
                log::info!(
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
                return Ok(());
            } else {
                if let Some(window) = player.open_inventories.get_mut(&packet.window_id) {
                    let mut inventory = &mut window.inventory;
                    if inventory.items.len() < packet.slot as usize {
                        log::info!("switching to plr inv");
                        packet.slot -= 4;
                        //packet.slot -= inventory.items.len() as i16;
                        inventory = &mut player.inventory;
                    } else {
                        log::info!("Is some! {} {}", inventory.items.len(), packet.slot);
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
                        if 
                            inventory
                            .items
                            .get(&(packet.slot as i8))
                            .expect("Slot doesn't exist!")
                            != &item
                        {
                            log::info!("Declined! Tried to get {}", packet.slot);
                            player.sync_inventory();
                            //crate::systems::sync_inv_force(game, server, player)?;
                            player.write(ServerPacket::Transaction {
                                window_id: 0,
                                action_number: packet.action_number,
                                accepted: false,
                            });
                            return Ok(());
                        }
                        let invslot = 
                            inventory
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
                        if invslot.id != 0 {
                            if (*player).current_cursored_item.is_some()
                                && invslot.id == player.current_cursored_item.as_ref().unwrap().id
                                && packet.right_click == 0
                            {
                                //*invslot = ItemStack::default();
                                //log::info!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
                                if invslot.count
                                    + player.current_cursored_item.clone().unwrap().count.max(1)
                                    > registry.get_item(item.id).unwrap().get_item().stack_size() as i8
                                {
                                    let thing = invslot.count
                                        + player.current_cursored_item.clone().unwrap().count.max(1);
                                    let thing = thing
                                        - registry.get_item(item.id).unwrap().get_item().stack_size()
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
                    } else {
                        if player.current_cursored_item.is_none()
                            || (inventory.items.get(&(packet.slot as i8)).unwrap().id
                                != player.current_cursored_item.clone().unwrap().id
                                && inventory.items.get(&(packet.slot as i8)).unwrap().id != 0)
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
                    }
                    //player.last_transaction_id = packet.action_number;
                    log::info!(
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
        //                 log::info!("Declined! Tried to get {}", packet.slot);
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
        //                     //log::info!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
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
        //         log::info!(
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
                    if true {
                        let mut plr = plr.unwrap().unwrap();
                        if plr.dead {
                            return Ok(());
                        }
                        plr.damage(
                            DamageType::Player {
                                damager: player.username.clone(),
                            },
                            1,
                            Some(&mut player),
                        );
                        use std::ops::Mul;
                        let arr = player.position.get_direction().mul(1980.0).to_array();
                        let x = arr[0];
                        let y = arr[1];
                        let z = arr[2];
                        //log::info!("Adding velocity {} {} {}", x, y, z);
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
                    log::info!("Not, comparing {:?} to {:?}", item, *held);
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
                        exopacket.y += 1;
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
                        log::info!("Fal {}", x);
                        //return false;
                    }
                }
                let registry = ItemRegistry::global();
                if let Some(block) = game.world.get_block(packet.x, packet.y as i32, packet.z) {
                    log::info!("Block: {:?}", block);
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
                    log::info!("Block: {:?}", block);
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
                    log::info!("Not, comparing {:?} to {:?}", item, *held);
                    despawn = true;
                    return Ok(());
                } else {
                    despawn = false;
                }
                log::info!("Block: {:?} {:?} {:?}", packet.x, packet.y, packet.z);
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
                log::info!(
                    "Setting at X: {} Y: {} Z: {}",
                    packet.x,
                    packet.y as i32,
                    packet.z
                );
                if block.get_type() == 0 {
                    //player.write(ServerPacket::BlockChange { x: packet.x, y: packet.y, z: packet.z, block_type: item.id as i8, block_metadata: 0x00 });
                    log::info!("Setting block.");
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
            /*             match packet.face {
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
                _ => {}
            } */
            match packet.status {
                0 => {
                    player.mining_block.block = BlockPosition {
                        x: packet.x,
                        y: packet.y as i32,
                        z: packet.z as i32,
                    };
                    player.mining_block.face = packet.face;
                }
                2 => {
                    log::info!("Got pos {} {} {}", packet.x, packet.y, packet.z);
                    let block = if let Some(blk) =
                        game.world
                            .get_block(packet.x, (packet.y - 0) as i32, packet.z)
                    {
                        blk
                    } else {
                        return Ok(());
                    };
                    let orig_type = block.b_type.clone();
                    if player.mining_block.block
                        != (BlockPosition {
                            x: packet.x,
                            y: packet.y as i32,
                            z: packet.z,
                        })
                    {
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
                    log::info!("orig_type: {}", orig_type);
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
        _ => {}
    }
    Ok(())
}
