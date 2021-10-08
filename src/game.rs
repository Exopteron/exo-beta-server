use crate::commands::*;
use crate::entities::*;
use crate::network::ids::EntityID;
use crate::network::ids::IDS;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::objects::Objects;
use crate::server::Server;
use crate::systems::Systems;
use flume::{Receiver, Sender};
use std::any::Any;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::cell::{Ref, RefMut};
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
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
    pub block: crate::world::Block,
}
use glam::DVec3;
impl Position {
    pub const FEET_DISTANCE: i16 = 51;
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
    pub fn distance(&self, other: &Position) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2))
            .sqrt()
    }
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
}
#[derive(Clone, PartialEq, Debug)]
pub struct ItemStack {
    pub id: i16,
    pub damage: i16,
    pub count: i8,
}
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
    pub fn new(id: i16, damage: i16, count: i8) -> Self {
        Self { id, damage, count }
    }
}
#[derive(Clone, PartialEq)]
pub struct Inventory {
    pub items: HashMap<i8, ItemStack>,
}
impl Inventory {
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
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}
//use num::num_integer::roots::Roots;
impl ChunkCoords {
    pub fn distance(&self, other: &ChunkCoords) -> usize {
        (((self.x - other.x).pow(2) + (self.z - other.z).pow(2)) as f64).sqrt() as usize
    }
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
#[derive(Clone)]
pub struct Chatbox {
    pub messages: Vec<Message>,
}
impl Chatbox {
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
pub struct PlayerRef {
    player: RefCell<Player>,
}
impl PlayerRef {
    pub fn new(player: RefCell<Player>) -> Self {
        Self { player }
    }
    pub fn send_message(&self, message: Message) {
        self.player.borrow_mut().chatbox.push(message);
    }
    pub fn write_packet(&self, packet: ServerPacket) {
        self.player.borrow_mut().write(packet);
    }
    pub fn is_crouching(&self) -> bool {
        self.player.borrow().crouching.clone()
    }
    // GAHHHHHHHHHH I JUST FIGURED THIS OUT NOW i'll do it tommorow. (did it almost)
    pub fn get_item_in_hand(&self) -> RefMut<'_, ItemStack> {
        RefMut::map(self.player.borrow_mut(), |plr| plr.get_item_in_hand_mut().unwrap())
    }
    pub fn get_item_in_hand_clone(&self) -> ItemStack {
        self.player.borrow().get_item_in_hand_ref().clone()
    }
    pub fn get_position_clone(&self) -> Position {
        self.player.borrow().position.clone()
    }
    pub fn get_last_position_clone(&self) -> Position {
        self.player.borrow().last_position.clone()
    }
    pub fn get_world(&self) -> i8 {
        self.player.borrow().world.clone()
    }
    pub fn get_username(&self) -> String {
        self.player.borrow().username.clone()
    }
    pub fn set_held_slot(&self, idx: i16) {
        self.player.borrow_mut().held_item_changed = true;
        self.player.borrow_mut().held_slot = idx;
    }
    pub fn get_checking_fall(&self) -> bool {
        self.player.borrow().checking_fall.clone()
    }
    pub fn set_checking_fall(&self, value: bool) {
        self.player.borrow_mut().checking_fall = value;
    }
/*     pub fn get_rendered_players_ref(&self) -> &HashMap<(EntityID, String), RenderedPlayerInfo> {
        &self.player.borrow().rendered_players
    }
    pub fn get_rendered_players_mut(&self) -> &mut HashMap<(EntityID, String), RenderedPlayerInfo> {
        &mut self.player.borrow_mut().rendered_players
    } */
    pub fn get_id(&self) -> EntityID {
        self.player.borrow().id.clone()
    }
    pub fn disconnect(&self, reason: String) {
        self.player.borrow_mut().disconnect(reason);
    }
    pub fn get_health(&self) -> i16 {
        self.player.borrow().health.clone()
    }
    pub fn set_offground_height(&self, height: f32) {
        self.player.borrow_mut().offground_height = height;
    }
    pub fn damage(&mut self, damage_type: DamageType, amount: i16, damagee: Option<&mut Player>) {
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
        //self.write_packet(ServerPacket::UpdateHealth { health: self.health });
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
            /*             let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            }; */
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
    pub fn set_health(&self, health: i16) {
        let mut us = self.player.borrow_mut();
        us.health = health;
        us.write(ServerPacket::UpdateHealth { health });
    }
    pub fn get_loaded_chunks(&self) -> Vec<ChunkCoords> {
        self.player.borrow().loaded_chunks.clone()
    }
    pub fn held_items_check(&self, game: &mut Game) -> anyhow::Result<()> {
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
                damage: 0,
            },
        )?;
        Ok(())
    }
    pub fn is_dead(&self) -> bool {
        self.player.borrow().dead.clone()
    }
    pub fn set_dead(&self, dead: bool) {
        self.player.borrow_mut().dead = dead;
    }
    pub fn get_position(&self) -> Position {
        self.player.borrow_mut().position
    }
    pub fn set_position(&self, pos: Position) {
        self.player.borrow_mut().position = pos;
    }
    pub fn set_last_position(&self, position: Position) {
        self.player.borrow_mut().last_position = position;
    }
    pub fn get_crouching(&self) -> bool {
        self.player.borrow_mut().crouching
    }
    pub fn set_crouching(&self, crouching: bool) {
        self.player.borrow_mut().crouching = crouching;
    }
    pub fn get_inventory(&self) -> RefMut<'_, Inventory> {
        self.player.borrow_mut().inv_changed = true;
        RefMut::map(self.player.borrow_mut(), |plr| &mut plr.inventory)
    }
    pub fn can_borrow(&self) -> bool {
        self.player.try_borrow().is_ok()
    }
/*     pub fn get_inventory_slot_mut(&self, slot: i8) -> Option<RefMut<'_, ItemStack>> {
        if self.player.borrow().inventory.items.get(&slot).is_none() {
            return None;
        }
        Some(RefMut::map(self.player.borrow_mut(), |plr| plr.inventory.items.get_mut(&slot).expect("Slot does not exist, fixlater")))
    }
    pub fn get_inventory_slot(&self, slot: i8) -> Option<ItemStack> {
        Some(self.player.borrow().inventory.items.get(&slot)?.clone())
    }
    pub fn set_inventory_slot(&self, slot: i8, stack: ItemStack) -> Option<()> {
        *self.player.borrow_mut().inventory.items.get_mut(&slot)? = stack;
        Some(())
    } */
    pub fn get_current_cursored_item_mut(&self) -> RefMut<'_, Option<ItemStack>> {
        RefMut::map(self.player.borrow_mut(), |plr| &mut plr.current_cursored_item)
    }
    pub fn unwrap(&self) -> Result<RefMut<'_, Player>, std::cell::BorrowMutError> {
        self.player.try_borrow_mut()
    }
    pub fn sync_inventory(&self) {
        let mut player = self.unwrap().unwrap();
        let inv = player.inventory.clone();
        log::info!("Writing inv items");
        player.write(ServerPacket::InvWindowItems { inventory: inv });
        player.last_inventory = player.inventory.clone();
        player.inv_changed = false;
    }
    pub fn tick(&self, game: &mut Game) -> anyhow::Result<()> {
        if self.player.borrow().held_item_changed.clone() {
            self.held_items_check(game)?;
        }
        if self.player.borrow().inv_changed.clone() {
            self.sync_inventory();
        }
        let interval = Duration::from_millis(750);
        let mut cl = self.player.borrow_mut();
        // Fall damage check
        if cl.position.on_ground {
            if cl.offground_height > (cl.position.y as f32) {
                log::info!("Offground: {}, pos: {}", cl.offground_height, cl.position.y);
                let height = cl.offground_height - (cl.position.y as f32);
                if height > 0.0 {
                    let fall_dmg = (height - 3.0).max(0.0);
                    log::info!("Damage: {}", fall_dmg.round());
                    cl.damage(DamageType::Fall, fall_dmg.round() as i16, None);
                    log::info!("Fell from a height of {}", height);
                }
                cl.offground_height = 0.0;
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
            cl.chatbox.push(msg);
            println!("Yo!");
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
        cl.write(ServerPacket::InvWindowItems {
            inventory: inv,
        });
        // Chat messages
        cl.chatbox.clone().messages.retain(|message| {
            cl.write(ServerPacket::ChatMessage { message: message.message.clone() });
            false
        });
        cl.chatbox = crate::game::Chatbox::default();
        // Spawn players check
        //log::info!("Running spawn players tick");
        let plrlist = cl.players_list.0.borrow().clone();
        for player in plrlist.iter() {
            if player.0 != &cl.id {
                let other_username = player.1.get_username();
                if cl.rendered_players.get(&(*player.0, other_username.clone())).is_none() /* && player.1.get_loaded_chunks().contains(&ChunkCoords::from_pos(&cl.position)) */ {
                    cl.rendered_players.insert((*player.0, other_username),  crate::game::RenderedPlayerInfo {position: player.1.get_position_clone(), held_item: player.1.get_item_in_hand_clone() });
                    let pos = player.1.get_position_clone();
                    cl.write(ServerPacket::NamedEntitySpawn { eid: player.1.get_id().0, name: player.1.get_username(), x: (pos.x * 32.0).round() as i32, y: (pos.y * 32.0).round() as i32, z: (pos.z * 32.0).round() as i32, rotation: 0, pitch: 0, current_item: 0 });
                }
            }
        }
        Ok(())
    }
}
pub struct Player {
    pub username: String,
    pub id: EntityID,
    pub position: Position,
    pub last_position: Position,
    pub recv_packets_recv: Receiver<ClientPacket>,
    pub packet_send_sender: Sender<ServerPacket>,
    pub rendered_players: HashMap<(EntityID, String), RenderedPlayerInfo>,
    pub rendered_entities: HashMap<EntityID, Box<dyn Entity>>,
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
    pub offground_height: f32,
    pub checking_fall: bool,
    players_list: PlayerList,
}
impl Player {
    pub fn sync_inventory(&mut self) {
        let mut player = self;
        let inv = player.inventory.clone();
        log::info!("Writing inv items");
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
        self.remove();
    }
    pub fn remove(&mut self) {
        log::info!("{} left the game.", self.username);
        for player in self.players_list.0.borrow().iter() {
            if player.1.can_borrow() {
                player
                .1
                .send_message(Message::new(&format!("{} left the game.", self.username)));
            }
            /*             if let Ok(mut plr) = player.1.try_borrow_mut() {
            } else {
                continue;
            } */
        }
        self.players_list.0.borrow_mut().remove(&self.id);
        IDS.lock().unwrap().push(self.id.0);
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
        self.write(ServerPacket::UpdateHealth { health: self.health });
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
        self.inventory
            .items
            .get_mut(&((self.held_slot + 36) as i8))
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
        if let Err(_) = self.packet_send_sender.send(packet) {
            self.players_list.0.borrow_mut().remove(&self.id);
            //clients.remove(&id);
            IDS.lock().unwrap().push(self.id.0);
        }
    }
    pub async fn read(&mut self) -> anyhow::Result<ClientPacket> {
        Ok(self.recv_packets_recv.recv_async().await?)
    }
}
impl CommandExecutor for Player {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
#[derive(Clone, Debug)]
pub struct Message {
    pub message: String,
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
pub struct Game {
    pub objects: Arc<Objects>,
    pub players: PlayerList,
    pub entities: Arc<RefCell<Vec<Box<dyn Entity>>>>,
    pub systems: Arc<RefCell<Systems>>,
    pub world: crate::world::World,
    pub block_updates: Vec<Block>,
    pub command_system: Arc<RefCell<CommandSystem>>,
    pub time: i64,
}
impl Game {
    pub fn new(systems: Systems) -> Self {
        //let generator = crate::temp_chunks::FlatWorldGenerator::new(64, 1,1, 1);
        let world = crate::world::World::crappy_generate();
        let mut command_system = CommandSystem::new();
        command_system.register(Command::new(
            "give",
            "give an item and count",
            vec![CommandArgumentTypes::Int, CommandArgumentTypes::Int],
            Box::new(|game, executor, mut args| {
                log::info!("g");
                let executor = if let Some(executor) = executor.as_any().downcast_mut::<Player>() {
                    executor
                } else {
                    return Ok(3);
                };
                // let item_id = args[0].as_any().downcast_mut::<i32>().unwrap();
                let item = ItemStack::new(
                    *args[0].as_any().downcast_mut::<i32>().unwrap() as i16,
                    0,
                    *args[1].as_any().downcast_mut::<i32>().unwrap() as i8,
                );
                *executor.get_item_in_hand() = item;
                executor.chatbox.push(Message::new(&format!(
                    "Giving you {} {}.",
                    args[1].display(),
                    args[0].display()
                )));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "abc",
            "test command 2",
            vec![CommandArgumentTypes::String],
            Box::new(|game, executor, args| {
                log::info!("g");
                let executor = if let Some(executor) = executor.as_any().downcast_mut::<Player>() {
                    executor
                } else {
                    return Ok(3);
                };
                executor.chatbox.push(Message::new(&format!(
                    "It works! Hello {}",
                    args[0].display()
                )));
                Ok(0)
            }),
        ));
        command_system.register(Command::new(
            "test",
            "test command",
            vec![CommandArgumentTypes::String],
            Box::new(|game, executor, args| {
                log::info!("g");
                let executor = if let Some(executor) = executor.as_any().downcast_mut::<Player>() {
                    executor
                } else {
                    return Ok(3);
                };
                executor.chatbox.push(Message::new(&format!(
                    "It works! Hello {}",
                    args[0].display()
                )));
                executor.position.y += 5.0;
                /*             let packets = world.to_packets();
                for packet in packets {
                    executor.write(packet)?;
                } */
                Ok(0)
            }),
        ));
        Self {
            objects: Arc::new(Objects::new()),
            players: PlayerList(Arc::new(RefCell::new(HashMap::new()))),
            systems: Arc::new(RefCell::new(systems)),
            world,
            block_updates: Vec::new(),
            command_system: Arc::new(RefCell::new(command_system)),
            time: 0,
            entities: Arc::new(RefCell::new(Vec::new())),
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
            self.accept_player(server, id)?;
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
            if let Err(e) =
                crate::network::packet::handler::handle_packet(self, server, player.clone(), packet)
            {
                log::error!(
                    "Error handling packet from user {}: {:?}",
                    player.clone().get_username(), /*borrow().username*/
                    e
                );
            }
            if orig != player.is_crouching()
            /* .borrow().crouching */
            {
                crate::systems::update_crouch(self, server, player.clone())?;
            }
            if orig_hi != player.get_item_in_hand_clone() {
                player.held_items_check(self)?;
                //crate::systems::update_held_items(self, server, player)?
            }
            if orig_pos != player.get_position_clone() {
                //crate::systems::check_chunks(self, server, &mut player.borrow_mut())?;
            }
        }
        Ok(())
    }
    pub fn broadcast_message(&mut self, message: Message) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
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
                .unwrap().unwrap().rendered_players
                .get(&(to_remove.id, to_remove.username.clone()))
                .is_some()
            {
                player.write_packet(ServerPacket::DestroyEntity {
                    eid: to_remove.id.0,
                });
            }
            player
                .unwrap().unwrap().rendered_players
                .remove(&(to_remove.id, to_remove.username.clone()));
            } else {
                log::info!("Can't");
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
                .unwrap().rendered_players
                .get(&(origin.id, origin.username.clone()))
                .is_some()
            {
                player.write_packet(packet.clone());
            }
        }
        Ok(())
    }
    fn accept_player(&mut self, server: &mut Server, id: EntityID) -> anyhow::Result<()> {
        log::info!("Player {:?}", id);
        let clients = server.clients.borrow_mut();
        let client = clients.get(&id).unwrap().clone();
        let mut client = client.borrow_mut();
        drop(clients);
        self.broadcast_message(Message::new(&format!(
            "{} joined the game.",
            client.username
        )))?;
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
        let mut loaded_chunks = Vec::new();
        client.write(ServerPacket::SpawnPosition {
            x: (3.0f64 * 32.0) as i32,
            y: (20.0f64 * 32.0) as i32,
            z: (5.0f64 * 32.0) as i32,
        })?;
        let spawnchunk = ChunkCoords { x: 0, z: 0 };
        loaded_chunks.push(spawnchunk.clone());
        self.world
            .to_packets_new(&mut client.packet_send_sender, &mut Vec::new())
            .unwrap();
        client.write(ServerPacket::PlayerPositionAndLook {
            x: 3.0,
            stance: 67.240000009536743,
            y: 20.0,
            z: 5.0,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
        })?;
        //client.write(ServerPacket::PlayerTeleport { player_id: -1, position: Position::from_pos(64, 128, 64)})?;
        let list = self.players.clone();
        let mut players = self.players.0.borrow_mut();
        let pos = Position::from_pos(3.0, 20.0, 5.0);
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
                health: 20,
                last_health: 20,
                last_position: pos.clone(),
                dead: false,
                world: 0,
                last_void_dmg: std::time::Instant::now(),
                inventory: Inventory::new(),
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
                held_item_changed: false,
                inv_changed: false,
                checking_fall: true,
                offground_height: 0.0,
                chatbox: Chatbox::default(),
            }))),
        );
        Ok(())
    }
}
