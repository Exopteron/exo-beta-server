use crate::game::Player;
use crate::game::{BlockPosition, DamageType, Game, ItemStack, Message};
use crate::network::ids::EntityID;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::server::Server;
use std::cell::RefCell;
use std::sync::Arc;
pub fn handle_packet(
    game: &mut Game,
    server: &mut Server,
    player: &mut Player,
    packet: ClientPacket,
) -> anyhow::Result<()> {
    match packet {
        ClientPacket::PlayerPacket(packet) => {
            if player.dead {
                return Ok(());
            }
            player.position.on_ground = packet.on_ground;
            player.last_position = player.position;
        }
        ClientPacket::PlayerLookPacket(packet) => {
            if player.dead {
                return Ok(());
            }
            player.position.yaw = packet.yaw;
            player.position.pitch = packet.pitch;
            player.position.on_ground = packet.on_ground;
            player.last_position = player.position;
        }
        ClientPacket::PlayerPositionPacket(packet) => {
            if player.dead {
                return Ok(());
            }
            player.position.x = packet.x;
            player.position.y = packet.y;
            player.position.stance = packet.stance;
            player.position.z = packet.z;
            player.position.on_ground = packet.on_ground;
            let pos = crate::game::Position::from_pos(3.0, 20.0, 5.0);
            if player.position.distance(&player.last_position) > 16.0 && player.last_position != pos
            {
                log::info!("Position: {:?}", player.position);
                if player.position.y < 0.0 || player.last_position.y < 0.0 {
                    player.last_position = pos;
                    player.position = pos;
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
                player.position = player.last_position;
                return Ok(());
            }
            player.last_position = player.position;
        }
        ClientPacket::PlayerPositionAndLookPacket(packet) => {
            if player.dead {
                return Ok(());
            }
            player.position.yaw = packet.yaw;
            player.position.pitch = packet.pitch;
            player.position.x = packet.x;
            player.position.y = packet.y;
            player.position.stance = packet.stance;
            player.position.z = packet.z;
            player.position.on_ground = packet.on_ground;
            let pos = crate::game::Position::from_pos(3.0, 20.0, 5.0);
            if player.position.distance(&player.last_position) > 16.0 && player.last_position != pos
            {
                log::info!("Position: {:?}", player.position);
                if player.position.y < 0.0 || player.last_position.y < 0.0 {
                    player.last_position = pos;
                    player.position = pos;
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
                player.position = player.last_position;
                return Ok(());
            }
            player.last_position = player.position;
        }
        ClientPacket::EntityAction(packet) => match packet.action {
            1 => {
                player.crouching = true;
            }
            2 => {
                player.crouching = false;
            }
            _ => {}
        },
        ClientPacket::ChatMessage(mut message) => {
            if message.message.len() > 64 {
                return Ok(());
            }
            log::info!("Message: {}", message.message);
            let slot = player
                .inventory
                .items
                .get_mut(&40)
                .expect("Player doesn't have expected slot!");
            slot.id = 50;
            slot.count = 64;
            let slot = player
                .inventory
                .items
                .get_mut(&41)
                .expect("Player doesn't have expected slot!");
            slot.id = 3;
            slot.count = 64;
            let slot = player
                .inventory
                .items
                .get_mut(&42)
                .expect("Player doesn't have expected slot!");
            slot.id = 285;
            slot.count = 1;
            if message.message.starts_with("/") {
                message.message.remove(0);
                let res = game.execute_command(player, &message.message)?;
                player.chatbox.push(Message::new(&format!("Command returned code {}.", res)));
            } else {
                let message = Message::new(&format!("<{}> {}", player.username, message.message));
                for (id, player_iter) in game.players.0.borrow().clone() {
                    if id == player.id {
                        player.chatbox.push(message.clone());
                    } else {
                        let mut player = player_iter.borrow_mut();
                        player.chatbox.push(message.clone());
                    }
                }
            }
        }
        ClientPacket::Respawn(_) => {
            if !player.dead {
                player.disconnect("Sent respawn packet when alive.".to_string());
                return Ok(());
            }
            player.dead = false;
            player.health = 20;
            player.position.x = 3.0;
            player.position.y = 20.0;
            player.position.z = 5.0;
            player.last_position = player.position;
            player.write(ServerPacket::Respawn {
                world: player.world,
            });
            //let id = player.id.0;
            game.hide_player(&player)?;
            //game.broadcast_to_loaded(&player, ServerPacket::DestroyEntity { eid: id })?;
            //game.broadcast_packet(ServerPacket::EntityStatus { eid: player.id.0, entity_status: 0x00 })?;
        }
        ClientPacket::Animation(packet) => {
            if packet.animate == 1 {
                let len = game.players.0.borrow().len().clone();
                for i in 0..len {
                    if i as i32 == player.id.0 {
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
                    let mut player_new = list2.borrow_mut();
                    drop(list);
                    if let Some(_) = player_new
                        .rendered_players
                        .get(&(player.id, player.username.clone()))
                    {
                        player_new.write(ServerPacket::Animation {
                            eid: player.id.0,
                            animate: 1,
                        });
                    } else {
                        continue;
                    }
                }
            }
        }
        ClientPacket::Disconnect(_) => {
            player.remove();
        }
        ClientPacket::HoldingChange(packet) => {
            player.held_slot = packet.slot_id;
        }
        ClientPacket::WindowClick(packet) => {
            if packet.window_id == 0 {
                if packet.item_id != -1 {
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
                        crate::systems::sync_inv_force(game, server, player)?;
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
                    if invslot.count == 0 {
                        *invslot = ItemStack::default();
                        crate::systems::sync_inv_force(game, server, player)?;
                        player.write(ServerPacket::Transaction {
                            window_id: 0,
                            action_number: packet.action_number,
                            accepted: false,
                        });
                        return Ok(());
                    }
                    if invslot.id != 0 {
                        if player.current_cursored_item.is_some()
                            && invslot.id == player.current_cursored_item.as_ref().unwrap().id
                            && packet.right_click == 0
                        {
                            //*invslot = ItemStack::default();
                            //log::info!("Maximum is {}", player.current_cursored_item.clone().unwrap().count.max(1));
                            invslot.count +=
                                player.current_cursored_item.clone().unwrap().count.max(1);
                            player.current_cursored_item = None;
                        } else {
                            *invslot = ItemStack::default();
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
                        crate::systems::sync_inv_force(game, server, player)?;
                        player.write(ServerPacket::Transaction {
                            window_id: 0,
                            action_number: packet.action_number,
                            accepted: false,
                        });
                        return Ok(());
                    }
                    let mut curcurid = player.current_cursored_item.clone().unwrap();
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
                            crate::systems::sync_inv_force(game, server, player)?;
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
                        *invslot = curcurid.clone();
                        player.current_cursored_item = None;
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
        ClientPacket::UseEntity(packet) => {
            let interval = std::time::Duration::from_millis(350);
            if packet.left_click {
                if player.since_last_attack + interval > std::time::Instant::now() {
                    return Ok(());
                }
                player.since_last_attack = std::time::Instant::now();
                //game.broadcast_packet(ServerPacket::SoundEffect { effect_id: 1001, x: player.position.x as i32, y: player.position.y as i8, z: player.position.z as i32, sound_data: 0 })?;
                let plrs = game.players.0.borrow();
                let plr = plrs.get(&EntityID(packet.target)).clone();
                if let Some(plr) = plr {
                    let plr = plr.try_borrow_mut();
                    if plr.is_ok() {
                        let mut plr = plr.unwrap();
                        if plr.dead {
                            return Ok(());
                        }
                        plr.damage(
                            DamageType::Player {
                                damager: player.username.clone(),
                            },
                            1,
                            Some(player),
                        );
                        use std::ops::Mul;
                        let arr = player.position.get_direction().mul(1980.0).to_array();
                        let x = arr[0];
                        let y = arr[1];
                        let z = arr[2];
                        log::info!("Adding velocity {} {} {}", x, y, z);
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
            let mut success = false;
            if packet.block_or_item_id >= 0 {
                if packet.x == -1 && packet.y == -1 && packet.z == -1 {
                    return Ok(());
                }
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
                let held = player.get_item_in_hand();
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
                    crate::systems::sync_inv_force(game, server, player)?;
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
                let pos = player.position.clone();
                let held = player.get_item_in_hand();
                for user in game.players.0.borrow().iter() {
                    let pos = user.1.try_borrow();
                    if pos.is_err() {
                        continue;
                    }
                    let pos = pos.unwrap().position.clone();
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
                //let pos = crate::world::World::pos_to_index(packet.x, packet.y as i32, packet.z as i32);
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
        ClientPacket::PlayerDigging(packet) => {
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
                            .get_block(packet.x, (packet.y - 1) as i32, packet.z)
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
                }
                _ => {}
            }
        }
        _ => {}
    }
    Ok(())
}
