use crate::block_entity;
use crate::block_entity::BlockEntity;
use crate::block_entity::BlockEntityLoader;
use crate::commands::*;
use crate::configuration::CONFIGURATION;
use crate::ecs::entities::living::Health;
use crate::ecs::entities::player::ChatMessage;
use crate::ecs::entities::player::Chatbox;
use crate::ecs::entities::player::CurrentWorldInfo;
use crate::ecs::entities::player::Gamemode;
use crate::ecs::entities::player::Player;
use crate::ecs::entities::player::PlayerBuilder;
use crate::ecs::entities::player::Username;
use crate::ecs::systems::SysResult;
use crate::ecs::systems::SystemExecutor;
use crate::ecs::Ecs;
use crate::ecs::EntityRef;
use crate::ecs::HasEcs;
use crate::ecs::HasResources;
use crate::entities::EntityInit;
use crate::events::block_change::BlockChangeEvent;
use crate::events::EntityCreateEvent;
use crate::events::EntityRemoveEvent;
use crate::events::PlayerJoinEvent;
use crate::events::PlayerSpawnEvent;
//use crate::game::events::*;
use crate::network::ids::NetworkID;
use crate::network::ids::IDS;
use crate::objects::Resources;
use crate::protocol::io::String16;
use crate::protocol::packets::server::LoginRequest;
use crate::protocol::packets::server::PlayerPositionAndLook;
use crate::protocol::packets::server::SpawnPosition;
use crate::protocol::packets::Face;
use crate::protocol::packets::SoundEffectKind;
use crate::protocol::ServerLoginPacket;
use crate::protocol::ServerPlayPacket;
use crate::server::Client;
use crate::server::Server;
use crate::translation::TranslationManager;
use crate::world::chunk_entities::ChunkEntities;
use crate::world::chunks::BlockState;
use crate::world::mcregion::MCRegionLoader;
use crate::world::World;
use ahash::AHashMap;
//pub mod aa_bounding_box;
//use items::*;
use flume::{Receiver, Sender};
use hecs::Entity;
use hecs::EntityBuilder;
use hecs::NoSuchEntity;
use once_cell::sync::Lazy;
use std::any::Any;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::mem;
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
        ChunkCoords {
            x: self.x >> 4,
            z: self.z >> 4,
        }
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
impl From<BlockPosition> for Position {
    fn from(bpos: BlockPosition) -> Self {
        Self {
            x: bpos.x as f64,
            stance: 0.0,
            y: bpos.y as f64,
            z: bpos.z as f64,
            yaw: 0.,
            pitch: 0.,
            on_ground: false,
        }
    }
}
#[derive(Copy, Clone)]
pub struct Block {
    pub position: BlockPosition,
    pub block: crate::world::chunks::BlockState,
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
        write!(f, "({:.3}, {:.3}, {:.3})", self.x, self.y, self.z)
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
impl DamageType {
    pub fn string(&self) -> String {
        match self {
            DamageType::Void => "fell into the void.",
            DamageType::Fall => "fell to their doom.",
            _ => "died",
        }.to_string()
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
}
impl Display for ChunkCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}
impl Eq for ChunkCoords {}
//use num::num_integer::roots::Roots;
impl ChunkCoords {
    pub fn new(x: i32, z: i32) -> Self {
        Self { x, z }
    }
    /// Computes the Manhattan distance from this chunk to another.
    pub fn manhattan_distance_to(self, other: ChunkCoords) -> i32 {
        (self.x - other.x).abs() + (self.z - other.z).abs()
    }

    /// Computes the squared Euclidean distance (in chunks) between `self` and `other`.
    pub fn distance_squared_to(self, other: ChunkCoords) -> i32 {
        (self.x - other.x).pow(2) + (self.z - other.z).pow(2)
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
pub struct LoadedChunks(pub AHashMap<ChunkCoords, u128>);
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
type EntitySpawnCallback = Box<dyn FnMut(&mut EntityBuilder, &EntityInit)>;
pub struct Game {
    pub objects: Arc<Resources>,
    pub worlds: AHashMap<i32, World>,
    pub ecs: Ecs,
    pub systems: Arc<RefCell<SystemExecutor<Game>>>,
    pub ticks: u128,
    entity_builder: EntityBuilder,
    entity_spawn_callbacks: Vec<EntitySpawnCallback>,
    pub chunk_entities: ChunkEntities,
    pub commands: Rc<RefCell<CommandSystem>>,
    pub time: i64,
    pub console_entity: Entity,
    pub scheduler: Scheduler,
    pub tps: f64,
}
use nbt::*;
use rand::Rng;
impl HasResources for Game {
    fn resources(&self) -> Arc<Resources> {
        self.objects.clone()
    }
}
impl HasEcs for Game {
    fn ecs(&self) -> &Ecs {
        &self.ecs
    }

    fn ecs_mut(&mut self) -> &mut Ecs {
        &mut self.ecs
    }
}
impl Game {
    pub fn is_block_entity_at(&self, position: BlockPosition, world: i32) -> bool {
        for (_, block_entity) in self.ecs.query::<&BlockEntity>().iter() {
            if block_entity.0 == position && block_entity.1 == world {
                return true;
            }
        } 
        false
    }
    pub fn remove_block_entity_at(&mut self, position: BlockPosition, world: i32) -> SysResult {
        let mut to_remove = Vec::new();
        for (entity, block_entity) in self.ecs.query::<&BlockEntity>().iter() {
            if block_entity.0 == position && block_entity.1 == world {
                to_remove.push(entity);
            }
        } 
        for entity in to_remove {
            //log::info!("Removing block entity");
            self.remove_entity(entity)?;
        }
        Ok(())
    }
    pub fn broadcast_to_ops(&mut self, username: &str, message: &str) {
        let epic_msg = format!("§7({}: {})", username, message);
        for (_, (chatbox, perm_level)) in
            self.ecs.query::<(&mut Chatbox, &PermissionLevel)>().iter()
        {
            if perm_level.0 >= 4 {
                chatbox.send_message(epic_msg.clone().into());
            }
        }
    }
    pub fn execute_command(&mut self, server: &mut Server, command: &str, executor: Entity) -> anyhow::Result<usize> {
        let commands = self.commands.clone();
        let mut commands = commands.borrow_mut();
        commands.execute(self, server, executor, command)
    }
    /// Checks if the block at the given position is solid.
    pub fn is_solid_block(&self, pos: BlockPosition, world: i32) -> bool {
        if let Some(world) = self.worlds.get(&world) {
            if let Some(block) = world.block_at(pos) {
                return block.is_solid();
            }
        }
        false
    }
    pub fn block_id_at(&self, pos: BlockPosition, world: i32) -> u8 {
        if let Some(state) = self.block(pos, world) {
            state.b_type
        } else {
            0
        }
    }
    /// Gets the block at the given position.
    pub fn block(&self, pos: BlockPosition, world: i32) -> Option<BlockState> {
        self.worlds.get(&world)?.block_at(pos)
    }
    /// Sets the block at the given position.
    ///
    /// Triggers necessary `BlockChangeEvent`s.
    pub fn set_block_nb(
        &mut self,
        pos: BlockPosition,
        block: BlockState,
        world_id: i32,
        update_neighbors: bool,
    ) -> bool {
        let world = match self.worlds.get_mut(&world_id) {
            Some(w) => w,
            None => {
                return false;
            }
        };
        let was_successful = world.set_block_at(pos, block);
        let mut event = BlockChangeEvent::single(pos, world_id);
        event.update_neighbors = update_neighbors;
        if was_successful {
            //log::info!("Propagating event for {:?}", pos);
            self.ecs.insert_event(event);
        }
        was_successful
    }
    /// Sets the block at the given position.
    ///
    /// Triggers necessary `BlockChangeEvent`s.
    pub fn set_block(&mut self, pos: BlockPosition, block: BlockState, world_id: i32) -> bool {
        self.set_block_nb(pos, block, world_id, true)
    }
    /// Breaks the block at the given position, propagating any
    /// necessary block updates.
    pub fn break_block(&mut self, pos: BlockPosition, world_id: i32) -> bool {
        self.set_block(pos, BlockState::air(), world_id)
    }
    /// Broadcasts a chat message to all entities with
    /// a `ChatBox` component (usually just players).
    pub fn broadcast_chat(&self, message: impl Into<ChatMessage>) {
        let message = message.into();
        for (_, mailbox) in self.ecs.query::<&mut Chatbox>().iter() {
            mailbox.send_message(message.clone());
        }
    }
    /// Spawns an entity and returns its [`Entity`](ecs::Entity) handle.
    ///
    /// Also triggers necessary events, like `EntitySpawnEvent` and `PlayerJoinEvent`.
    pub fn spawn_entity(&mut self, mut builder: EntityBuilder) -> Entity {
        let entity = self.ecs.spawn(builder.build());
        self.entity_builder = builder;
        self.trigger_entity_spawn_events(entity);
        entity
    }

    fn trigger_entity_spawn_events(&mut self, entity: Entity) {
        self.ecs
            .insert_entity_event(entity, EntityCreateEvent)
            .unwrap();
        if self.ecs.get::<Player>(entity).is_ok() {
            self.ecs
                .insert_entity_event(entity, PlayerJoinEvent)
                .unwrap();
            self.ecs
                .insert_entity_event(entity, PlayerSpawnEvent)
                .unwrap();
        }
    }
    pub fn new() -> Self {
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
        let mut objects = Arc::new(Resources::new());
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
        let mut worlds = AHashMap::new();
        worlds.insert(
            0i32,
            crate::world::World::from_file_mcr(&CONFIGURATION.level_name).unwrap(),
        );
        let mut commands = CommandSystem::new();
        commands.register(Command::new(
            "reload",
            "reload",
            4,
            vec![],
            Box::new(|game, server, executor, mut args| {
                *game.objects.get_mut::<TranslationManager>()? = TranslationManager::initialize()?;
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "time",
            "set time",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let time: i32 = args[0].as_int();
                game.time = time as i64;
                let name = executor.get::<Username>()?.0.clone();
                game.broadcast_to_ops(&name, &format!("Set the game time to {}", time));
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "die",
            "die",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let arg: i32 = args[0].as_int();
                executor
                    .get_mut::<Health>()?
                    .damage(arg as i16, DamageType::Void);

                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "gamemode",
            "change gamemode",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let name = executor.get::<Username>()?.0.clone();
                let arg: i32 = args[0].as_int();
                let id = Gamemode::from_id(arg as u8);
                if let Some(gamemode) = id {
                    *executor.get_mut::<Gamemode>()? = gamemode;
                    game.broadcast_to_ops(&name, &format!("Set own game mode to {:?}", gamemode));
                } else {
                    let mut chatbox = executor.get_mut::<Chatbox>()?;
                    chatbox.send_message("§7Invalid gamemode.".into());
                    return Ok(3);
                }
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "debug",
            "debug info",
            4,
            vec![],
            Box::new(|game, server, executor, _| {
                let executor = game.ecs.entity(executor)?;
                let name = executor.get::<Username>()?.0.clone();
                let mut chatbox = executor.get_mut::<Chatbox>()?;
                chatbox.send_message(format!("TPS: {}", game.tps).into());
                chatbox.send_message(format!("Game time: {}", game.time).into());
                chatbox.send_message(format!("Game tick count: {}", game.ticks).into());
                chatbox.send_message(format!("Block entities: {}", game.ecs.count::<BlockEntity>()).into());
                chatbox.send_message(format!("Player entities: {}", game.ecs.count::<Player>()).into());
                chatbox.send_message(format!("Players: {}/{}", server.player_count.get(), server.player_count.get_max()).into());
                chatbox.send_message(format!("MOTD: {}", CONFIGURATION.server_motd).into());
                chatbox.send_message(format!("View distance: {}", CONFIGURATION.chunk_distance).into());
                chatbox.send_message(format!("Worlds:").into());
                for (id, world) in game.worlds.iter() {
                    chatbox.send_message(format!("World {} - {} loaded chunks", id, world.loaded_chunks().len()).into());
                }
                drop(chatbox);
                game.broadcast_to_ops(&name, &format!("Acquired debug info"));
                Ok(0)
            }),
        ));
        let game = Self {
            objects: objects,
            systems: Arc::new(RefCell::new(SystemExecutor::new())),
            worlds,
            ecs: Ecs::new(),
            ticks: 0,
            entity_builder: EntityBuilder::new(),
            entity_spawn_callbacks: Vec::new(),
            chunk_entities: ChunkEntities::default(),
            commands: Rc::new(RefCell::new(commands)),
            time: 0,
            console_entity: Entity::from_bits(0),
            scheduler: Scheduler::new(),
            tps: 0.0,
        };
        //let mut game_globals = GameGlobals { time: 0 };
        //GAME_GLOBAL.set(game_globals);
        game
    }
        /// Creates an entity builder with the default components
    /// for an entity of type `init`.
    pub fn create_entity_builder_posless(&mut self, init: EntityInit) -> EntityBuilder {
        let mut builder = mem::take(&mut self.entity_builder);
        self.invoke_entity_spawn_callbacks(&mut builder, init);
        builder
    }
    /// Creates an entity builder with the default components
    /// for an entity of type `init`.
    pub fn create_entity_builder(&mut self, position: Position, init: EntityInit) -> EntityBuilder {
        let mut builder = mem::take(&mut self.entity_builder);
        builder.add(position);
        self.invoke_entity_spawn_callbacks(&mut builder, init);
        builder
    }
    /// Adds a new entity spawn callback, invoked
    /// before an entity is created.
    ///
    /// This allows you to add components to entities
    /// before they are built.
    pub fn add_entity_spawn_callback(
        &mut self,
        callback: impl FnMut(&mut EntityBuilder, &EntityInit) + 'static,
    ) {
        self.entity_spawn_callbacks.push(Box::new(callback));
    }
    fn invoke_entity_spawn_callbacks(&mut self, builder: &mut EntityBuilder, init: EntityInit) {
        let mut callbacks = mem::take(&mut self.entity_spawn_callbacks);
        for callback in &mut callbacks {
            callback(builder, &init);
        }
        self.entity_spawn_callbacks = callbacks;
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
                if let Err(e) =
                    crate::network::packet::handler::handle_packet(self, server, player, packet)
                {
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
    /// Causes the given entity to be removed on the next tick.
    /// In the meantime, triggers `EntityRemoveEvent`.
    pub fn remove_entity(&mut self, entity: Entity) -> Result<(), NoSuchEntity> {
        self.ecs.defer_despawn(entity);
        self.ecs.insert_entity_event(entity, EntityRemoveEvent)
    }
    fn accept_player(&mut self, server: &mut Server, id: NetworkID) -> anyhow::Result<()> {
        let clients = &mut server.clients;
        let client = clients
            .get_mut(&id)
            .ok_or(anyhow::anyhow!("Client does not exist"))?;
        let pos = Position::from_pos(0., 75., 0.);
        log::info!(
            "{} logging in with entity ID {} at {}",
            client.username(),
            id.0,
            pos
        );
        client.write(ServerPlayPacket::LoginRequest(LoginRequest {
            entity_id: id.0,
            not_used: String16("".to_owned()),
            map_seed: 0,
            server_mode: 1,
            dimension: 0,
            difficulty: 0,
            world_height: 128,
            max_players: server.player_count.get_max() as u8,
        }))?;
        client.write(ServerPlayPacket::SpawnPosition(SpawnPosition {
            x: pos.x.round() as i32,
            y: pos.x.round() as i32,
            z: pos.x.round() as i32,
        }))?;
        let player = PlayerBuilder::create(
            self,
            Username(client.username().to_owned()),
            pos,
            id,
            CurrentWorldInfo::new(0),
        );
        self.spawn_entity(player);
        broadcast_player_join(self, client.username());
        Ok(())
    }
}

fn broadcast_player_join(game: &mut Game, username: &str) {
    let translation = game.objects.get::<TranslationManager>().unwrap();
    let message = translation.translate(
        "multiplayer.player.joined",
        Some(vec![username.to_string()]),
    );
    game.broadcast_chat(message);
}
