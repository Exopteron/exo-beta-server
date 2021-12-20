use crate::commands::*;
use crate::configuration::CONFIGURATION;
use crate::ecs::Ecs;
use crate::ecs::EntityRef;
use crate::ecs::entities::player::CurrentWorldInfo;
use crate::ecs::entities::player::Player;
use crate::ecs::entities::player::PlayerBuilder;
use crate::ecs::entities::player::Username;
use crate::ecs::systems::Systems;
//use crate::game::events::*;
use crate::network::ids::NetworkID;
use crate::network::ids::IDS;
use crate::objects::Objects;
use crate::protocol::ServerLoginPacket;
use crate::protocol::ServerPlayPacket;
use crate::protocol::io::String16;
use crate::protocol::packets::server::LoginRequest;
use crate::protocol::packets::server::PlayerPositionAndLook;
use crate::protocol::packets::server::SpawnPosition;
use crate::server::Client;
use crate::server::Server;
use crate::world::World;
use crate::world::mcregion::MCRegionLoader;
//pub mod aa_bounding_box;
//use items::*;
use flume::{Receiver, Sender};
use hecs::Entity;
use once_cell::sync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::Mutex;
pub type RefContainer<T> = Arc<RefCell<T>>;

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl Eq for BlockPosition {}
impl BlockPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
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
}
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
//use plugins::*;
#[derive(Clone, Debug)]
pub struct CachedCommandData {
    args: Vec<CommandArgumentTypes>,
    root: String,
    desc: String,
}
pub struct Game {
    pub objects: Arc<Objects>,
    pub worlds: HashMap<i32, World>,
    pub ecs: Ecs,
    pub systems: Arc<RefCell<Systems>>,
    pub loaded_chunks: LoadedChunks,
    pub ticks: u128,
}
use nbt::*;
use rand::Rng;
impl Game {
    pub fn new(
        systems: Systems,
    ) -> Self {
        use rand::RngCore;
        //let generator = crate::temp_chunks::FlatWorldGenerator::new(64, 1,1, 1);
        use crate::world::chunks::*;
/*         let mut world: crate::world::chunks::World;
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
        } */
        //world = crate::world::mcregion::temp_from_dir("New World").unwrap();
        let mut objects = Arc::new(Objects::new());
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
        let mut worlds = HashMap::new();
        //worlds.insert(0i32, crate::world::World::from_file_mcr("epicpog").unwrap());
        let game = Self {
            objects: objects,
            systems: Arc::new(RefCell::new(systems)),
            worlds,
            ecs: Ecs::new(),
            ticks: 0,
            loaded_chunks: LoadedChunks(HashMap::new()),
        };
        //let mut game_globals = GameGlobals { time: 0 };
        //GAME_GLOBAL.set(game_globals);
        game
    }
    pub fn insert_object<T>(&mut self, object: T)
    where
        T: 'static,
    {
        Arc::get_mut(&mut self.objects)
            .expect("cyrntly borwd")
            .insert(object);
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
        for (entity, (player, id)) in self.ecs.query::<(&Player, &NetworkID)>().iter() {
            if let Some(cl) = server.clients.get(&id) {
                for packet in cl.recieved_packets() {
                    //log::info!("Got one");
                    packets.push((entity.clone(), packet));
                }
            }
        }
        for (player, packet) in packets {
            if let Err(e) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Err(e) = crate::network::packet::handler::handle_packet(
                    self,
                    server,
                    player,
                    packet,
                ) {
                    let player = self.ecs.entity(player).unwrap();
                    log::error!(
                        "Error handling packet from user {}: {:?}",
                        player.get::<Username>().unwrap().0, /*borrow().username*/
                        e
                    );
                }
            })) {
                //log::info!("Critical error handling packet from user {}.", player.clone().get_username());
/*                 player.write_packet(ServerPacket::Disconnect {
                    reason: String::from("A fatal error has occured."),
                });
                player.remove(Some(format!(
                    "Fatal error handling packet from user {}.",
                    player.clone().get_username()
                ))); */
            }
        }
        Ok(())
    }
    fn accept_player(&mut self, server: &mut Server, id: NetworkID) -> anyhow::Result<()> {
        log::info!("Player {:?}", id);
        let clients = &mut server.clients;
        let mut client = clients.get_mut(&id).ok_or(anyhow::anyhow!("Client does not exist"))?;
        let pos = Position::from_pos(0., 255., 0.);
        log::info!("Pos {:?}", pos);
        client.write(ServerPlayPacket::LoginRequest(LoginRequest { entity_id: id.0, not_used: String16("".to_owned()), map_seed: 0, server_mode: 1, dimension: 0, difficulty: 0, world_height: 128, max_players: 8 }))?;
        client.write(ServerPlayPacket::PlayerPositionAndLook(PlayerPositionAndLook { x: pos.x, stance: 67.240000009536743, y: pos.y, z: pos.z, yaw: pos.yaw, pitch: pos.pitch, on_ground: false }))?;
        client.write(ServerPlayPacket::SpawnPosition(SpawnPosition { x: pos.x.round() as i32, y: pos.x.round() as i32, z: pos.x.round() as i32 }))?;
        PlayerBuilder::build(&mut self.ecs, Username(client.username.clone()), pos, id, CurrentWorldInfo::new(0));
        Ok(())
    }
}
