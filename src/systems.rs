use crate::server::Server;
use crate::game::{Game, PlayerRef, Player, Message, DamageType, ChunkCoords};
use crate::network::ids::{EntityID, IDS};
use crate::network::packet::{ClientPacket, ServerPacket};
use std::sync::Arc;
use std::time::{Duration, Instant};
pub struct Systems {
    systems: Vec<Box<dyn FnMut(&mut crate::game::Game) -> anyhow::Result<()> + 'static>>,
}
impl Systems {
    pub fn new() -> Self {
        Self { systems: Vec::new() }
    }
    pub fn add_system(&mut self, system: impl FnMut(&mut crate::game::Game) -> anyhow::Result<()> + 'static) {
        self.systems.push(Box::new(system));
    }
    pub fn run(&mut self, game: &mut crate::game::Game) {
        for system in &mut self.systems {
            if let Err(e) = system(game) {
                log::error!("System returned an error. Details: {:?}", e);
            }
        }
    }
}
pub fn ping(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let interval = Duration::from_millis(150);
    if server.last_ping_time + interval < Instant::now() {
        let mut clients = server.clients.borrow_mut();
        let mut remove = Vec::new();
        for client in clients.iter_mut() {
            let mut cl = client.1.borrow_mut();
            if let Err(_) = cl.write(ServerPacket::KeepAlive) {
                 remove.push(cl.id);
            } 
        }
        for id in remove {
            let username = if let Some(plr) = game.players.0.borrow().get(&id) {
                plr.get_username() // borrow().username.clone()
            } else {
                clients.remove(&id);
                continue;
            };
            log::info!("{} left the game.", username);
            game.players.0.borrow_mut().remove(&id);
            clients.remove(&id);
            IDS.lock().unwrap().push(id.0);
        }
        server.last_ping_time = Instant::now();
    }
    Ok(())
}
/* pub fn update_local_health(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let players = game.players.0.borrow();
    for player in players.iter() {
        let mut cl = player.1;
        if cl.health != cl.last_health {
            let health = cl.health;
            cl.write_packet(ServerPacket::UpdateHealth { health: health as i16 });
            cl.last_health = cl.health;
        }
    }
    Ok(())
} */
pub fn time_update(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    game.time += 1;
    game.time %= 24000;
    let players = game.players.0.borrow();
    for player in players.iter() {
/*         let mut cl = player.1.borrow_mut(); */
        // cl.
        player.1.write_packet(ServerPacket::TimeUpdate { time: game.time });
    }
    Ok(())
}
pub fn tick_entities(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let interval = Duration::from_millis(750);
    let entities = game.entities.borrow().clone();
    for entity in entities.iter() {
        if game.loaded_chunks.0.contains(&entity.1.borrow_mut().get_position().to_chunk_coords()) {
            entity.1.borrow_mut().tick(game);
        }
    }
    Ok(())
}
pub fn tick_players(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let interval = Duration::from_millis(750);
    let players = game.players.0.borrow().clone();
    for player in players.iter() {
        player.1.tick(game)?;
    }
    Ok(())
}
/* pub fn check_void(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let interval = Duration::from_millis(750);
    let players = game.players.0.borrow();
    for player in players.iter() {
        let mut cl = player.1; // .borrow_mut();
        if cl.position.y <= 0.0 && !cl.dead {
            if cl.last_void_dmg + interval < Instant::now() {
                cl.damage(DamageType::Void, 3, None);
/*                 cl.health -= 3; */
                cl.last_void_dmg = Instant::now();
            }
        }
    }
    Ok(())
} */
pub fn block_updates(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    for _ in 0..game.block_updates.len() {
        let update = game.block_updates.pop().unwrap();
        let clients = game.players.0.borrow();
        for client in clients.iter() {
            if client.1.get_loaded_chunks().contains(&update.position.to_chunk_coords()) {
            //let mut cl = client.1.borrow_mut();
            client.1.write_packet(ServerPacket::BlockChange { x: update.position.x, y: update.position.y as i8, z: update.position.z , block_type: update.block.b_type as i8, block_metadata: update.block.b_metadata as i8 });
            }
        }
    }
    Ok(())
}
/* pub fn check_dead(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let players = game.players.0.borrow().clone();
    for player in players.iter() {
        let mut cl = player.1.borrow_mut();
        if cl.health <= 0 && !cl.dead {
            let mut msg = Message::new(&format!("{} died.", cl.username));
            match &cl.last_dmg_type {
                DamageType::None => {

                }
                DamageType::Void => {
                    msg = Message::new(&format!("{} fell into the abyss.", cl.username));
                }
                DamageType::Player { damager } => {
                    msg = Message::new(&format!("{} was beaten to death by {}.", cl.username, damager));
                }
            }
            let id = cl.id.0;
            game.broadcast_to_loaded(&cl, ServerPacket::EntityStatus { eid: id, entity_status: 3 })?;
            game.broadcast_message(msg.clone())?;
            cl.chatbox.push(msg);
            println!("Yo!");
            cl.write(ServerPacket::UpdateHealth { health: 0 });
            cl.dead = true;
        }
    }
    Ok(())
} */
 pub fn sync_inv_force(game: &mut Game, server: &mut Server, player: &mut Player) -> anyhow::Result<()> {
    player.write(ServerPacket::InvWindowItems { inventory: player.inventory.clone() });
    player.last_inventory = player.inventory.clone();
    Ok(())
}
/* pub fn check_chunks(game: &mut Game, server: &mut Server, player: &mut Player) -> anyhow::Result<()> {
    //let len = player.loaded_chunks.len();
    let pos = player.position.clone();
    let mut packets = vec![];
    player.loaded_chunks.retain(|chunk| {
        if chunk.distance(&ChunkCoords::from_pos(&pos)) > 3 {
            packets.push(ServerPacket::PreChunk { x: chunk.x, z: chunk.z, mode: false });
            return false;
        }
        true
    });
    for packet in packets {
        player.write(packet);
    }
    for x in -3..3 {
        for z in -3..3 {
            //let coords = ChunkCoords { x: x, z: z };
            let mut coords = ChunkCoords::from_pos(&pos);
            coords.x += x;
            coords.z += z;
            if game.world.check_chunk_exists(coords)/*  && !(x == 0 && z == 0) */ {
                if !player.loaded_chunks.contains(&coords) {
                    player.loaded_chunks.push(coords);
                    let packets = game.world.to_packets_chunk(coords, &mut player.packet_send_sender, &mut player.has_loaded_before);
                    if packets.is_none() {
                        continue;
                    }
                }
            }
        }
    }
    let mut packets = vec![];
    player.rendered_players.retain(|id, _| {
        packets.push(ServerPacket::DestroyEntity { eid: id.0.0 });
        false
    });
    Ok(())
} */
/* pub fn check_inv(game: &mut Game, server: &mut Server, player: &mut Player) -> anyhow::Result<()> {
    let len = player.inventory.items.len();
    for i in 0..len {
        let item = player.inventory.items.get_mut(&(i as i8)).unwrap();
        if item.count == 0 {
            item.id = 0;
            item.damage = 0;
        }
    }
    Ok(())
} */
/* pub fn sync_inv(game: &mut Game, server: &mut Server, player: &mut Player) -> anyhow::Result<()> {
    if player.inventory != player.last_inventory {
        player.write(ServerPacket::InvWindowItems { inventory: player.inventory.clone() });
        player.last_inventory = player.inventory.clone();
    }
    Ok(())
} */
/*
            cl.health = 20;
            cl.position.x = 0.0;
            cl.position.y = 120.0;
            cl.position.z = 0.0;
*/
pub fn sync_positions(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    for player in game.players.0.borrow().iter() {
        let mut player = player.1;
        let position = player.get_position_clone();
        if position != player.get_last_position_clone() {
            player.write_packet(ServerPacket::PlayerPositionAndLook {x: position.x, stance: position.stance, y: position.y, z: position.z, yaw: position.yaw, pitch: position.pitch, on_ground: position.on_ground});
        }
        //player.write(ServerPacket::PlayerTeleport { player_id: -1, position })?;
    }
    Ok(())
}
pub fn check_loaded_chunks(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let players = game.players.0.borrow().clone();
    //log::info!("Loaded chunks: {:?}", game.loaded_chunks.0);
    game.loaded_chunks.0.retain(|chunk| {
        for player in players.iter() {
            let player = player.1;
            if player.get_loaded_chunks().contains(chunk) {
                return true;
            }
            //let position = player.get_position_clone();
            //player.write(ServerPacket::PlayerTeleport { player_id: -1, position })?;
        }
        false
    });
    Ok(())
}
pub fn update_crouch(game: &mut Game, server: &mut Server, player_upd: Arc<PlayerRef>) -> anyhow::Result<()> {
    log::debug!("update_crouch called!");
    let len = game.players.0.borrow().len().clone();
    for i in 0..len {
        if i as i32 == player_upd.get_id().0 {
            continue;
        }
        let list = game.players.0.borrow();
/*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
        let list2 = if let Some(plr) = list.get(&crate::network::ids::EntityID(i as i32)) {
            plr.clone()
        } else {
            continue;
        };
        let mut player = list2.unwrap().unwrap();
        drop(list);
        if let Some(_) = player.rendered_players.get(&(player_upd.get_id(), player_upd.get_username())) {
            let animate = match player_upd.is_crouching() {
                true => {
                    104
                }
                false => {
                    105
                }
            };
            log::info!("Sending animation packet!");
            player.write(ServerPacket::Animation { eid: player_upd.get_id().0, animate: 0 });
            player.write(ServerPacket::Animation { eid: player_upd.get_id().0, animate: 104 });
        } else {
            continue;
        }
    }
    Ok(())
}
/* pub fn update_held_items(game: &mut Game, server: &mut Server, player_upd: Arc<Player>) -> anyhow::Result<()> {
    //log::info!("update_held_items called!");
    let mut item = player_upd.get_item_in_hand_ref().clone();
    if item.id == 0 {
        item.id = -1;
    }
    game.broadcast_to_loaded(player_upd, ServerPacket::EntityEquipment { eid: player_upd.id.0, slot: 0, item_id: item.id, damage: 0 })?;
    Ok(())
} */
pub fn rem_old_clients(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let len = server.clients.borrow().len().clone();
    for i in 0..len {
        let mut list = server.clients.borrow_mut();
/*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
        if let Some(_) = list.get(&crate::network::ids::EntityID(i as i32)) {
            if game.players.0.borrow().get(&crate::network::ids::EntityID(i as i32)).is_none() {
                list.remove(&crate::network::ids::EntityID(i as i32));
            }
        } else {
            continue;
        };
    }
    Ok(())
}
pub fn update_positions(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let len = game.players.0.borrow().len().clone();
    for i in 0..len {
        let list = game.players.0.borrow();
/*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
        let list2 = if let Some(plr) = list.get(&crate::network::ids::EntityID(i as i32)) {
            plr.clone()
        } else {
            continue;
        };
        let mut player = list2; // .borrow_mut();
        drop(list);
        let mut packets = Vec::new();
        for id in player.unwrap().unwrap().rendered_players.iter_mut() {
            let pos = if let Some(plr) = game.players.0.borrow().get(&id.0.0) {
                plr.get_position_clone()
            } else {
                continue;
            };
            if id.1.position != pos {
                if pos.distance(&id.1.position) < 3.5 && true == false {
                    let x_diff = (pos.x - id.1.position.x);
                    let y_diff = (pos.y - id.1.position.y);
                    let z_diff = (pos.z - id.1.position.z);
                    packets.push(ServerPacket::EntityLookAndRelativeMove { eid: id.0.0.0, dX: (x_diff * 32.0) as i8, dY: (y_diff * 32.0) as i8, dZ: (z_diff * 32.0) as i8, yaw: pos.yaw as i8, pitch: pos.pitch as i8});
                    //log::info!("Sending relative");
                } else {
                    //log::info!("Sending absolute");
                    packets.push(ServerPacket::EntityTeleport { eid: id.0.0.0, x: (pos.x * 32.0) as i32, y: (pos.y * 32.0) as i32, z: (pos.z * 32.0) as i32, yaw: pos.yaw as i8, pitch: pos.pitch as i8});
                }
                //log::info!("Sending entity teleport!");
                //packets.push(ServerPacket::EntityLook { eid: id.0.0.0, yaw: pos.yaw as i8, pitch: pos.pitch as i8 });
            }
            id.1.position = pos;
            //log::info!("tping {} from {:?} to {:?}", id.0.0, player.id, pos);
        }
        for packet in packets {
            player.write_packet(packet);
        }
    }
    Ok(())
}
pub fn entity_positions(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let len = game.players.0.borrow().len().clone();
    for i in 0..len {
        let list = game.players.0.borrow();
/*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
        let list2 = if let Some(plr) = list.get(&crate::network::ids::EntityID(i as i32)) {
            plr.clone()
        } else {
            continue;
        };
        let mut player = list2; // .borrow_mut();
        drop(list);
        let mut packets = Vec::new();
        for id in player.unwrap().unwrap().rendered_entities.iter_mut() {
            let pos = if let Some(plr) = game.entities.borrow().get(&id.0) {
                let mut plr = plr.borrow_mut();
                if !plr.broadcast_pos_change() {
                    continue;
                }
                plr.get_position().clone()
            } else {
                continue;
            };

            if id.1.position != pos {
                //log::info!("Sending entity teleport!");
                packets.push(ServerPacket::EntityTeleport { eid: id.0.0, x: (pos.x * 32.0) as i32, y: (pos.y * 32.0) as i32, z: (pos.z * 32.0) as i32, yaw: pos.yaw as i8, pitch: pos.pitch as i8});
                //packets.push(ServerPacket::EntityLook { eid: id.0.0.0, yaw: pos.yaw as i8, pitch: pos.pitch as i8 });
            }
            id.1.position = pos;
            //log::info!("tping {} from {:?} to {:?}", id.0.0, player.id, pos);
        }
        for packet in packets {
            player.write_packet(packet);
        }
    }
    Ok(())
}
pub fn spawn_players(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let len = game.players.0.borrow().len().clone();
    for i in 0..len {
        let list = game.players.0.borrow();
/*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
        let list2 = if let Some(plr) = list.get(&crate::network::ids::EntityID(i as i32)) {
            plr.clone()
        } else {
            continue;
        };
        let mut player = list2;
        drop(list);
        for (id, player_obj) in game.players.0.borrow_mut().iter() {
            if id == &player.get_id() {
                continue;
            }
            let mut player_obj = player_obj;
            if player_obj.unwrap().unwrap().rendered_players.get(&(player.get_id(), player.get_username())).is_none() && player_obj.get_loaded_chunks().contains(&ChunkCoords::from_pos(&player.get_position_clone())) {
                player_obj.unwrap().unwrap().rendered_players.insert((player.get_id(), player.get_username()), crate::game::RenderedPlayerInfo {position: player.get_position_clone(), held_item: player.get_item_in_hand_clone() });
                let pos = player.get_position_clone();
                player_obj.write_packet(ServerPacket::NamedEntitySpawn { eid: player.get_id().0, name: player.get_username(), x: (pos.x * 32.0).round() as i32, y: (pos.y * 32.0).round() as i32, z: (pos.z * 32.0).round() as i32, rotation: 0, pitch: 0, current_item: 0 });
            }
        }
    }
    Ok(())
}
/* pub fn chat_msgs(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let players = game.players.0.borrow();
    for player in players.iter() {
        let mut cl = player.1.borrow_mut();
        cl.chatbox.clone().messages.retain(|message| {
            cl.write(ServerPacket::ChatMessage { message: message.message.clone() });
            false
        });
        cl.chatbox = crate::game::Chatbox::default();
    }
    Ok(())
} */
pub fn cull_players(game: &mut Game, server: &mut Server) -> anyhow::Result<()> {
    let len = game.players.0.borrow().len().clone();
    for i in 0..len + 1 {
        let list = game.players.0.borrow();
/*         let list2 = list[&crate::network::ids::EntityID(i as i8)].clone(); */
        let list2 = if let Some(plr) = list.get(&crate::network::ids::EntityID(i as i32)) {
            plr.clone()
        } else {
            continue;
        };
        let mut player = list2;
        drop(list);
        let mut to_derender = Vec::new();
        let our_name = player.get_username();
        //log::info!("For {}, len {}", our_name, player.rendered_players.len());
        player.unwrap().unwrap().rendered_players.retain(|idname, _| {
            let (id, name) = idname;
          //  log::info!("For {}, {}", our_name, name);
            if game.players.0.borrow().get(id).is_none() || &game.players.0.borrow().get(id).unwrap().get_username() != name { // .borrow().username
            //    log::info!("For {}, derendering {}", our_name, name);
                to_derender.push(id.clone());
                return false;
            }
            true
        });
        for id in &to_derender {
            log::info!("Run");
            player.write_packet(ServerPacket::DestroyEntity { eid: id.0 });
        }
    }
    Ok(())
}