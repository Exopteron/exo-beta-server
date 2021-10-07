use flume::{Sender, Receiver};
use crate::systems::Systems;
use crate::network::packet::{ClientPacket, ServerPacket};
use crate::network::ids::EntityID;
use crate::entities::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::cell::RefCell;
use std::rc::Rc;
use std::any::Any;
use crate::commands::*;
use crate::server::Server;
use crate::objects::Objects;
use crate::network::ids::IDS;
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
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2) + (self.z - other.z).powi(2)).sqrt()
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
        let mut us_pos = BlockPosition { x: (self.x + 0.1).floor() as i32, y: (self.y + 0.1).floor() as i32, z: (self.z + 0.1).floor() as i32 };
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
        Self { block: BlockPosition::default(), face: 0 }
    }
}
#[derive(Clone, PartialEq, Debug)]
pub enum DamageType {
    Void,
    Player { damager: String },
    None,
}
#[derive(Clone, PartialEq, Debug)]
pub struct ItemStack {
    pub id: i16,
    pub damage: i16,
    pub count: i8,
}
impl std::default::Default for ItemStack {
    fn default() -> Self {
        Self { id: 0, damage: 0, count: 0 }
    }
}
impl ItemStack {
    pub fn new(id: i16, damage: i16, count: i8) -> Self {
        Self { id, damage, count }
    }
}
#[derive(Clone, PartialEq)]
pub struct Inventory {
    pub items: HashMap<i8, ItemStack>
}
impl Inventory {
    pub fn new() -> Self {
        let mut slots = HashMap::new();
        for i in 0..45 {
            slots.insert(i, ItemStack::default());
        }
        Self { items: slots }
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
        Self { x: (position.x / 16.0) as i32, z: (position.z / 16.0) as i32 }
    }
}
#[derive(Clone, Debug)]
pub struct RenderedPlayerInfo {
    pub position: Position,
    pub held_item: ItemStack,
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
    pub chatbox: Vec<Message>,
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
    players_list: PlayerList,
}
impl Player {
    pub fn add_velocity(&mut self, x: i16, y: i16, z: i16) {
        self.write(ServerPacket::EntityVelocity { eid: self.id.0, velocity_x: x, velocity_y: y, velocity_z: z });
    }
    pub fn disconnect(&mut self, reason: String) {
        self.write(ServerPacket::Disconnect { reason });
        self.remove();
    }
    pub fn remove(&mut self) {
        for player in self.players_list.0.borrow().iter() {
            if let Ok(mut plr) = player.1.try_borrow_mut() {
                plr.chatbox.push(Message::new(&format!("{} left the game.", self.username)));
            } else {
                continue;
            }
        }
        self.players_list.0.borrow_mut().remove(&self.id);
        IDS.lock().unwrap().push(self.id.0);  
    }
    pub fn damage(&mut self, damage_type: DamageType, amount: i16, damagee: Option<&mut Player>) {
        self.health -= amount;
        let id = self.id.0;
        self.write(ServerPacket::Animation { eid: id, animate: 2});
        self.write(ServerPacket::EntityStatus { eid: id, entity_status: 2 });
        if let Some(plr) = damagee {
            plr.write(ServerPacket::EntityStatus { eid: self.id.0, entity_status: 2 });
            plr.write(ServerPacket::Animation { eid: self.id.0, animate: 2 });
        }
        for (_, player) in &*self.players_list.0.borrow() {
            let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            };
            player.write(ServerPacket::EntityStatus { eid: self.id.0, entity_status: 2 });
            player.write(ServerPacket::Animation { eid: self.id.0, animate: 2 });
        }
        self.last_dmg_type = damage_type;
    }
    pub fn get_item_in_hand_ref(&self) -> &ItemStack {
        //log::info!("Checking slot {}", self.held_slot + 36);
        self.inventory.items.get(&((self.held_slot + 36) as i8)).unwrap()
    }
    pub fn get_item_in_hand(&mut self) -> &mut ItemStack {
        //log::info!("Checking slot {}", self.held_slot + 36);
        self.inventory.items.get_mut(&((self.held_slot + 36) as i8)).unwrap()
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
        Self { message: msg.to_string() }
    }
}
#[derive(Clone)]
pub struct PlayerList(pub Arc<RefCell<HashMap<EntityID, Arc<RefCell<Player>>>>>);
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
        command_system.register(Command::new("give", "give an item and count", vec![CommandArgumentTypes::Int, CommandArgumentTypes::Int], Box::new(|game, executor, mut args| {
            log::info!("g");
            let executor = if let Some(executor) = executor.as_any().downcast_mut::<Player>() {
                executor
            } else {
                return Ok(3);
            };
            // let item_id = args[0].as_any().downcast_mut::<i32>().unwrap();
            let item = ItemStack::new(*args[0].as_any().downcast_mut::<i32>().unwrap() as i16, 0, *args[1].as_any().downcast_mut::<i32>().unwrap() as i8);
            *executor.get_item_in_hand() = item;
            executor.chatbox.push(Message::new(&format!("Giving you {} {}.", args[1].display(), args[0].display())));
            Ok(0)
        })));
        command_system.register(Command::new("abc", "test command 2", vec![CommandArgumentTypes::String], Box::new(|game, executor, args| {
            log::info!("g");
            let executor = if let Some(executor) = executor.as_any().downcast_mut::<Player>() {
                executor
            } else {
                return Ok(3);
            };
            executor.chatbox.push(Message::new(&format!("It works! Hello {}", args[0].display())));
            Ok(0)
        })));
        command_system.register(Command::new("test", "test command", vec![CommandArgumentTypes::String], Box::new(|game, executor, args| {
            log::info!("g");
            let executor = if let Some(executor) = executor.as_any().downcast_mut::<Player>() {
                executor
            } else {
                return Ok(3);
            };
            executor.chatbox.push(Message::new(&format!("It works! Hello {}", args[0].display())));
            executor.position.y += 5.0;
/*             let packets = world.to_packets();
            for packet in packets {
                executor.write(packet)?;
            } */
            Ok(0)
        })));
        Self { objects: Arc::new(Objects::new()), players: PlayerList(Arc::new(RefCell::new(HashMap::new()))), systems: Arc::new(RefCell::new(systems)), world, block_updates: Vec::new(), command_system: Arc::new(RefCell::new(command_system)), time: 0, entities: Arc::new(RefCell::new(Vec::new())) }
    }
    pub fn insert_object<T>(&mut self, object: T) 
    where
        T: 'static,
    {
        Arc::get_mut(&mut self.objects).expect("cyrntly borwd").insert(object);
    }
    pub fn execute_command(&mut self, executor: &mut dyn CommandExecutor, command: &str) -> anyhow::Result<usize> {
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
            let orig = player.borrow().crouching;
            let orig_hi = player.borrow().get_item_in_hand_ref().clone();
            let orig_pos = player.borrow().position.clone();
            if let Err(e) = crate::network::packet::handler::handle_packet(self, server, &mut player.borrow_mut(), packet) {
                log::error!("Error handling packet from user {}: {:?}", player.borrow().username, e);
            }
            if orig != player.borrow().crouching {
                crate::systems::update_crouch(self, server, &player.borrow())?;
            }
            if &orig_hi != player.borrow().get_item_in_hand_ref() {
                crate::systems::update_held_items(self, server, &player.borrow())?
            }
            if orig_pos != player.borrow().position {
                //crate::systems::check_chunks(self, server, &mut player.borrow_mut())?;
            }
        } 
        Ok(())
    }
    pub fn broadcast_message(&mut self, message: Message) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            };
            player.chatbox.push(message.clone());
        }
        Ok(())
    }
    pub fn broadcast_packet(&mut self, packet: ServerPacket) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            };
            player.write(packet.clone());
        }
        Ok(())
    }
    pub fn hide_player(&mut self, to_remove: &Player) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            };
            if player.rendered_players.get(&(to_remove.id, to_remove.username.clone())).is_some() {
                player.write(ServerPacket::DestroyEntity { eid: to_remove.id.0 });
            }
            player.rendered_players.remove(&(to_remove.id, to_remove.username.clone()));
        }
        Ok(())
    }
    pub fn broadcast_to_loaded(&mut self, origin: &Player, packet: ServerPacket) -> anyhow::Result<()> {
        for (_, player) in self.players.0.borrow().clone() {
            let mut player = if let Ok(plr) = player.try_borrow_mut() {
                plr
            } else {
                continue;
            };
            if player.rendered_players.get(&(origin.id, origin.username.clone())).is_some() {
                player.write(packet.clone());
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
        self.broadcast_message(Message::new(&format!("{} joined the game.", client.username)))?;
        let plrs = self.players.0.borrow();
        let plrs2 = plrs.clone();
        drop(plrs);
        for player in plrs2.iter() {
            let mut lg = player.1.borrow_mut();
            let name = lg.username.clone();
            let id = lg.id;
            //drop(lg);
            if name == client.username {
                lg.disconnect("You logged in from another location".to_string());
                //panic!("Same username");
                //self.disconnect(server, id, "You logged in from another location")?;
                IDS.lock().unwrap().push(id.0);
            }
        }
        let mut loaded_chunks = Vec::new();
        client.write(ServerPacket::SpawnPosition { x: (3.0f64 * 32.0) as i32, y: (20.0f64 * 32.0) as i32, z: (5.0f64 * 32.0) as i32})?;
        let spawnchunk = ChunkCoords { x: 0, z: 0 };
        loaded_chunks.push(spawnchunk.clone());
        self.world.to_packets_new(&mut client.packet_send_sender, &mut Vec::new()).unwrap();
        client.write(ServerPacket::PlayerPositionAndLook { x: 3.0, stance: 67.240000009536743, y: 20.0, z: 5.0, yaw: 0.0, pitch: 0.0, on_ground: false})?;
        //client.write(ServerPacket::PlayerTeleport { player_id: -1, position: Position::from_pos(64, 128, 64)})?;
        let list = self.players.clone();
        let mut players = self.players.0.borrow_mut();
        let pos = Position::from_pos(3.0, 20.0, 5.0);
        players.insert(id, Arc::new(RefCell::new(Player { username: client.username.clone(), id, position: pos.clone(), recv_packets_recv: client.recv_packets_recv.clone(), packet_send_sender: client.packet_send_sender.clone(), rendered_players: HashMap::new(), perm_level: 1, players_list: list, crouching: false, health: 20, last_health: 20, last_position: pos.clone(), dead: false, world: 0, last_void_dmg: std::time::Instant::now(), inventory: Inventory::new(), last_inventory: Inventory::new(), held_slot: 0, last_dmg_type: DamageType::None, last_transaction_id: 0, current_cursored_item: None, loaded_chunks: loaded_chunks, has_loaded_before: Vec::new(), since_last_attack: std::time::Instant::now(), mining_block: MiningBlockData::default(), rendered_entities: HashMap::new(), chatbox: Vec::new()})));
        Ok(())
    }
}