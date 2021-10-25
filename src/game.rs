use crate::commands::*;
use crate::configuration::CONFIGURATION;
use crate::game::events::*;
use crate::network::ids::EntityID;
use crate::network::ids::IDS;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::objects::Objects;
use crate::server::Server;
use crate::systems::Systems;
use crate::world::mcregion::MCRegionLoader;
//pub mod aa_bounding_box;
use items::*;
pub mod entities;
pub mod events;
pub mod gamerule;
pub mod items;
pub mod raycast;
use entities::*;
use events::*;
use flume::{Receiver, Sender};
use once_cell::sync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
pub struct GameGlobals {
    time: i64,
}
pub struct GameGlobalRef {
    globals: Mutex<Option<GameGlobals>>,
}
impl GameGlobalRef {
    pub fn set(&self, globals: GameGlobals) {
        *self.globals.lock().unwrap() = Some(globals);
    }
    pub fn get_time(&self) -> i64 {
        self.globals.lock().unwrap().as_ref().unwrap().time
    }
    pub fn set_time(&self, time: i64) {
        self.globals.lock().unwrap().as_mut().unwrap().time = time;
    }
}
pub static GAME_GLOBAL: Lazy<GameGlobalRef> = Lazy::new(|| {
    GameGlobalRef { globals: Mutex::new(None) }
});
#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Eq for BlockPosition {}
impl BlockPosition {
    /// Check if position is within a block.
    pub fn within_block(&self, pos: Position) -> bool {
        let min_x = self.x as f64;
        let min_y = self.y as f64;
        let min_z = self.z as f64;

        let max_x = self.x as f64 + 1.0;
        let max_y = self.y as f64 + 1.0;
        let max_z = self.z as f64 + 1.0;
        (pos.x <= max_x && pos.x >= min_x)
            && (pos.y <= max_y && pos.y >= min_y)
            && (pos.z <= max_z && pos.z >= min_z)
    }
    /// Convert block position to chunk coordinates.
    pub fn to_chunk_coords(&self) -> ChunkCoords {
        let x = self.x as i32 / 16;
        let z = self.z as i32 / 16;
        ChunkCoords { x, z }
    }
    /// Get +x,-x,+y,-y,+z,-z offsets from a block position.
    pub fn all_directions(&self) -> impl Iterator<Item = BlockPosition> {
        let mut list = Vec::with_capacity(6);
        let mut clone = self.clone();
        clone.x += 1;
        list.push(clone);

        let mut clone = self.clone();
        clone.y += 1;
        list.push(clone);

        let mut clone = self.clone();
        clone.z += 1;
        list.push(clone);

        let mut clone = self.clone();
        clone.x -= 1;
        list.push(clone);

        let mut clone = self.clone();
        clone.y -= 1;
        list.push(clone);

        let mut clone = self.clone();
        clone.z -= 1;
        list.push(clone);
        list.into_iter()
    }
}
impl std::default::Default for BlockPosition {
    fn default() -> Self {
        Self { x: 0, y: 0, z: 0 }
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FixedPointShort(pub i16);
impl std::ops::Add<i16> for FixedPointShort {
    type Output = Self;
    fn add(mut self, other: i16) -> Self {
        self.0 += (other << 5) + Position::FEET_DISTANCE;
        self
    }
}
impl std::ops::Add for FixedPointShort {
    type Output = Self;
    fn add(mut self, other: FixedPointShort) -> Self {
        self.0 += other.0;
        self
    }
}
impl std::ops::AddAssign for FixedPointShort {
    fn add_assign(&mut self, other: FixedPointShort) {
        self.0 += other.0
    }
}
impl std::ops::AddAssign<i16> for FixedPointShort {
    fn add_assign(&mut self, other: i16) {
        *self += FixedPointShort::from(other)
    }
}
impl std::convert::From<i16> for FixedPointShort {
    fn from(other: i16) -> Self {
        Self((other << 5) + 16)
    }
}
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Position {
    pub x: f64,
    pub stance: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
}
#[derive(Copy, Clone)]
pub struct Block {
    pub position: BlockPosition,
    pub block: crate::world::chunks::Block,
}
use glam::DVec3;
impl std::default::Default for Position {
    fn default() -> Self {
        Self {
            x: 0.,
            stance: 67.240000009536743,
            y: 0.,
            z: 0.,
            yaw: 0.,
            pitch: 0.,
            on_ground: false,
        }
    }
}
use std::fmt;
impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
impl Position {
    pub const FEET_DISTANCE: i16 = 51;
    /// Position from x, y, z
    pub fn from_pos(x: f64, y: f64, z: f64) -> Self {
        Position {
            x,
            stance: 67.240000009536743,
            y,
            z,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
        }
    }
    pub fn as_vec3(&self) -> DVec3 {
        DVec3::new(self.x, self.y, self.z)
    }
    pub fn from_vec3(vec3: DVec3) -> Self {
        Self {
            x: vec3.x,
            y: vec3.y,
            z: vec3.z,
            ..Position::default()
        }
    }
    pub fn move_towards(&mut self, other: &Position, amount: f64) {
        if other.x < self.x {
            self.x -= amount;
        } else {
            self.x += amount;
        }
        if other.y < self.y {
            self.y -= amount;
        } else {
            self.y += amount;
        }
        if other.z < self.z {
            self.z -= amount;
        } else {
            self.z += amount;
        }
    }
    /// Distance to other position
    pub fn distance(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
    /// Direction this position is facing
    pub fn get_direction(&self) -> DVec3 {
        //let mut vector = DVec3::new(0.0, 0.0, 0.0);
        let rot_x = self.yaw as f64;
        let rot_y = self.pitch as f64;
        let vector_y = rot_y.to_radians().sin();
        let xz = rot_y.to_radians().cos();
        let vector_x = -xz * rot_x.to_radians().sin();
        let vector_z = xz * rot_x.to_radians().cos();
        DVec3::new(vector_x, vector_y, vector_z)
    }
    /// If block is inside this position
    pub fn contains_block(&self, block_position: BlockPosition) -> bool {
        let mut us_pos = BlockPosition {
            x: (self.x + 0.1).floor() as i32,
            y: (self.y + 0.1).floor() as i32,
            z: (self.z + 0.1).floor() as i32,
        };
        if us_pos == block_position {
            return true;
        }
        us_pos.y += 1;
        if us_pos == block_position {
            return true;
        }
        false
    }
    /// Position to chunk coordinates
    pub fn to_chunk_coords(&self) -> ChunkCoords {
        let x = self.x as i32 / 16;
        let z = self.z as i32 / 16;
        ChunkCoords { x, z }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct MiningBlockData {
    pub block: BlockPosition,
    pub face: i8,
}
impl std::default::Default for MiningBlockData {
    fn default() -> Self {
        Self {
            block: BlockPosition::default(),
            face: 0,
        }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum DamageType {
    Void,
    Player { damager: String },
    None,
    Fall,
    Mob { damager: String },
    Drown,
}
#[derive(Clone, PartialEq, Debug, Hash, Copy, Ord)]
pub struct ItemStack {
    pub id: i16,
    pub damage: i16,
    pub count: i8,
}
use std::cmp::*;
impl PartialOrd for ItemStack {
    fn partial_cmp(&self, other: &ItemStack) -> Option<Ordering> {
        Some(self.count.cmp(&other.count))
    }
}
//impl std::borrow::Borrow for ItemStack {}
impl Eq for ItemStack {}
impl std::default::Default for ItemStack {
    fn default() -> Self {
        Self {
            id: 0,
            damage: 0,
            count: 0,
        }
    }
}
impl ItemStack {
    /// New itemstack from id, damage and count.
    pub fn new(id: i16, damage: i16, count: i8) -> Self {
        Self { id, damage, count }
    }
    /// Reset itemstack to default.
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}
#[derive(Clone, PartialEq, Debug)]
pub struct Inventory {
    pub items: HashMap<i8, ItemStack>,
}
impl std::default::Default for Inventory {
    fn default() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}
impl Inventory {
    /// Default player inventory.
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        for i in 0..45 {
            slots.insert(i, ItemStack::default());
        }
        Self { items: slots }
    }
    pub fn get_slot(&mut self, slot: i8) -> Option<&mut ItemStack> {
        self.items.get_mut(&slot)
    }
    /// Insert an itemstack into a player inventory.
    pub fn insert_itemstack(&mut self, stack: ItemStack) -> Option<()> {
        'main_1: for i in 36..45 {
            let slot = if let Some(s) = self.items.get_mut(&i) {
                s
            } else {
                continue;
            };
            let num = i;
            if num > 8 {
                if slot.id == stack.id && slot.damage == stack.damage {
                    let registry = ItemRegistry::global();
                    let mut our_count = stack.count;
                    if let Some(item) = registry.get_item(slot.id) {
                        for _ in 0..stack.count {
                            if slot.count as u64 + our_count as u64
                                > item.get_item().stack_size() as u64
                            {
                                continue 'main_1;
                            }
                            our_count -= 1;
                            slot.count += 1;
                        }
                    }
                    //slot.count += stack.count;
                    return Some(());
                }
            }
        }
        // TODO this will check what was just checked, fix that.
        'main: for (num, slot) in &mut self.items {
            if num > &8 {
                if slot.id == stack.id && slot.damage == stack.damage {
                    let registry = ItemRegistry::global();
                    let mut our_count = stack.count;
                    if let Some(item) = registry.get_item(slot.id) {
                        for _ in 0..stack.count {
                            if slot.count as u64 + our_count as u64
                                > item.get_item().stack_size() as u64
                            {
                                continue 'main;
                            }
                            our_count -= 1;
                            slot.count += 1;
                        }
                    }
                    //slot.count += stack.count;
                    return Some(());
                }
            }
        }
        // TODO this has the same issue as above
        for i in 36..45 {
            let slot = if let Some(s) = self.items.get_mut(&i) {
                s
            } else {
                continue;
            };
            let num = i;
            if num > 8 {
                if slot.id == 0 {
                    slot.id = stack.id;
                    slot.damage = stack.damage;
                    slot.count = stack.count;
                    return Some(());
                }
            }
        }
        for (num, slot) in &mut self.items {
            if num > &8 {
                if slot.id == 0 {
                    slot.id = stack.id;
                    slot.damage = stack.damage;
                    slot.count = stack.count;
                    return Some(());
                }
            }
        }
        Some(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}
impl Eq for ChunkCoords {}
//use num::num_integer::roots::Roots;
impl ChunkCoords {
    /// Check distance to another chunk coordinate.
    pub fn distance(&self, other: &ChunkCoords) -> f64 {
        (((self.x - other.x).pow(2) + (self.z - other.z).pow(2)) as f64).sqrt()
    }
    /// Chunk coordinates from a position.
    pub fn from_pos(position: &Position) -> Self {
        Self {
            x: (position.x / 16.0) as i32,
            z: (position.z / 16.0) as i32,
        }
    }
}
#[derive(Clone, Debug)]
pub struct RenderedPlayerInfo {
    pub position: Position,
    pub held_item: ItemStack,
}
#[derive(Clone, Debug)]
pub struct RenderedEntityInfo {
    pub position: Position,
}
#[derive(Clone)]
pub struct Chatbox {
    pub messages: Vec<Message>,
}
impl Chatbox {
    /// Add message to chatbox.
    pub fn push(&mut self, message: Message) {
        self.messages.push(message);
    }
}
impl std::default::Default for Chatbox {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}
use std::time::{Duration, Instant};
#[derive(Clone)]
pub struct PlayerRef {
    player: Arc<RefCell<Player>>,
}
use crate::network::metadata::Metadata;
impl PlayerRef {
    /// Load chunks & teleport player to position.
    pub fn teleport(&self, game: &mut Game, position: &Position) {
        self.set_position(*position);
        self.check_chunks(game);
        let pos = position;
        let packet = ServerPacket::PlayerPositionAndLook {
            x: pos.x,
            y: pos.y,
            z: pos.z,
            yaw: pos.yaw,
            pitch: pos.pitch,
            on_ground: pos.on_ground,
            stance: pos.stance,
        };
        self.write_packet(packet);
    }
    /// Set a player's permission level.
    pub fn set_permission_level(&self, level: u8) {
        self.player.borrow_mut().permission_level = level;
    }
    /// Get a player's permission level.
    pub fn get_permission_level(&self) -> u8 {
        self.player.borrow().permission_level
    }
    /// Remove this player from the server.
    pub fn remove(&self, extra: Option<String>) {
        self.player.borrow_mut().remove(extra);
    }
    /// Create a new player.
    pub fn new(player: RefCell<Player>) -> Self {
        Self {
            player: Arc::new(player),
        }
    }
    /// Open a window.
    pub fn open_window(&self, window: Window, id: i8) {
        self.player
            .borrow_mut()
            .open_inventories
            .insert(id, window.clone());
        self.write_packet(ServerPacket::OpenWindow {
            window_id: id,
            inventory_type: window.inventory_type,
            window_title: window.window_title,
            num_slots: window.inventory.borrow().items.len() as i8 - 1,
        });
    }
    /// Get this player's open inventories.
    pub fn get_open_inventories(&self) -> HashMap<i8, Window> {
        self.player.borrow().open_inventories.clone()
    }
    /// Close an open inventory.
    pub fn close_window(&self, id: i8) {
        self.player.borrow_mut().open_inventories.remove(&id);
    }
    /// Send this player a chat message.
    pub fn send_message(&self, message: Message) {
        //log::info!("[{} CHAT] {}", self.get_username(), message.message);
        if CONFIGURATION.experimental.async_chat {
            let us = self.player.borrow_mut();
            if let Err(e) = us.async_chat.send(AsyncChatCommand::ChatToUser {
                name: us.username.clone(),
                message: message,
            }) {
                log::error!("Error sending async chat message: {:?}", e);
            }
        } else {
            self.player.borrow_mut().chatbox.push(message);
        }
    }
    /// Write a packet to this player.
    pub fn write_packet(&self, packet: ServerPacket) {
        self.player.borrow_mut().write(packet);
    }
    /// Check if this player is crouching.
    pub fn is_crouching(&self) -> bool {
        self.player.borrow().crouching.clone()
    }
    /// Get the item in the player's hand.
    pub fn get_item_in_hand(&self) -> RefMut<'_, ItemStack> {
        RefMut::map(self.player.borrow_mut(), |plr| {
            plr.get_item_in_hand_mut().unwrap()
        })
    }
    /// Clear the item in the player's in-inventory cursor.
    pub fn clear_cursor(&self) {
        self.write_packet(ServerPacket::SetSlot {
            window_id: -1,
            slot: -1,
            item_id: -1,
            item_count: None,
            item_uses: None,
        });
    }
    /// Get a clone of the item in the player's hand.
    pub fn get_item_in_hand_clone(&self) -> ItemStack {
        self.player.borrow().get_item_in_hand_ref().clone()
    }
    /// Get a clone of the player's position.
    pub fn get_position_clone(&self) -> Position {
        self.player.borrow().position.clone()
    }
    /// Get a clone of the player's last position.
    pub fn get_last_position_clone(&self) -> Position {
        self.player.borrow().last_position.clone()
    }
    /// Get the ID of the world the player is in.
    pub fn get_world(&self) -> i8 {
        self.player.borrow().world.clone()
    }
    /// Get the player's username.
    pub fn get_username(&self) -> String {
        self.player.borrow().username.clone()
    }
    /// Set if the held item was changed.
    pub fn held_item_changed(&self, state: bool) {
        self.player.borrow_mut().held_item_changed = state;
    }
    /// Change one of the held slots of the player.
    pub fn set_held_slot(&self, idx: i16) {
        self.player.borrow_mut().held_item_changed = true;
        self.player.borrow_mut().held_slot = idx;
    }
    /// Get checking fall
    pub fn get_checking_fall(&self) -> bool {
        self.player.borrow().checking_fall.clone()
    }
    /// Set checking fall
    pub fn set_checking_fall(&self, value: bool) {
        self.player.borrow_mut().checking_fall = value;
    }
    /// Get the player's entity ID.
    pub fn get_id(&self) -> EntityID {
        self.player.borrow().id.clone()
    }
    /// Disconnect the player,
    pub fn disconnect(&self, reason: String) {
        self.player.borrow_mut().disconnect(reason);
    }
    /// Get the player's health.
    pub fn get_health(&self) -> i16 {
        self.player.borrow().health.clone()
    }
    /// Set the player's offground height.
    pub fn set_offground_height(&self, height: f32) {
        self.player.borrow_mut().offground_height = height;
    }
    /// Damage the player.
    pub fn damage(&self, damage_type: DamageType, amount: i16, damagee: Option<&mut Player>) {
        if self.is_dead() {
            return;
        }
        self.set_health(self.get_health() - amount);
        let id = self.get_id().0;
        self.write_packet(ServerPacket::Animation {
            eid: id,
            animate: 2,
        });
        self.write_packet(ServerPacket::EntityStatus {
            eid: id,
            entity_status: 2,
        });
        if let Some(plr) = damagee {
            plr.write(ServerPacket::EntityStatus {
                eid: self.get_id().0,
                entity_status: 2,
            });
            plr.write(ServerPacket::Animation {
                eid: self.get_id().0,
                animate: 2,
            });
        }
        let mut us = self.unwrap().unwrap();
        for (_, player) in us.players_list.0.borrow().iter() {
            if player.can_borrow() {
                player.write_packet(ServerPacket::EntityStatus {
                    eid: us.id.0,
                    entity_status: 2,
                });
                player.write_packet(ServerPacket::Animation {
                    eid: us.id.0,
                    animate: 2,
                });
            }
        }
        us.last_dmg_type = damage_type;
    }
    /// Set the player's health.
    pub fn set_health(&self, health: i16) {
        let mut us = self.player.borrow_mut();
        us.health = health;
        us.write(ServerPacket::UpdateHealth { health });
    }
    /// Get the player's loaded chunks.
    pub fn get_loaded_chunks(&self) -> Vec<ChunkCoords> {
        self.player.borrow().loaded_chunks.clone()
    }
    /// Notify clients of equipment updates from this player.
    pub fn equipment_check(&self, game: &mut Game) -> anyhow::Result<()> {
        let mut cl = self.player.borrow_mut();
        // Held items check
        let mut item = cl.get_item_in_hand_ref().clone();
        if item.id == 0 {
            item.id = -1;
        }
        cl.held_item_changed = false;
        game.broadcast_to_loaded(
            &cl,
            ServerPacket::EntityEquipment {
                eid: cl.id.0,
                slot: 0,
                item_id: item.id,
                damage: item.damage,
            },
        )?;
        for i in 5..8 {
            let mut item = cl.inventory.items.get(&i).unwrap().clone();
            if item.id == 0 {
                item.id = -1;
            }
            //log::info!("Item id: {}", item.id);
            game.broadcast_to_loaded(
                &cl,
                ServerPacket::EntityEquipment {
                    eid: cl.id.0,
                    slot: (i - 3) as i16,
                    item_id: item.id,
                    damage: item.damage,
                },
            )?;
        }
        Ok(())
    }
    /// Check if the player is dead.
    pub fn is_dead(&self) -> bool {
        self.player.borrow().dead.clone()
    }
    /// Set if the player is dead.
    pub fn set_dead(&self, dead: bool) {
        self.player.borrow_mut().dead = dead;
    }
    /// Get a mutable reference to the player's position.
    pub fn get_position(&self) -> Position {
        self.player.borrow_mut().pos_changed = true;
        self.player.borrow_mut().position
    }
    /// Set the player's position.
    pub fn set_position(&self, pos: Position) {
        self.player.borrow_mut().pos_changed = true;
        self.player.borrow_mut().position = pos;
    }
    /// Set the player's last position.
    pub fn set_last_position(&self, position: Position) {
        self.player.borrow_mut().last_position = position;
    }
    /// Is the player crouching?
    pub fn get_crouching(&self) -> bool {
        self.player.borrow_mut().crouching
    }
    /// Set if the player is crouching. Does not propogate to the player.
    pub fn set_crouching(&self, crouching: bool) {
        self.player.borrow_mut().metadata_changed = true;
        self.player.borrow_mut().crouching = crouching;
    }
    /// Get a mutable reference to the player's inventory.
    pub fn get_inventory(&self) -> RefMut<'_, Inventory> {
        self.player.borrow_mut().inv_changed = true;
        RefMut::map(self.player.borrow_mut(), |plr| &mut plr.inventory)
    }
    /// Check if the player can be borrowed.
    pub fn can_borrow(&self) -> bool {
        self.player.try_borrow().is_ok()
    }
    /// Get a mutable reference to the item in the cursor
    pub fn get_current_cursored_item_mut(&self) -> RefMut<'_, Option<ItemStack>> {
        RefMut::map(self.player.borrow_mut(), |plr| {
            &mut plr.current_cursored_item
        })
    }
    /// Get the internal player object
    pub fn unwrap(&self) -> Result<RefMut<'_, Player>, std::cell::BorrowMutError> {
        self.player.try_borrow_mut()
    }
    /// Sync inventory with the client
    pub fn sync_inventory(&self) {
        let mut player = self.unwrap().unwrap();
        let inv = player.inventory.clone();
        player.write(ServerPacket::InvWindowItems { inventory: inv });
        player.last_inventory = player.inventory.clone();
        player.inv_changed = false;
    }
    /// Check chunks to load/unload
    pub fn check_chunks(&self, game: &mut Game) {
        let mut cl = self.player.borrow_mut();
        // Chunk check
        cl.pos_changed = false;
        let pos = cl.position.clone();
        let mut packets = vec![];
        cl.loaded_chunks.retain(|chunk| {
            if chunk.distance(&ChunkCoords::from_pos(&pos))
                > CONFIGURATION.chunk_distance as f64 * 1.5
            {
                if CONFIGURATION.logging.chunk_unload {
                    log::info!("Unloading chunk at ({}, {})", chunk.x, chunk.z);
                }
                log::debug!("Unloading chunk {}, {}", chunk.x, chunk.z);
                packets.push(ServerPacket::PreChunk {
                    x: chunk.x,
                    z: chunk.z,
                    mode: false,
                });
                return false;
            }
            true
        });
        for packet in packets {
            cl.write(packet);
        }
        for x in -CONFIGURATION.chunk_distance..CONFIGURATION.chunk_distance {
            for z in -CONFIGURATION.chunk_distance..CONFIGURATION.chunk_distance {
                //let coords = ChunkCoords { x: x, z: z };
                let mut coords = ChunkCoords::from_pos(&pos);
                coords.x += x;
                coords.z += z;
                if game.world.check_chunk_exists(&coords)
                /*  && !(x == 0 && z == 0) */
                {
                    if !cl.loaded_chunks.contains(&coords) {
                        cl.loaded_chunks.push(coords);
                        game.loaded_chunks.push(coords);
                        if CONFIGURATION.logging.chunk_load {
                            log::info!(
                                "{} is loading chunk at ({}, {})",
                                cl.username,
                                coords.x,
                                coords.z
                            );
                        }
                        let packets = game
                            .world
                            .chunk_to_packets(coords, cl.packet_send_sender.clone());
                        if packets.is_err() {
                            continue;
                        }
                    }
                } else {
/*                     if let Some(c) = game.world.mcr_helper.get_chunk(ChunkCoords { x: coords.x, z: coords.z }) {
                        game.world.chunks.insert(coords, c);
                    } else {
                        game.world.chunks.insert(
                            coords,
                            game.world.generator.gen_chunk(ChunkCoords {
                                x: coords.x,
                                z: coords.z,
                            }),
                        );
                    } */
                    game.world.init_chunk(&ChunkCoords { x: coords.x, z: coords.z });
                }
            }
        }
        let mut packets = vec![];
        let loaded = cl.loaded_chunks.clone();
        cl.rendered_players.retain(|id, _| {
            if let Some(plr) = game.players.0.borrow().clone().get(&id.0) {
                let otherpos = plr.get_position_clone();
                let c_x = otherpos.x as i32 / 16;
                let c_z = otherpos.z as i32 / 16;
                let c_coords = ChunkCoords { x: c_x, z: c_z };
                if !loaded.contains(&c_coords) {
                    packets.push(ServerPacket::DestroyEntity { eid: id.0 .0 });
                    return false;
                }
            } else {
                packets.push(ServerPacket::DestroyEntity { eid: id.0 .0 });
                return false;
            }
            true
        });
        for packet in packets {
            cl.write(packet);
        }
    }
    /// Get this player's metadata
    pub fn build_metadata(&self) -> Metadata {
        self.unwrap().unwrap().build_metadata()
    }
    /// Is the player underwater?
    pub fn is_underwater(&self, game: &mut Game) -> bool {
        let pos = self.get_position();
        let mut b1 = false;
        let mut b2 = false;
        if let Some(block) = game
            .world
            .get_block(pos.x as i32, (pos.y as i32) + 1, pos.z as i32)
        {
            if let Some(reg_block) = ItemRegistry::global().get_item(block.b_type as i16) {
                if let Some(reg_block) = reg_block.get_item().as_block() {
                    if reg_block.is_fluid() {
                        b1 = true;
                    }
                }
            }
        }
        if let Some(block) = game
            .world
            .get_block(pos.x as i32, (pos.y as i32) + 0, pos.z as i32)
        {
            if let Some(reg_block) = ItemRegistry::global().get_item(block.b_type as i16) {
                if let Some(reg_block) = reg_block.get_item().as_block() {
                    if reg_block.is_fluid() {
                        b2 = true;
                    }
                }
            }
        }
        b1 && b2
    }
    /// Get the player's air level.
    pub fn get_air(&self) -> u16 {
        self.player.borrow().air
    }
    /// Set the player's air level. Does not notify client.
    pub fn set_air(&self, air: u16) {
        self.player.borrow_mut().air = air;
    }
    /// Get the last drown tick.
    pub fn get_last_drown_tick(&self) -> u128 {
        self.player.borrow().last_drown_tick
    }
    /// Set the last drown tick.
    pub fn set_last_drown_tick(&self, tick: u128) {
        self.player.borrow_mut().last_drown_tick = tick;
    }
    /// Tick the player.
    pub fn tick(&self, game: &mut Game) -> anyhow::Result<()> {
        if self.player.borrow().recv_packets_recv.is_disconnected() {
            self.remove(Some(String::from("Disconnected")));
            return Ok(());
        }
        if self.player.borrow().last_keepalive_time + 1200 < game.ticks {
            self.remove(Some(String::from("Timed out")));
            //self.disconnect("Timed out".to_string());
            return Ok(());
        }
        if self.player.borrow().metadata_changed {
            for player in game.players.iter() {
                let player = player.1;
                if player
                    .get_loaded_chunks()
                    .contains(&self.get_position_clone().to_chunk_coords())
                {
                    player.write_packet(ServerPacket::EntityMetadata {
                        eid: self.get_id().0,
                        entity_metadata: self.build_metadata(),
                    });
                }
            }
            self.player.borrow_mut().metadata_changed = false;
        }
        if self.player.borrow().held_item_changed.clone() {
            self.equipment_check(game)?;
        }
        if self.player.borrow().pos_changed.clone() {
            self.check_chunks(game);
        }
        if self.player.borrow().inv_changed.clone() {
            self.sync_inventory();
        }
        let interval = Duration::from_millis(750);
        if self.is_underwater(game) {
            let air = self.get_air();
            if air == 0 {
                let player = self.clone();
                if self.get_last_drown_tick() + 20 < game.ticks {
                    self.damage(DamageType::Drown, 2, None);
                    self.set_last_drown_tick(game.ticks);
                }
            } else {
                self.set_air(air - 1);
            }
        } else {
            self.set_air(300);
        }
        let mut cl = self.player.borrow_mut();
        // Fall damage check
        if cl.position.on_ground {
            if cl.offground_height > (cl.position.y as f32) {
                let gamerule = game.gamerules.rules.get("fall-damage").unwrap();
                if let crate::game::gamerule::GameruleValue::Boolean(value) = gamerule {
                    if *value {
                        //log::info!("Offground: {}, pos: {}", cl.offground_height, cl.position.y);
                        let height = cl.offground_height - (cl.position.y as f32);
                        let mut do_dmg = true;
                        //log::info!("Pos for {}: {:?}", cl.username, cl.position);
                        if let Some(block) = game.world.get_block(
                            cl.position.x as i32,
                            (cl.position.y as i32) + 1,
                            cl.position.z as i32,
                        ) {
                            if let Some(reg_block) =
                                ItemRegistry::global().get_item(block.b_type as i16)
                            {
                                if let Some(reg_block) = reg_block.get_item().as_block() {
                                    if reg_block.is_fluid() {
                                        do_dmg = false;
                                    }
                                }
                            }
                        }
                        if let Some(block) = game.world.get_block(
                            cl.position.x as i32,
                            (cl.position.y as i32) + 0,
                            cl.position.z as i32,
                        ) {
                            if let Some(reg_block) =
                                ItemRegistry::global().get_item(block.b_type as i16)
                            {
                                if let Some(reg_block) = reg_block.get_item().as_block() {
                                    if reg_block.is_fluid() {
                                        do_dmg = false;
                                    }
                                }
                            }
                        }
                        if do_dmg {
                            if height > 0.0 {
                                let fall_dmg = (height - 3.0).max(0.0);
                                //log::info!("Damage: {}", fall_dmg.round());
                                cl.damage(DamageType::Fall, fall_dmg.round() as i16, None);
                                //log::info!("Fell from a height of {}", height);
                            }
                        }
                    }
                    cl.offground_height = 0.0;
                } else {
                    panic!("Fall damage gamerule is not a boolean!");
                }
            }
        }
        // Void dmg check
        if cl.position.y <= 0.0 && !cl.dead {
            if cl.last_void_dmg + interval < Instant::now() {
                cl.damage(DamageType::Void, 3, None);
                /*                 cl.health -= 3; */
                cl.last_void_dmg = Instant::now();
            }
        }
        // Death check
        if cl.health <= 0 && !cl.dead {
            let mut msg = Message::new(&format!("{} died.", cl.username));
            match &cl.last_dmg_type {
                DamageType::None => {}
                DamageType::Void => {
                    msg = Message::new(&format!("{} fell into the abyss.", cl.username));
                }
                DamageType::Player { damager } => {
                    msg = Message::new(&format!(
                        "{} was beaten to death by {}.",
                        cl.username, damager
                    ));
                }
                DamageType::Fall => {
                    msg = Message::new(&format!("{} fell out of the sky.", cl.username));
                }
                DamageType::Mob { damager } => {
                    msg = Message::new(&format!("{} got slain by {}.", cl.username, damager));
                }
                DamageType::Drown => {
                    msg = Message::new(&format!("{} drowned.", cl.username));
                }
            }
            let id = cl.id.0;
            game.broadcast_to_loaded(
                &cl,
                ServerPacket::EntityStatus {
                    eid: id,
                    entity_status: 3,
                },
            )?;
            game.broadcast_message(msg.clone())?;
            let pos = cl.position.clone();
            cl.inventory.items.retain(|_, item| {
                if item.id != 0 {
                    game.spawn_entity(Box::new(
                        crate::game::entities::item_entity::ItemEntity::new(
                            Position::from_pos(pos.x as f64, (pos.y as f64) + 1.0, pos.z as f64),
                            game.ticks,
                            item.clone(),
                            None,
                        ),
                    ));
                }
                item.id = 0;
                item.count = 0;
                item.damage = 0;
                true
            });
            cl.chatbox.push(msg);
            //println!("Yo!");
            cl.write(ServerPacket::UpdateHealth { health: 0 });
            cl.dead = true;
        }
        // Inventory check
        let len = cl.inventory.items.len();
        for i in 0..len {
            let item = cl.inventory.items.get_mut(&(i as i8)).unwrap();
            if item.count == 0 {
                item.id = 0;
                item.damage = 0;
            }
        }
        let inv = cl.inventory.clone();
        cl.write(ServerPacket::InvWindowItems { inventory: inv });
        // Chat messages
        cl.chatbox.clone().messages.retain(|message| {
            cl.write(ServerPacket::ChatMessage {
                message: message.message.clone(),
            });
            false
        });
        cl.chatbox = crate::game::Chatbox::default();
        // Spawn players check
        //log::info!("Running spawn players tick");
        let plrlist = cl.players_list.0.borrow().clone();
        for player in plrlist.iter() {
            if player.0 != &cl.id {
                let other_username = player.1.get_username();
                if !cl
                    .loaded_chunks
                    .contains(&player.1.get_position().to_chunk_coords())
                {
                    //log::info!("{} not in {} loaded chunks", other_username, cl.username);
                    continue;
                }
                if cl
                    .rendered_players
                    .get(&(*player.0, other_username.clone()))
                    .is_none()
                /* && player.1.get_loaded_chunks().contains(&ChunkCoords::from_pos(&cl.position)) */
                {
                    cl.rendered_players.insert(
                        (*player.0, other_username),
                        crate::game::RenderedPlayerInfo {
                            position: player.1.get_position_clone(),
                            held_item: player.1.get_item_in_hand_clone(),
                        },
                    );
                    let pos = player.1.get_position_clone();
                    cl.write(ServerPacket::NamedEntitySpawn {
                        eid: player.1.get_id().0,
                        name: player.1.get_username(),
                        x: (pos.x * 32.0) as i32,
                        y: (pos.y * 32.0) as i32,
                        z: (pos.z * 32.0) as i32,
                        rotation: 0,
                        pitch: 0,
                        current_item: 0,
                    });
                    cl.write(ServerPacket::EntityTeleport {
                        eid: player.1.get_id().0,
                        x: (pos.x * 32.0) as i32,
                        y: (pos.y * 32.0) as i32,
                        z: (pos.z * 32.0) as i32,
                        yaw: pos.yaw as i8,
                        pitch: pos.pitch as i8,
                    });
                    cl.write(ServerPacket::EntityMetadata {
                        eid: player.1.get_id().0,
                        entity_metadata: player.1.build_metadata(),
                    });
                    if player.1.is_dead() {
                        cl.write(ServerPacket::EntityStatus {
                            eid: player.1.get_id().0,
                            entity_status: 3,
                        });
                    }
                } else {
                    //log::info!("{} already rendering {}", cl.username, other_username);
                }
            }
        }
        // Player positions check
        let mut packets = Vec::new();
        let name = cl.username.clone();
        for id in cl.rendered_players.iter_mut() {
            let pos = if let Some(plr) = game.players.0.borrow().get(&id.0 .0) {
                plr.get_position_clone()
            } else {
                log::info!("Skipping player");
                continue;
            };
            if id.1.position != pos {
                if pos.distance(&id.1.position) < 2. && true == false {
                    let x_diff = (pos.x - id.1.position.x);
                    let y_diff = (pos.y - id.1.position.y);
                    let z_diff = (pos.z - id.1.position.z);
                    packets.push(ServerPacket::EntityLookAndRelativeMove {
                        eid: id.0 .0 .0,
                        dX: (x_diff * 32.0) as i8,
                        dY: (y_diff * 32.0) as i8,
                        dZ: (z_diff * 32.0) as i8,
                        yaw: pos.yaw as i8,
                        pitch: pos.pitch as i8,
                    });
                    //log::info!("Sending relative");
                } else {
                    //log::info!("Sending packet to {}", name);
                    //log::info!("Sending absolute");
                    let packet = ServerPacket::EntityTeleport {
                        eid: id.0 .0 .0,
                        x: (pos.x * 32.0) as i32,
                        y: (pos.y * 32.0) as i32,
                        z: (pos.z * 32.0) as i32,
                        yaw: pos.yaw as i8,
                        pitch: pos.pitch as i8,
                    };
                    //log::info!("Sending packet {:?} to {}", packet, name);
                    packets.push(packet);
                }
                //log::info!("Sending entity teleport!");
                //packets.push(ServerPacket::EntityLook { eid: id.0.0.0, yaw: pos.yaw as i8, pitch: pos.pitch as i8 });
            } else {
                id.1.position = pos;
                //log::info!("Not for {}", name);
            }
            id.1.position = pos;
            //log::info!("tping {} from {:?} to {:?}", id.0.0, player.id, pos);
        }
        for packet in packets {
            cl.write(packet);
        }
        // Cull entities check
        let mut ids = Vec::new();
        cl.rendered_entities.retain(|id, _| {
            for (_, entity_game) in game.entities.borrow_mut().iter_mut() {
                let entity_game = entity_game.borrow();
                if id == &entity_game.get_id() {
                    return true;
                }
            }
            ids.push(id.clone());
            false
        });
        for id in ids {
            cl.write(ServerPacket::DestroyEntity { eid: id.0 });
            //game.entities.borrow().get(&id).unwrap().borrow_mut().destruct_entity(&mut cl);
        }
        // Cull spawned entities check
        let loaded = cl.loaded_chunks.clone();
        let mut ids = Vec::new();
        cl.rendered_entities.retain(|id, _| {
            if let Some(entity) = game.entities.borrow_mut().get_mut(id) {
                if loaded.contains(&entity.borrow_mut().get_position().to_chunk_coords()) {
                    return true;
                }
            }
            ids.push(id.clone());
            false
        });
        for id in ids {
            game.entities
                .borrow()
                .get(&id)
                .unwrap()
                .borrow_mut()
                .destruct_entity(&mut cl);
        }
        // Spawn entities check
        //log::info!("Running spawn players tick");
        let mut entitylist = game.entities.borrow_mut();
        for (_, entity) in entitylist.iter_mut() {
            let mut entity = entity.borrow_mut();
            if entity.get_id() != cl.id {
                //let other_username = player.1.get_username();
                if !cl
                    .loaded_chunks
                    .contains(&entity.get_position().clone().to_chunk_coords())
                {
                    continue;
                }
                if cl.rendered_entities.get(&entity.get_id()).is_none()
                /* && player.1.get_loaded_chunks().contains(&ChunkCoords::from_pos(&cl.position)) */
                {
                    cl.rendered_entities.insert(
                        entity.get_id(),
                        crate::game::RenderedEntityInfo {
                            position: entity.get_position().clone(),
                        },
                    );
                    log::debug!("Spawning entity!");
                    entity.spawn_entity(&mut cl);
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone)]
pub struct Window {
    pub inventory: Arc<RefCell<Inventory>>,
    pub inventory_type: i8,
    pub window_title: String,
}
use std::net::*;
pub struct Player {
    pub username: String,
    pub id: EntityID,
    pub position: Position,
    pub last_position: Position,
    pub recv_packets_recv: Receiver<ClientPacket>,
    pub packet_send_sender: Sender<ServerPacket>,
    pub rendered_players: HashMap<(EntityID, String), RenderedPlayerInfo>,
    pub rendered_entities: HashMap<EntityID, RenderedEntityInfo>,
    pub open_inventories: HashMap<i8, Window>,
    pub metadata_changed: bool,
    pub chatbox: Chatbox,
    pub perm_level: usize,
    pub crouching: bool,
    pub last_health: i16,
    pub health: i16,
    pub dead: bool,
    pub world: i8,
    pub last_void_dmg: std::time::Instant,
    pub inventory: Inventory,
    pub last_inventory: Inventory,
    pub held_slot: i16,
    pub last_dmg_type: DamageType,
    pub last_transaction_id: i16,
    pub current_cursored_item: Option<ItemStack>,
    pub loaded_chunks: Vec<ChunkCoords>,
    pub has_loaded_before: Vec<ChunkCoords>,
    pub since_last_attack: std::time::Instant,
    pub mining_block: MiningBlockData,
    pub held_item_changed: bool,
    pub inv_changed: bool,
    pub pos_changed: bool,
    pub offground_height: f32,
    pub checking_fall: bool,
    pub socket_addr: SocketAddr,
    pub all_player_data: Arc<RefCell<HashMap<String, PersistentPlayerData>>>,
    pub permission_level: u8,
    pub air: u16,
    pub last_drown_tick: u128,
    pub async_chat: Sender<AsyncChatCommand>,
    pub last_keepalive_time: u128,
    players_list: PlayerList,
}
impl Player {
    pub fn build_metadata(&self) -> Metadata {
        let mut metadata = Metadata::new();
        let mut meta_val = 0;
        match self.crouching {
            true => meta_val |= 0x02,
            false => {}
        };
        metadata.insert_byte(meta_val);
        metadata
    }
    pub fn save_to_mem(&mut self) {
        //log::info!("Saving playerdata");
        let name = self.username.clone();
        let mut data = self.all_player_data.borrow_mut();
        let data = data.get_mut(&name).expect("This should not be possible!");
        data.position = self.position.clone();
        data.health = self.health;
        data.inventory = self.inventory.clone();
    }
    pub fn sync_position(&mut self) {
        let pos = self.position;
        let id = self.id;
        let packet = ServerPacket::EntityTeleport {
            eid: id.0,
            x: (pos.x * 32.0) as i32,
            y: (pos.y * 32.0) as i32,
            z: (pos.z * 32.0) as i32,
            yaw: pos.yaw as i8,
            pitch: pos.pitch as i8,
        };
        self.write(packet);
    }
    pub fn sync_inventory(&mut self) {
        let mut player = self;
        let inv = player.inventory.clone();
        //log::info!("Writing inv items");
        player.write(ServerPacket::InvWindowItems { inventory: inv });
        player.last_inventory = player.inventory.clone();
        player.inv_changed = false;
    }
    pub fn add_velocity(&mut self, x: i16, y: i16, z: i16) {
        self.write(ServerPacket::EntityVelocity {
            eid: self.id.0,
            velocity_x: x,
            velocity_y: y,
            velocity_z: z,
        });
    }
    pub fn disconnect(&mut self, reason: String) {
        self.write(ServerPacket::Disconnect { reason });
        self.remove(None);
    }
    pub fn remove(&mut self, extra: Option<String>) {
        self.save_to_mem();
        self.players_list.0.borrow_mut().remove(&self.id);
        IDS.lock().unwrap().push(self.id.0);
        if let Some(extra) = extra {
            log::info!(
                "{}[/{}] lost connection: {}",
                self.username,
                self.socket_addr,
                extra
            );
        }
        log::info!("§e{} left the game.", self.username);
        for player in self.players_list.0.borrow().iter() {
            if player.1.can_borrow() {
                player
                    .1
                    .clone()
                    .send_message(Message::new(&format!("§e{} left the game.", self.username)));
            }
            /*             if let Ok(mut plr) = player.1.try_borrow_mut() {
            } else {
                continue;
            } */
        }
    }

    pub fn damage(&mut self, damage_type: DamageType, amount: i16, damagee: Option<&mut Player>) {
        if amount == 0 {
            return;
        }
        if self.dead {
            return;
        }
        self.health -= amount;
        let id = self.id.0;
        self.write(ServerPacket::Animation {
            eid: id,
            animate: 2,
        });
        self.write(ServerPacket::EntityStatus {
            eid: id,
            entity_status: 2,
        });
        self.write(ServerPacket::UpdateHealth {
            health: self.health,
        });
        if let Some(plr) = damagee {
            plr.write(ServerPacket::EntityStatus {
                eid: self.id.0,
                entity_status: 2,
            });
            plr.write(ServerPacket::Animation {
                eid: self.id.0,
                animate: 2,
            });
        }
        for (_, player) in &*self.players_list.0.borrow() {
            /*             let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            }; */
            if player.can_borrow() {
                player.write_packet(ServerPacket::EntityStatus {
                    eid: self.id.0,
                    entity_status: 2,
                });
                player.write_packet(ServerPacket::Animation {
                    eid: self.id.0,
                    animate: 2,
                });
            }
        }
        self.last_dmg_type = damage_type;
    }
    pub fn get_item_in_hand_mut(&mut self) -> Option<&mut ItemStack> {
        //log::info!("Checking slot {}", self.held_slot + 36);
        self.inventory.items.get_mut(&((self.held_slot + 36) as i8))
    }
    pub fn get_item_in_hand_ref(&self) -> &ItemStack {
        //log::info!("Checking slot {}", self.held_slot + 36);
        self.inventory
            .items
            .get(&((self.held_slot + 36) as i8))
            .unwrap()
    }
    pub fn get_item_in_hand(&mut self) -> &mut ItemStack {
        //log::info!("Checking slot {}", self.held_slot + 36);
        self.inventory
            .items
            .get_mut(&((self.held_slot + 36) as i8))
            .unwrap()
    }
    pub fn write(&mut self, packet: ServerPacket) {
        if let Err(e) = self.packet_send_sender.send(packet) {
            self.remove(Some(format!("error while writing packet: {:?}", e)));
            /*             self.players_list.0.borrow_mut().remove(&self.id);
            //clients.remove(&id);
            IDS.lock().unwrap().push(self.id.0); */
        }
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
}
impl CommandExecutor for Arc<PlayerRef> {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
    fn send_message(&mut self, message: Message) {
        PlayerRef::send_message(self, message);
    }
    fn permission_level(&self) -> u8 {
        PlayerRef::get_permission_level(self)
    }
    fn username(&self) -> String {
        self.get_username()
    }
}
#[derive(Clone, Debug)]
pub struct Message {
    pub message: String,
}
impl std::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}
impl Message {
    pub fn new(msg: &str) -> Self {
        Self {
            message: msg.to_string(),
        }
    }
}
#[derive(Clone)]
pub struct PlayerList(pub Arc<RefCell<HashMap<EntityID, Arc<PlayerRef>>>>);
impl PlayerList {
    pub fn get_player(&self, name: &str) -> Option<Arc<PlayerRef>> {
        for player in self.0.borrow().iter() {
            if !player.1.can_borrow() {
                continue;
            }
            if player.1.get_username().to_lowercase() == name.to_lowercase() {
                return Some(player.1.clone());
            }
        }
        None
    }
    pub fn iter(&self) -> impl Iterator<Item = (EntityID, Arc<PlayerRef>)> {
        let list = self.0.borrow().clone();
        list.into_iter()
    }
}
pub struct LoadedChunks(pub HashMap<ChunkCoords, u128>);
impl LoadedChunks {
    pub fn push(&mut self, coords: ChunkCoords) {
        if !self.0.contains_key(&coords) {
            self.0.insert(coords, 0);
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = (&ChunkCoords, &u128)> {
        self.0.iter()
    }
    pub fn contains(&self, coords: &ChunkCoords) -> bool {
        self.0.contains_key(coords)
    }
}
#[derive(PartialEq, Clone)]
pub struct PersistentPlayerData {
    pub username: String,
    pub position: Position,
    pub health: i16,
    pub inventory: Inventory,
}
use tile_entity::*;
impl Eq for PersistentPlayerData {}
pub struct Scheduler {
    tasks: Vec<(u128, Arc<Box<dyn Fn(&mut Game) -> Option<u128>>>)>,
}
impl Scheduler {
    pub fn new() -> Self {
        Self { tasks: Vec::new() }
    }
    pub fn schedule_task(
        &mut self,
        ticks: u128,
        func: Arc<Box<dyn Fn(&mut Game) -> Option<u128>>>,
    ) {
        self.tasks.push((ticks, func));
    }
    pub fn run_tasks(&mut self, game: &mut Game) {
        let ticks = game.ticks;
        let mut to_reschedule = Vec::new();
        self.tasks.retain(|task| {
            if task.0 == ticks {
                if let Some(time) = task.1(game) {
                    to_reschedule.push((time, task.1.clone()));
                }
                return false;
            }
            true
        });
        self.tasks.append(&mut to_reschedule);
    }
}
use crate::async_systems::chat::*;
use crate::async_systems::AsyncGameCommand;
//use plugins::*;
#[derive(Clone, Debug)]
pub struct CachedCommandData {
    args: Vec<CommandArgumentTypes>,
    root: String,
    desc: String,
}
pub struct Game {
    pub objects: Arc<Objects>,
    pub players: PlayerList,
    pub entities: Arc<RefCell<HashMap<EntityID, Arc<RefCell<Box<dyn Entity>>>>>>,
    pub tile_entities:
        Arc<RefCell<HashMap<BlockPosition, Arc<RefCell<Box<dyn tile_entity::BlockTileEntity>>>>>>,
    pub systems: Arc<RefCell<Systems>>,
    pub world: crate::world::chunks::World,
    pub block_updates: Vec<Block>,
    pub command_system: Arc<RefCell<CommandSystem>>,
    pub loaded_chunks: LoadedChunks,
    pub ticks: u128,
    pub persistent_player_data: Arc<RefCell<HashMap<String, PersistentPlayerData>>>,
    pub gamerules: gamerule::Gamerules,
    pub tps: f64,
    pub async_commands: Receiver<AsyncGameCommand>,
    pub async_chat_manager: Sender<AsyncChatCommand>,
    pub perm_level_map: HashMap<String, u8>,
    pub cached_command_list: Vec<CachedCommandData>,
    pub rain_ticks: u128,
    pub is_raining: bool,
    pub is_storming: bool,
    pub world_saving: bool,
}
use nbt::*;
use rand::Rng;
impl Game {
    pub fn check_world_save(&mut self) {
        if self.world_saving {
            if GAME_GLOBAL.get_time() % CONFIGURATION.autosave_interval == 0 {
                self.op_status_message("CONSOLE", "Auto-saving the world..");
                if let Err(e) = self.save_playerdata() {
                    log::info!("Error saving playerdata: {:?}", e);
                }
                if let Err(e) = self.world.to_file(&CONFIGURATION.level_name) {
                    log::info!("Error saving the world: {:?}", e);
                }
                self.op_status_message("CONSOLE", "Auto-save complete.");
            }
        }
    }
    pub fn strike_lightning(&mut self, pos: BlockPosition) {
        let id = EntityID::new();
        self.packet_to_chunk(
            pos.to_chunk_coords(),
            ServerPacket::Thunderbolt {
                eid: id.0,
                unknown: true,
                x: pos.x,
                y: pos.y,
                z: pos.z,
            },
        );
        IDS.lock().unwrap().push(id.0);
    }
    pub fn packet_to_chunk(&mut self, chunk: ChunkCoords, packet: ServerPacket) {
        for player in self.players.iter() {
            if player.1.get_loaded_chunks().contains(&chunk) {
                player.1.write_packet(packet.clone());
            }
        }
    }
    pub fn check_rain(&mut self) {
        if self.rain_ticks > 0 {
            self.rain_ticks -= 1;
            if !self.is_raining {
                self.is_raining = true;
                for player in self.players.iter() {
                    player
                        .1
                        .write_packet(ServerPacket::NewInvalidState { reason: 1 });
                }
            }
        } else if self.is_raining {
            self.is_raining = false;
            self.is_storming = false;
            for player in self.players.iter() {
                player
                    .1
                    .write_packet(ServerPacket::NewInvalidState { reason: 2 });
            }
        }
    }
    pub fn stop_server(&mut self) {
        self.save_playerdata().unwrap();
        let plrs = self.players.0.borrow().clone();
        for player in plrs.iter() {
            player.1.disconnect("Server closed".to_string());
        }
        self.world.to_file(&CONFIGURATION.level_name);
        std::process::exit(0);
    }
    pub fn op_status_message(&mut self, username: &str, message: &str) {
        let epic_msg = format!("({}: {})", username, message);
        log::info!("{}", epic_msg);
        let epic_msg = format!("§7{}", epic_msg);
        for (_, player) in self.players.iter() {
            if player.get_permission_level() >= 4 {
                let mut player = player;
                player.send_message(Message::new(&epic_msg));
            }
        }
    }
    pub fn remove_op(&mut self, player: &str) {
        if let Some(plr) = self.perm_level_map.get_mut(player) {
            *plr = 1;
        }
        crate::configuration::remove_op(player);
        if let Some(mut player) = self.players.get_player(player) {
            player.send_message(Message::new("§eYou are no longer OP!"));
            player.set_permission_level(1);
        }
    }
    pub fn add_op(&mut self, player: &str) {
        if let Some(plr) = self.perm_level_map.get_mut(player) {
            *plr = 4;
        } else {
            self.perm_level_map.insert(player.to_string(), 4);
        }
        if let Some(mut player) = self.players.get_player(player) {
            player.send_message(Message::new("§eYou are now OP!"));
            player.set_permission_level(4);
        }
        crate::configuration::add_op(player);
    }
    pub fn handle_async_commands(&mut self) {
        let mut to_execute = Vec::new();
        for command in self.async_commands.try_iter() {
            match command {
                AsyncGameCommand::ScheduleSyncTask { func } => {
                    to_execute.push(func);
                }
            }
        }
        for func in to_execute {
            func(self);
        }
    }
    pub fn broadcast_player_loaded_entity(&mut self, id: EntityID, packet: ServerPacket) {
        for player in self.players.0.borrow().clone().iter() {
            if player
                .1
                .unwrap()
                .unwrap()
                .rendered_entities
                .get(&id)
                .is_some()
            {
                player.1.write_packet(packet.clone());
            }
        }
    }
    pub fn tile_entity_ticks(&mut self) {
        let tiles = self.tile_entities.borrow().clone();
        for (pos, entity) in tiles.iter() {
            if self.loaded_chunks.contains(&pos.to_chunk_coords()) {
                entity.borrow_mut().tick(self, *pos);
            }
        }
    }
    pub fn random_ticks(&mut self) {
        let random_tick_speed: i32;
        let gamerule = self.gamerules.rules.get("random-tick-speed").unwrap();
        if let crate::game::gamerule::GameruleValue::Int(value) = gamerule {
            random_tick_speed = *value;
        } else {
            panic!("Random tick speed gamerule is not an int!");
        }
        for chunk in self.loaded_chunks.0.clone().iter() {
            let mut rng = rand::thread_rng();
            let chunk = self
                .world
                .chunks
                .get_mut(chunk.0)
                .expect("should be impossible!");
            let mut blocks = Vec::new();
            for section in 0..8 {
                for _ in 0..random_tick_speed {
                    let mut pos = (
                        rng.gen_range(0..16),
                        rng.gen_range(section * 16..16 + section * 16),
                        rng.gen_range(0..16),
                    );
                    let blk = chunk
                        .get_block(pos.0, pos.1, pos.2)
                        .expect("Should not error!")
                        .clone();
                    pos.0 += chunk.x * 16;
                    pos.2 += chunk.z * 16;
                    blocks.push((blk, pos));
                }
            }
            //log::info!("Ticking {} blocks", blocks.len());
            for block in blocks {
                if let Some(reg_block) = ItemRegistry::global().get_item(block.0.b_type as i16) {
                    if let Some(reg_block) = reg_block.get_item().as_block() {
                        //log::info!("Ticking {:?}", block.1);
                        reg_block.random_tick(
                            self,
                            BlockPosition {
                                x: block.1 .0,
                                y: block.1 .1,
                                z: block.1 .2,
                            },
                        );
                    }
                }
            }
        }
    }
    pub fn inv_to_tag(inv: &Inventory) -> CompoundTag {
        let mut inventory_tag = CompoundTag::new();
        let mut items = Vec::new();
        for (slot, item) in inv.items.iter() {
            let mut item_tag = CompoundTag::new();
            item_tag.insert_i8("Slot", *slot);
            item_tag.insert_i16("Item", item.id);
            item_tag.insert_i16("Damage", item.damage);
            item_tag.insert_i8("Count", item.count);
            items.push(item_tag);
        }
        inventory_tag.insert_compound_tag_vec("Items", items);
        inventory_tag
    }
    pub fn tag_to_inv(tag: &CompoundTag) -> Inventory {
        let mut inventory = Inventory::default();
        let items = tag.get_compound_tag_vec("Items").unwrap();
        for tag in items {
            let slot = tag.get_i8("Slot").unwrap();
            let item = tag.get_i16("Item").unwrap();
            let damage = tag.get_i16("Damage").unwrap();
            let count = tag.get_i8("Count").unwrap();
            inventory.items.insert(
                slot.clone(),
                ItemStack::new(item.clone(), damage.clone(), count.clone()),
            );
        }
        inventory
    }
    pub fn load_playerdata() -> anyhow::Result<HashMap<String, PersistentPlayerData>> {
        let mut faxvec: Vec<std::path::PathBuf> = Vec::new();
        for element in
            std::path::Path::new(&format!("{}/playerdata/", CONFIGURATION.level_name)).read_dir()?
        {
            let path = element.unwrap().path();
            if let Some(extension) = path.extension() {
                if extension == "nbt" {
                    faxvec.push(path);
                }
            }
        }
        let mut playerdata = HashMap::new();
        for path in faxvec {
            use nbt::decode::read_gzip_compound_tag;
            use nbt::CompoundTag;
            let mut file = std::fs::File::open(path)?;
            let root = read_gzip_compound_tag(&mut file)?;
            let position = root.get_compound_tag("Position").ok().unwrap().clone();
            let name = root.get_str("Username").ok().unwrap();
            let mut pos = Position {
                x: position.get_f64("x").ok().unwrap().clone(),
                y: position.get_f64("y").ok().unwrap().clone(),
                z: position.get_f64("z").ok().unwrap().clone(),
                yaw: position.get_f32("yaw").ok().unwrap().clone(),
                pitch: position.get_f32("pitch").ok().unwrap().clone(),
                ..Position::default()
            };
            playerdata.insert(
                name.to_string(),
                PersistentPlayerData {
                    username: name.to_string(),
                    position: pos,
                    health: root.get_i16("health").ok().unwrap().clone(),
                    inventory: Game::tag_to_inv(&root.get_compound_tag("Inventory").unwrap()),
                },
            );
        }
        Ok(playerdata)
    }
    pub fn save_playerdata(&mut self) -> anyhow::Result<()> {
        for player in self.players.0.borrow().iter() {
            let player = player.1;
            player.unwrap().unwrap().save_to_mem();
            /*             let name = player.get_username();
            let mut data = self.persistent_player_data.borrow_mut();
            let data = data.get_mut(&name).expect("This should not be possible!");
            data.position = player.get_position_clone(); */
        }
        use nbt::encode::write_gzip_compound_tag;
        use nbt::CompoundTag;
        std::fs::create_dir_all(format!("{}/playerdata", CONFIGURATION.level_name))?;
        for (name, player) in self.persistent_player_data.borrow().iter() {
            let mut root = CompoundTag::new();
            let mut position_tag = CompoundTag::new();
            position_tag.insert_f64("x", player.position.x);
            position_tag.insert_f64("y", player.position.y);
            position_tag.insert_f64("z", player.position.z);
            position_tag.insert_f32("yaw", player.position.yaw);
            position_tag.insert_f32("pitch", player.position.pitch);
            root.insert_compound_tag("Position", position_tag);
            root.insert_str("Username", name);
            root.insert_i16("health", player.health);
            root.insert_compound_tag("Inventory", Game::inv_to_tag(&player.inventory));
            let mut file = std::fs::File::create(format!(
                "{}/playerdata/{}.nbt",
                CONFIGURATION.level_name, name
            ))?;
            write_gzip_compound_tag(&mut file, &root)?;
        }
        Ok(())
    }
    pub fn spawn_entity(&mut self, entity: Box<dyn Entity>) {
        self.entities
            .borrow_mut()
            .insert(entity.get_id(), Arc::new(RefCell::new(entity)));
    }
    pub fn new(
        systems: Systems,
        recv: Receiver<AsyncGameCommand>,
        async_chat_manager: Sender<AsyncChatCommand>,
    ) -> Self {
        let mut game_globals = GameGlobals { time: 0 };
        GAME_GLOBAL.set(game_globals);
        let mut registry = ItemRegistry::new();
        let mut event_handler = EventHandler::new();
        event_handler.register_handler(Box::new(|event, game| {
            let event = event.event.as_any().downcast_ref::<BlockPlacementEvent>();
            if event.is_none() {
                return false;
            }
            let event = event.unwrap();
            //log::info!("Got event!");
            let mut packet = event.packet.clone();
            let mut player = event.player.unwrap().unwrap();
            //packet.y -= 1;
            match packet.direction {
                0 => {
                    packet.y -= 1;
                }
                1 => {
                    //packet.y += 1;
                    packet.y = match packet.y.checked_add(1) {
                        Some(num) => num,
                        None => {
                            return false;
                        }
                    }
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
                x => {
                    log::debug!("Fal {}", x);
                    return false;
                }
            }
            // BLOCKS stuff
            let block = if let Some(blk) =
                game.world
                    .get_block_mut(packet.x, (packet.y + 0) as i32, packet.z)
            {
                blk
            } else {
                //log::info!("false.");
                return false;
            };

            if let Some(blk) = ItemRegistry::global().get_item(block.b_type as i16) {
                if let Some(blk) = blk.get_item().as_block() {
                    if !blk.is_solid() {
                        if event.needs_align {
                            block.b_metadata = 1;
                        }
                        block.set_type(event.packet.block_or_item_id as u8);
                        game.block_updates.push(crate::game::Block {
                            position: crate::game::BlockPosition {
                                x: packet.x,
                                y: (packet.y + 0) as i32,
                                z: packet.z,
                            },
                            block: block.clone(),
                        });
                    } else {
                        player.write(ServerPacket::BlockChange {
                            x: packet.x,
                            y: packet.y + 0,
                            z: packet.z,
                            block_type: block.get_type() as i8,
                            block_metadata: block.b_metadata as i8,
                        })
                    }
                }
            }
            true
        }));
        use rand::RngCore;
        items::default::init_items(&mut registry);
        crate::game::entities::tile_entity::init_items(&mut registry);
        ITEM_REGISTRY
            .set(registry)
            .ok()
            .expect("Can't set item registry!");
        //let generator = crate::temp_chunks::FlatWorldGenerator::new(64, 1,1, 1);
        use crate::world::chunks::*;
        let mut world: crate::world::chunks::World;
        if let Ok(w) = crate::world::chunks::World::from_file_mcr(&CONFIGURATION.level_name) {
            log::info!("LOading world!");
            world = w;
        } else {
            /*match CONFIGURATION.chunk_generator.as_str() {
                "noise" => {
                    let mut seed = rand::thread_rng().next_u64();
                    if let Some(s) = CONFIGURATION.world_seed {
                        seed = s as u64;
                    }
                    log::info!(
                        "Initializing world with chunk generator \"noise\" with seed ({})",
                        seed
                    );
                    Box::new(FunnyChunkGenerator::new(seed, FunnyChunkPreset::MOUNTAIN))
                        as Box<dyn ChunkGenerator>
                }
                "flat" => {
                    log::info!("Initializing world with chunk generator \"flat\"");
                    Box::new(FlatChunkGenerator {}) as Box<dyn ChunkGenerator>
                }
                "mountain" => {
                    let mut seed = rand::thread_rng().next_u64();
                    if let Some(s) = CONFIGURATION.world_seed {
                        seed = s as u64;
                    }
                    //seed = 14686157966026215583;
                    log::info!(
                        "Initializing world with chunk generator \"mountain\" with seed ({})",
                        seed
                    );
                    Box::new(MountainChunkGenerator::new(seed)) as Box<dyn ChunkGenerator>
                }
                unknown => {
                    log::info!("Unknown chunk generator \"{}\", using \"flat\"", unknown);
                    Box::new(FlatChunkGenerator {}) as Box<dyn ChunkGenerator>
                }
            } */
            let mut seed = rand::thread_rng().next_u64();
            if let Some(s) = CONFIGURATION.world_seed {
                seed = s as u64;
            }
            world = crate::world::chunks::World::new(
                Box::new(MountainWorldGenerator::new(seed)),
                MCRegionLoader::new(&CONFIGURATION.level_name).unwrap(),
            );
            world.generate_spawn_chunks();
        }
        //world = crate::world::mcregion::temp_from_dir("New World").unwrap();
        let mut command_system = CommandSystem::new();
        command_system.register(Command::new(
            "rain",
            "set the rain status",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, executor, mut args| {
                let rain_ticks = *args[0].as_any().downcast_mut::<i32>().unwrap();
                game.rain_ticks = rain_ticks as u128;
                executor.send_message(Message::new(&format!(
                    "Setting the rain ticks to {}.",
                    args[0].display(),
                )));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "time",
            "set tie time",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, executor, mut args| {
                let time = *args[0].as_any().downcast_mut::<i32>().unwrap();
                if !(0..24001).contains(&time) {
                    return Ok(3);
                }
                GAME_GLOBAL.set_time(time as i64);
                //game.time = time as i64;
                executor.send_message(Message::new(&format!(
                    "Setting the time to {}.",
                    args[0].display(),
                )));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "gamerule",
            "change a gamerule",
            4,
            vec![CommandArgumentTypes::String, CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                use crate::game::gamerule::*;
                let rule_str = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                let state = args[1].as_any().downcast_mut::<String>().unwrap().clone();
                use std::str::FromStr;
                if let Some(rule) = game.gamerules.rules.get_mut(&rule_str) {
                    match rule {
                        GameruleValue::Boolean(_) => {
                            if let Ok(value) = bool::from_str(&state) {
                                *rule = GameruleValue::Boolean(value);
                            } else {
                                return Ok(3);
                            }
                        }
                        GameruleValue::String(_) => {
                            *rule = GameruleValue::String(state.clone());
                        }
                        GameruleValue::Int(_) => {
                            if let Ok(value) = i32::from_str_radix(&state, 10) {
                                *rule = GameruleValue::Int(value);
                            } else {
                                return Ok(3);
                            }
                        }
                    }
                    log::info!(
                        "[Command] {} set gamerule \"{}\" to \"{}\"",
                        executor.username(),
                        rule_str,
                        state
                    );
                    executor.send_message(Message::new(&format!(
                        "Changed gamerule \"{}\" to \"{}\"",
                        rule_str, state
                    )));
                } else {
                    return Ok(3);
                }
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "deop",
            "deop a player",
            4,
            vec![CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                use crate::game::gamerule::*;
                let player_name = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                let player_name = player_name.trim();
                if let Some(plr) = game.perm_level_map.get(player_name) {
                    if plr <= &1 {
                        executor.send_message(Message::new(&format!(
                            "§7Nothing changed. That player is not an operator."
                        )));
                        return Ok(3);
                    }
                }
                game.remove_op(&player_name);
                game.op_status_message(
                    &executor.username(),
                    &format!("De-opping {}", player_name.trim()),
                );
                executor.send_message(Message::new(&format!(
                    "Made \"{}\" no longer a server operator",
                    player_name.trim()
                )));
                Ok(3)
            }),
        ));
        command_system.register(Command::new(
            "op",
            "op a player",
            4,
            vec![CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                use crate::game::gamerule::*;
                let player_name = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                let player_name = player_name.trim();
                if let Some(plr) = game.perm_level_map.get(player_name) {
                    if plr >= &4 {
                        executor.send_message(Message::new(&format!(
                            "§7Nothing changed. That player is already an operator."
                        )));
                        return Ok(3);
                    }
                }
                game.add_op(&player_name);
                game.op_status_message(
                    &executor.username(),
                    &format!("Opping {}", player_name.trim()),
                );
                executor.send_message(Message::new(&format!(
                    "Made \"{}\" a server operator",
                    player_name.trim()
                )));
                Ok(3)
            }),
        ));
        command_system.register(Command::new(
            "kick",
            "kick a player",
            4,
            vec![CommandArgumentTypes::String, CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                use crate::game::gamerule::*;
                let player_name = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                let reason = args[1].as_any().downcast_mut::<String>().unwrap().clone();
                use std::str::FromStr;
                let plrs = game.players.0.borrow().clone();
                for player in plrs.iter() {
                    let player = player.1;
                    if player_name.contains(&player.get_username()) {
                        player.disconnect(reason.clone());
                        game.op_status_message(
                            &executor.username(),
                            &format!("Kicking {}", player_name),
                        );
                        executor.send_message(Message::new(&format!(
                            "Kicking player \"{}\" for \"{}\".",
                            player_name, reason
                        )));
                        return Ok(0);
                    }
                }
                executor.send_message(Message::new(&format!(
                    "Could not find player \"{}\".",
                    player_name
                )));
                Ok(3)
            }),
        ));
        command_system.register(Command::new(
            "smite",
            "smite a player",
            4,
            vec![CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                use crate::game::gamerule::*;
                let player_name = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                use std::str::FromStr;
                let plrs = game.players.0.borrow().clone();
                for player in plrs.iter() {
                    let player = player.1;
                    if player_name.contains(&player.get_username()) {
                        let pos = player.get_position();
                        game.strike_lightning(BlockPosition {
                            x: pos.x as i32,
                            y: pos.y as i32,
                            z: pos.z as i32,
                        });
                        game.op_status_message(
                            &executor.username(),
                            &format!("Smiting {}", player_name),
                        );
                        return Ok(0);
                    }
                }
                executor.send_message(Message::new(&format!(
                    "Could not find player \"{}\".",
                    player_name
                )));
                Ok(3)
            }),
        ));
        command_system.register(Command::new(
            "give",
            "give an item and count",
            4,
            vec![
                CommandArgumentTypes::String,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
            ],
            Box::new(|game, executor, mut args| {
                //log::info!("g");
                /*                 let executor =
                if let Some(executor) = executor.as_any().downcast_mut::<Arc<PlayerRef>>() {
                    executor
                } else {
                    executor.send_message(Message::new("§7You are not a player."));
                    return Ok(3);
                }; */
                // let item_id = args[0].as_any().downcast_mut::<i32>().unwrap();
                let player = args[0].as_any().downcast_mut::<String>().unwrap();
                if game.players.get_player(&player).is_none() {
                    executor.send_message(Message::new(&format!("§7Player does not exist.",)));
                    return Ok(3);
                }
                let player = game.players.get_player(&player).unwrap();
                let count = *args[2].as_any().downcast_mut::<i32>().unwrap() as i8;
                if !(0..65).contains(&count) {
                    executor.send_message(Message::new(&format!(
                        "§7Amount must be between 0 and 64!",
                    )));
                    return Ok(3);
                }
                let item = ItemStack::new(
                    *args[1].as_any().downcast_mut::<i32>().unwrap() as i16,
                    0,
                    count,
                );
                let name;
                if let Some(reg_item) = ItemRegistry::global().get_item(item.id) {
                    name = reg_item.get_name().to_string();
                } else {
                    executor.send_message(Message::new(&format!("§7Item does not exist.",)));
                    return Ok(3);
                }
                player.get_inventory().insert_itemstack(item);
                //*executor.get_item_in_hand() = item;
                game.op_status_message(
                    &executor.username(),
                    &format!(
                        "Giving {} {} ({}) to {}",
                        args[2].display(),
                        name,
                        args[1].display(),
                        args[0].display(),
                    ),
                );
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "weather",
            "change weather",
            4,
            vec![CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                let status = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                match status.as_str() {
                    "clear" => {
                        game.rain_ticks = 0;
                        game.is_storming = false;
                    }
                    "rain" => {
                        game.rain_ticks +=
                            rand::thread_rng().gen_range((2 * 60) * 20..(10 * 60) * 20);
                    }
                    "storm" => {
                        game.rain_ticks = 0;
                        game.rain_ticks +=
                            rand::thread_rng().gen_range((2 * 60) * 20..(10 * 60) * 20);
                        game.is_storming = true;
                    }
                    status => {
                        executor
                            .send_message(Message::new(&format!("Unknown status \"{}\".", status)));
                        return Ok(3);
                    }
                }
                game.op_status_message(
                    &executor.username(),
                    &format!("Changed weather to \"{}\"", status),
                );
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "tp",
            "teleport command",
            4,
            vec![CommandArgumentTypes::String, CommandArgumentTypes::String],
            Box::new(|game, executor, mut args| {
                //log::info!("g");
                let from = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                let to = args[1].as_any().downcast_mut::<String>().unwrap().clone();
                if let Some(from) = game.players.get_player(&from) {
                    if let Some(to) = game.players.get_player(&to) {
                        game.op_status_message(
                            &executor.username(),
                            &format!(
                                "Teleporting {} to {}.",
                                from.get_username(),
                                to.get_username()
                            ),
                        );
                        from.teleport(game, &to.get_position());
                    } else {
                        executor
                            .send_message(Message::new(&format!("Can't find user {}. No tp.", to)));
                        return Ok(3);
                    }
                } else {
                    executor
                        .send_message(Message::new(&format!("Can't find user {}. No tp.", from)));
                    return Ok(3);
                };
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "kill",
            "die",
            4,
            vec![],
            Box::new(|game, executor, mut args| {
                let executor =
                    if let Some(executor) = executor.as_any().downcast_mut::<Arc<PlayerRef>>() {
                        executor
                    } else {
                        return Ok(3);
                    };
                executor.set_offground_height(0.);
                executor.damage(DamageType::Void, 9999, None);
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "save-off",
            "disable world saving",
            4,
            vec![],
            Box::new(|game, executor, mut args| {
                game.op_status_message(&executor.username(), "Disabling level saving..");
                game.world_saving = false;
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "save-on",
            "enable world saving",
            4,
            vec![],
            Box::new(|game, executor, mut args| {
                game.op_status_message(&executor.username(), "Enabling level saving..");
                game.world_saving = true;
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "tell",
            "tell (player) (message)",
            1,
            vec![
                CommandArgumentTypes::String,
                CommandArgumentTypes::StringRest,
            ],
            Box::new(|game, executor, mut args| {
                let target = args[0].as_any().downcast_mut::<String>().unwrap().clone();
                let message = args[1]
                    .as_any()
                    .downcast_mut::<Vec<String>>()
                    .unwrap()
                    .clone()
                    .join(" ");
                if let Some(mut player) = game.players.get_player(&target) {
                    executor.send_message(Message::new(&format!(
                        "§7You whisper {} to {}",
                        message,
                        player.get_username()
                    )));
                    let msg = format!("§7{} whispers {}", executor.username(), message);
                    log::info!(
                        "{} whispers {} to {}",
                        executor.username(),
                        message,
                        player.get_username()
                    );
                    player.send_message(Message::new(&msg));
                } else {
                    executor.send_message(Message::new(&format!(
                        "§7There's no player by that name online."
                    )));
                }
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "me",
            "me (message)",
            1,
            vec![CommandArgumentTypes::StringRest],
            Box::new(|game, executor, mut args| {
                let message = args[0]
                    .as_any()
                    .downcast_mut::<Vec<String>>()
                    .unwrap()
                    .clone()
                    .join(" ");
                game.broadcast_message(Message::new(&format!(
                    "* {} {}",
                    executor.username(),
                    message
                )))
                .expect("Not possible!");
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "help",
            "get help",
            1,
            vec![],
            Box::new(|game, executor, _| {
                executor.send_message(Message::new("Command help:"));
                let registry = ItemRegistry::global();
                for item in game.cached_command_list.iter() {
                    executor.send_message(Message::new(&format!(
                        "/{} - {}. Args: {:?}",
                        item.root, item.desc, item.args,
                    )));
                }
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "list",
            "List players.",
            1,
            vec![],
            Box::new(|game, executor, _| {
                let mut msg = format!(
                    "{}/{} online players: ",
                    game.players.0.borrow().len(),
                    CONFIGURATION.max_players
                );
                let players = game.players.0.borrow();
                let mut players = players.iter().peekable();
                while let Some(plr) = players.next() {
                    if players.peek().is_some() {
                        msg.push_str(&format!("{}, ", plr.1.get_username()));
                    } else {
                        msg.push_str(&format!("{}.", plr.1.get_username()));
                    }
                }
                executor.send_message(Message::new(&msg));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "chunks",
            "List chunks.",
            1,
            vec![],
            Box::new(|game, executor, _| {
                executor.send_message(Message::new(&format!(
                    "There are {} chunks loaded.",
                    game.loaded_chunks.0.len()
                )));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "list-items",
            "List items.",
            1,
            vec![],
            Box::new(|game, executor, _| {
                executor.send_message(Message::new("All items:"));
                let registry = ItemRegistry::global();
                for item in registry.get_items() {
                    executor.send_message(Message::new(&format!(
                        "({}) {}",
                        item.0.0,
                        item.1.get_name()
                    )));
                }
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "cause-lag",
            "Cause lag for x ms.",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, executor, mut args| {
                let amount = args[0].as_any().downcast_mut::<i32>().unwrap().clone();
                if amount < 0 {
                    return Ok(3);
                }
                executor.send_message(Message::new(&format!("Stalling server for {}ms..", amount)));
                std::thread::sleep(Duration::from_millis(amount as u64));
                executor.send_message(Message::new("Complete!"));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "tps",
            "Server TPS.",
            1,
            vec![],
            Box::new(|game, executor, _| {
                //executor.send_message(Message::new(&format!("Memory usage: {}")));
                executor.send_message(Message::new(&format!(
                    "TPS over the last 5 seconds: {}",
                    game.tps
                )));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "stop",
            "stop the server.",
            4,
            vec![],
            Box::new(|game, executor, _| {
                if executor.permission_level() < 4 {
                    return Ok(5);
                }
                game.op_status_message(&executor.username(), "Stopping the server..");
                game.stop_server();
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "save-all",
            "save the world.",
            4,
            vec![],
            Box::new(|game, executor, _| {
                if executor.permission_level() < 4 {
                    return Ok(5);
                }
                game.op_status_message(&executor.username(), "Forcing save..");
                //executor.send_message(Message::new("Saving the world.."));
                if let Err(e) = game.save_playerdata() {
                    executor
                        .send_message(Message::new(&format!("Error saving playerdata: {:?}", e)));
                }
                if let Err(e) = game.world.to_file(&CONFIGURATION.level_name) {
                    log::info!("Error saving the world: {:?}", e);
                    game.op_status_message(&executor.username(), "Save failed. Check console for details.");
                } else {
                    game.op_status_message(&executor.username(), "Save complete.");
                }
                //game.world.to_file(&CONFIGURATION.level_name);
                Ok(0)
            }),
        ));
        let mut epic_data = HashMap::new();
        if let Ok(data) = Game::load_playerdata() {
            epic_data = data;
        }
        let mut scheduler = Scheduler::new();
        //let mut scheduler = obj.get_mut::<game::Scheduler>()?;
        scheduler.schedule_task(
            1,
            std::sync::Arc::new(Box::new(|game| {
                if game.is_storming {
                    for (chunk, _) in game.loaded_chunks.0.clone().iter() {
                        if rand::thread_rng().gen() {
                            let x = rand::thread_rng().gen_range(0..16);
                            let z = rand::thread_rng().gen_range(0..16);
                            let y = game.world.chunks.get(&chunk).expect("impossible").heightmap
                                [x as usize][z as usize];
                            let x = x + (chunk.x * 16);
                            let z = z + (chunk.z * 16);
                            game.strike_lightning(BlockPosition {
                                x: x,
                                y: y as i32,
                                z: z,
                            });
                            break;
                        }
                    }
                }
                Some(game.ticks + rand::thread_rng().gen_range(40..250))
            })),
        );
        scheduler.schedule_task(
            1,
            std::sync::Arc::new(Box::new(|game| {
                if rand::thread_rng().gen_range(0..150) == rand::thread_rng().gen_range(0..50)
                    && game.rain_ticks == 0
                {
                    game.rain_ticks += rand::thread_rng().gen_range((2 * 60) * 20..(10 * 60) * 20);
                }
                if rand::thread_rng().gen_range(0..175) == rand::thread_rng().gen_range(0..50)
                    && game.rain_ticks != 0
                {
                    game.is_storming = true;
                }
                Some(game.ticks + 60)
            })),
        );
        let mut objects = Arc::new(Objects::new());
        Arc::get_mut(&mut objects)
            .expect("cyrntly borwd")
            .insert(event_handler);

        Arc::get_mut(&mut objects)
            .expect("cyrntly borwd")
            .insert(scheduler);
        let mut perm_level_map = HashMap::new();
        let ops = crate::configuration::get_ops();
        if ops.len() > 0 {
            log::info!("Loading operators from ops.toml");
            for op in ops {
                log::info!("Loading operator {}", op);
                perm_level_map.insert(op, 4);
            }
        } else {
            log::info!("No server operators in file.");
        }
        let mut cached_command_list = Vec::new();
        for command in command_system.commands.iter() {
            cached_command_list.push(CachedCommandData {
                root: command.root.to_string(),
                desc: command.description.to_string(),
                args: command.arguments.clone(),
            });
        }
        Self {
            objects: objects,
            players: PlayerList(Arc::new(RefCell::new(HashMap::new()))),
            systems: Arc::new(RefCell::new(systems)),
            world,
            block_updates: Vec::new(),
            command_system: Arc::new(RefCell::new(command_system)),
            ticks: 0,
            tile_entities: Arc::new(RefCell::new(HashMap::new())),
            entities: Arc::new(RefCell::new(HashMap::new())),
            loaded_chunks: LoadedChunks(HashMap::new()),
            persistent_player_data: Arc::new(RefCell::new(epic_data)),
            gamerules: gamerule::Gamerules::default(),
            tps: 0.,
            async_commands: recv,
            async_chat_manager: async_chat_manager,
            perm_level_map: perm_level_map,
            cached_command_list: cached_command_list,
            rain_ticks: 0,
            is_raining: false,
            is_storming: false,
            world_saving: true,
        }
    }
    pub fn insert_object<T>(&mut self, object: T)
    where
        T: 'static,
    {
        Arc::get_mut(&mut self.objects)
            .expect("cyrntly borwd")
            .insert(object);
    }
    pub fn execute_command(
        &mut self,
        executor: &mut dyn CommandExecutor,
        command: &str,
    ) -> anyhow::Result<usize> {
        let system = self.command_system.clone();
        let mut system = system.borrow_mut();
        system.execute(self, executor, command)
    }
    pub fn poll_new_players(&mut self, server: &mut Server) -> anyhow::Result<()> {
        for id in server.accept_clients() {
            if let Err(e) = self.accept_player(server, id) {
                log::info!("Error accepting client: {:?}", e);
            }
        }
        Ok(())
    }
    pub fn accept_packets(&mut self, server: &mut Server) -> anyhow::Result<()> {
        let mut packets = Vec::new();
        for (id, player) in self.players.0.borrow().clone() {
            if let Some(cl) = server.clients.borrow().get(&id) {
                for packet in cl.borrow().recieved_packets() {
                    //log::info!("Got one");
                    packets.push((player.clone(), packet));
                }
            }
        }
        for (player, packet) in packets {
            let orig = player.is_crouching(); // borrow().crouching;
            let orig_hi = player.get_item_in_hand_clone(); // .borrow().get_item_in_hand_ref().clone();
            let orig_pos = player.get_position_clone(); // .borrow().position.clone();
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Err(e) = crate::network::packet::handler::handle_packet(
                    self,
                    server,
                    player.clone(),
                    packet,
                ) {
                    log::error!(
                        "Error handling packet from user {}: {:?}",
                        player.clone().get_username(), /*borrow().username*/
                        e
                    );
                }
            })) {
                //log::info!("Critical error handling packet from user {}.", player.clone().get_username());
                player.write_packet(ServerPacket::Disconnect {
                    reason: String::from("A fatal error has occured."),
                });
                player.remove(Some(format!(
                    "Fatal error handling packet from user {}.",
                    player.clone().get_username()
                )));
            }
            if orig != player.is_crouching()
            /* .borrow().crouching */
            {
                //crate::systems::update_crouch(self, server, player.clone())?;
            }
            if orig_hi != player.get_item_in_hand_clone() {
                player.equipment_check(self)?;
                //crate::systems::update_held_items(self, server, player)?
            }
            if orig_pos != player.get_position_clone() {
                //crate::systems::check_chunks(self, server, &mut player.borrow_mut())?;
            }
        }
        Ok(())
    }
    pub fn broadcast_message(&mut self, message: Message) -> anyhow::Result<()> {
        log::info!("{}", message);
        for (_, mut player) in self.players.0.borrow().clone() {
            /*             let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            }; */
            if player.can_borrow() {
                player.send_message(message.clone()); // .chatbox.push
            }
        }
        Ok(())
    }
    pub fn broadcast_packet(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            /*             let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            }; */
            player.write_packet(packet.clone());
        }
        Ok(())
    }
    pub fn hide_player(&mut self, to_remove: &Player) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            /*             let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            }; */
            if player.can_borrow() {
                if player
                    .unwrap()
                    .unwrap()
                    .rendered_players
                    .get(&(to_remove.id, to_remove.username.clone()))
                    .is_some()
                {
                    player.write_packet(ServerPacket::DestroyEntity {
                        eid: to_remove.id.0,
                    });
                }
                player
                    .unwrap()
                    .unwrap()
                    .rendered_players
                    .remove(&(to_remove.id, to_remove.username.clone()));
            } else {
                //log::info!("Can't");
            }
        }
        Ok(())
    }
    pub fn broadcast_to_loaded(
        &mut self,
        origin: &Player,
        packet: ServerPacket,
    ) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            /*             let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            }; */
            let plr = player.unwrap();
            if plr.is_err() {
                continue;
            }
            if plr // unwrap().
                .unwrap()
                .rendered_players
                .get(&(origin.id, origin.username.clone()))
                .is_some()
            {
                player.write_packet(packet.clone());
            }
        }
        Ok(())
    }
    fn accept_player(&mut self, server: &mut Server, id: EntityID) -> anyhow::Result<()> {
        log::debug!("Player {:?}", id);
        let clients = server.clients.borrow_mut();
        let client = clients.get(&id).unwrap().clone();
        let mut client = client.borrow_mut();
        let packet = ServerPacket::ServerLoginRequest {
            entity_id: id.0,
            unknown: "".to_string(),
            map_seed: self.world.generator.get_seed() as i64,
            dimension: 0,
        };
        client.write(packet)?;
        let addr = client.addr;
        drop(clients);
        if self.players.0.borrow().len() + 1 > CONFIGURATION.max_players as usize {
            client.write(ServerPacket::Disconnect {
                reason: "The server is full!".to_string(),
            })?;
            return Err(anyhow::anyhow!("The server is full!"));
        }
        if self.is_raining {
            client.write(ServerPacket::NewInvalidState { reason: 1 })?;
        }
        let plrs = self.players.0.borrow();
        let plrs2 = plrs.clone();
        drop(plrs);
        for player in plrs2.iter() {
            let mut lg = player.1;
            let name = lg.get_username(); // .username.clone();
            let id = lg.get_id(); // .id;
                                  //drop(lg);
            if name == client.username {
                lg.disconnect("You logged in from another location".to_string());
                //panic!("Same username");
                //self.disconnect(server, id, "You logged in from another location")?;
                IDS.lock().unwrap().push(id.0);
            }
        }
        let mut persist_data: Option<PersistentPlayerData> = None;
        let mut pos = self.world.spawn_position;
        if let Some(data) = self.persistent_player_data.borrow().get(&client.username) {
            log::debug!("Position from persist: {:?}", data.position);
            pos = data.position;
            pos.y += 2.;
            persist_data = Some(data.clone());
        }
        log::info!(
            "{}/{} logging in with entity id {} at position {}",
            client.username,
            client.addr,
            id.0,
            pos
        );
        let mut loaded_chunks = Vec::new();
        let spawnchunk = pos.to_chunk_coords();
        for x in -CONFIGURATION.chunk_distance..CONFIGURATION.chunk_distance {
            for z in -CONFIGURATION.chunk_distance..CONFIGURATION.chunk_distance {
                let spawnchunk = ChunkCoords {
                    x: (pos.x as i32) + x,
                    z: (pos.z as i32) + z,
                };
                if self.world.check_chunk_exists(&spawnchunk) {
                    loaded_chunks.push(spawnchunk.clone());
                    self.world
                        .chunk_to_packets(spawnchunk, client.packet_send_sender.clone())?;
                    self.loaded_chunks.push(spawnchunk);
                } else {
                    self.world.init_chunk(&ChunkCoords {
                        x: spawnchunk.x,
                        z: spawnchunk.z,
                    });
/*                     if let Some(c) = self.world.mcr_helper.get_chunk(ChunkCoords {
                        x: spawnchunk.x,
                        z: spawnchunk.z,
                    }) {
                        self.world.chunks.insert(spawnchunk, c);
                    } else {
                        // TODO STRUCTURE GEN
                        self.world.chunks.insert(
                            spawnchunk,
                            self.world.generator.gen_chunk(ChunkCoords {
                                x: spawnchunk.x,
                                z: spawnchunk.z,
                            }),
                        );
                        loaded_chunks.push(spawnchunk.clone());
                    } */
                    self.world
                        .chunk_to_packets(spawnchunk, client.packet_send_sender.clone())?;
                    self.loaded_chunks.push(spawnchunk);
                }
            }
        }
        //std::thread::sleep_ms(1000);
        //log::info!("finished sending chunks");
        /*         self.world
        .chunks[&spawnchunk].data[0].as_ref().unwrap().to_packets_section_raw(&mut client.packet_send_sender, &mut Vec::new())
        .unwrap(); */
        client.write(ServerPacket::PlayerPositionAndLook {
            x: pos.x,
            stance: pos.y,
            y: 67.240000009536743,
            z: pos.z,
            yaw: pos.yaw,
            pitch: pos.pitch,
            on_ground: pos.on_ground,
        })?;
        client.write(ServerPacket::SpawnPosition {
            x: self.world.spawn_position.x as i32,
            y: self.world.spawn_position.y as i32,
            z: self.world.spawn_position.z as i32,
        })?;
        //client.write(ServerPacket::PlayerTeleport { player_id: -1, position: Position::from_pos(64, 128, 64)})?;
        let list = self.players.clone();
        let mut players = self.players.0.borrow_mut();
        let mut health = 20;
        let mut inventory = Inventory::new();
        if let Some(data) = persist_data {
            health = data.health;
            inventory = data.inventory;
            client.write(ServerPacket::UpdateHealth { health })?;
        }
        let mut perm_level = 1;
        if let Some(level) = self.perm_level_map.get(&client.username) {
            perm_level = *level;
        }
        players.insert(
            id,
            Arc::new(PlayerRef::new(RefCell::new(Player {
                username: client.username.clone(),
                id,
                position: pos.clone(),
                recv_packets_recv: client.recv_packets_recv.clone(),
                packet_send_sender: client.packet_send_sender.clone(),
                rendered_players: HashMap::new(),
                perm_level: 1,
                players_list: list,
                crouching: false,
                health: health,
                last_health: 20,
                last_position: pos.clone(),
                dead: false,
                world: 0,
                last_void_dmg: std::time::Instant::now(),
                inventory: inventory,
                last_inventory: Inventory::new(),
                held_slot: 0,
                last_dmg_type: DamageType::None,
                last_transaction_id: 0,
                current_cursored_item: None,
                loaded_chunks: loaded_chunks,
                has_loaded_before: Vec::new(),
                since_last_attack: std::time::Instant::now(),
                mining_block: MiningBlockData::default(),
                rendered_entities: HashMap::new(),
                open_inventories: HashMap::new(),
                held_item_changed: false,
                inv_changed: false,
                checking_fall: true,
                pos_changed: false,
                offground_height: 0.0,
                chatbox: Chatbox::default(),
                socket_addr: addr,
                all_player_data: self.persistent_player_data.clone(),
                permission_level: perm_level,
                air: 300,
                last_drown_tick: 0,
                metadata_changed: true,
                async_chat: self.async_chat_manager.clone(),
                last_keepalive_time: self.ticks,
            }))),
        );
        let us = players.get(&id).unwrap().clone();
        drop(players);
        self.broadcast_message(Message::new(&format!(
            "§e{} joined the game.",
            client.username
        )))?;
        if self
            .persistent_player_data
            .borrow()
            .get(&us.get_username())
            .is_none()
        {
            self.persistent_player_data.borrow_mut().insert(
                us.get_username(),
                PersistentPlayerData {
                    username: us.get_username(),
                    position: us.get_position_clone(),
                    health: us.get_health(),
                    inventory: us.get_inventory().clone(),
                },
            );
        }
        Ok(())
    }
}
