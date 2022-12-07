use crate::block_entity;
use crate::block_entity::BlockEntity;
use crate::block_entity::BlockEntityLoader;
use crate::block_entity::SignData;
use crate::commands::*;
use crate::configuration::OpManager;
use crate::configuration::CONFIGURATION;
use crate::ecs::entities::falling_block::FallingBlockEntityBuilder;
use crate::ecs::entities::falling_block::FallingBlockEntityData;
use crate::ecs::entities::item::ItemEntityBuilder;
use crate::ecs::entities::living::Health;

use crate::ecs::entities::living::hostile::zombie::ZombieEntityBuilder;
use crate::ecs::entities::player::ChatMessage;
use crate::ecs::entities::player::Chatbox;
use crate::ecs::entities::player::Gamemode;
use crate::ecs::entities::player::Player;
use crate::ecs::entities::player::PlayerBuilder;
use crate::ecs::entities::player::Username;
use crate::ecs::systems::world::light::LightPropagationManager;
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
use crate::events::WeatherChangeEvent;
use crate::item::item::ItemRegistry;
use crate::item::stack::ItemStack;
use crate::item::stack::ItemStackType;
use crate::VERSION;
//use crate::game::events::*;
use crate::network::ids::NetworkID;
use crate::objects::Resources;
use crate::physics::Physics;
use crate::player_dat::PlayerDat;
use crate::plugins::PluginManager;
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
use crate::status_effects::poison::PoisonEffect;
use crate::status_effects::StatusEffectsManager;
use crate::translation::TranslationManager;
use crate::world::chunk_entities::ChunkEntities;
use crate::world::chunk_map::chunk_relative_pos;
use crate::world::chunks::BlockState;
use crate::world::mcregion::MCRegionLoader;
use crate::world::LevelDat;
use crate::world::World;
use ahash::AHashMap;
//pub mod aa_bounding_box;
//use items::*;
use flume::{Receiver, Sender};
use hecs::Entity;
use hecs::EntityBuilder;
use hecs::NoSuchEntity;
use itertools::Itertools;
use j4rs::Jvm;

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rustc_hash::FxHashMap;
use std::any::Any;
use std::cell::RefCell;
use std::cell::{Ref, RefMut};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::fs;
use std::mem;
use std::ops::Deref;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
pub type RefContainer<T> = Arc<RefCell<T>>;

#[derive(Copy, Clone, PartialEq, Debug, Hash)]
pub struct BlockPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub world: i32,
}

impl Eq for BlockPosition {}
impl Display for BlockPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.x, self.y, self.z)
    }
}
impl BlockPosition {
    pub fn within_border(&self, border: i32) -> bool {
        let x = self.x;
        let z = self.z;
        if x > border {
            return false;
        }
        if x < -border {
            return false;
        }
        if z > border {
            return false;
        }
        if z < -border {
            return false;
        }
        true
    }
    pub fn offset(&self, x: i32, y: i32, z: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y + y,
            z: self.z + z,
            ..*self
        }
    }
    pub fn new(x: i32, y: i32, z: i32, world: i32) -> Self {
        Self { x, y, z, world }
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
            world: self.world,
        }
    }
    /// Get +x,-x,+y,-y,+z,-z offsets from a block position.
    pub fn all_directions(&self) -> [BlockPosition; 6] {
        let mut list = [BlockPosition::default(); 6];
        let mut clone = *self;
        clone.x += 1;
        list[0] = clone;

        let mut clone = *self;
        clone.y += 1;
        list[1] = clone;

        let mut clone = *self;
        clone.z += 1;
        list[2] = clone;

        let mut clone = *self;
        clone.x -= 1;
        list[3] = (clone);

        let mut clone = *self;
        clone.y -= 1;
        list[4] = (clone);

        let mut clone = *self;
        clone.z -= 1;
        list[5] = (clone);
        list
    }
}
impl std::default::Default for BlockPosition {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            world: 0,
        }
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
    pub world: i32,
    pub update: bool,
}
impl From<Position> for BlockPosition {
    fn from(other: Position) -> Self {
        Self {
            x: other.x.floor() as i32,
            y: other.y.floor() as i32,
            z: other.z.floor() as i32,
            world: other.world,
        }
    }
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
            world: bpos.world,
            update: true,
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
            world: 0,
            update: true,
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
    pub fn from_pos(x: f64, y: f64, z: f64, world: i32) -> Self {
        Position {
            x,
            stance: 67.240000009536743,
            y,
            z,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            world,
            update: true,
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
    /// Position to chunk coordinates
    pub fn to_chunk_coords(&self) -> ChunkCoords {
        let x = self.x as i32 / 16;
        let z = self.z as i32 / 16;
        ChunkCoords {
            x,
            z,
            world: self.world,
        }
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
    Fire,
    Player { damager: String },
    None,
    Fall,
    Mob { damager: String },
    Drown,
}
impl DamageType {
    pub fn string(&self) -> String {
        match self {
            DamageType::Void => "fell into the void.".to_string(),
            DamageType::Fall => "fell to their doom.".to_string(),
            DamageType::Fire => "burned to death.".to_string(),
            DamageType::Player { damager } => {
                format!("was slain by {}", damager)
            }
            _ => "died".to_string(),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Hash)]
pub struct ChunkCoords {
    pub x: i32,
    pub z: i32,
    pub world: i32,
}
impl Display for ChunkCoords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.z)
    }
}
impl Eq for ChunkCoords {}
//use num::num_integer::roots::Roots;
impl ChunkCoords {
    pub fn new(x: i32, z: i32, world: i32) -> Self {
        Self { x, z, world }
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
            world: position.world,
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
    pub tasks: FxHashMap<u128, Vec<Arc<Box<dyn Fn(&mut Game) -> Option<u128>>>>>,
}
impl Scheduler {
    pub fn new() -> Self {
        Self {
            tasks: FxHashMap::default(),
        }
    }
    pub fn schedule_task(
        &mut self,
        ticks: u128,
        func: impl Fn(&mut Game) -> Option<u128> + 'static,
    ) {
        self.tasks
            .entry(ticks)
            .or_default()
            .push(Arc::new(Box::new(func)));
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
    pub console_entity: Entity,
    pub scheduler: Rc<RefCell<Scheduler>>,
    pub tps: f64,
    pub rng: ChaCha8Rng,
    pub plugins: Rc<RefCell<PluginManager>>,
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
    pub fn can_see_sky(&self, pos: BlockPosition) -> bool {
        if let Some(world) = self.worlds.get(&pos.world) {
            let posoff = chunk_relative_pos(pos);
            let height = world
                .chunk_map
                .chunk_at(pos.to_chunk_coords())
                .unwrap()
                .heightmaps
                .light_blocking
                .height(posoff.0 as usize, posoff.2 as usize);
            if let Some(height) = height {
                if pos.y > height as i32 {
                    return true;
                }
            }
        }
        false
    }
    pub fn entity_by_network_id(&self, id: NetworkID) -> Option<Entity> {
        for (current_entity, entity_net_id) in self.ecs.query::<&NetworkID>().iter() {
            if *entity_net_id == id {
                return Some(current_entity);
            }
        }
        None
    }
    pub fn schedule_at(
        &mut self,
        ticks: u128,
        callback: impl Fn(&mut Game) -> Option<u128> + 'static,
    ) {
        let sch = self.scheduler.clone();
        let mut scheduler = sch.borrow_mut();
        scheduler.schedule_task(ticks, callback);
    }
    pub fn schedule_next_tick(&mut self, callback: impl Fn(&mut Game) -> Option<u128> + 'static) {
        let sch = self.scheduler.clone();
        let mut scheduler = sch.borrow_mut();
        scheduler.schedule_task(self.ticks + 1, callback);
    }
    pub fn run_scheduler(&mut self) {
        let ticks = self.ticks;
        let mut to_reschedule = Vec::new();
        let sch = self.scheduler.clone();
        let mut scheduler = sch.borrow_mut();
        if !scheduler.tasks.contains_key(&ticks) {
            return;
        }
        let mut tasks = scheduler.tasks.remove(&ticks).unwrap();
        drop(scheduler);
        for task in tasks {
            if let Some(time) = task(self) {
                to_reschedule.push((time, task.clone()));
            }
        }

        let mut scheduler = sch.borrow_mut();
        for task in to_reschedule {
            scheduler.tasks.entry(task.0).or_default().push(task.1);
        }
    }
    pub fn entity_for_username(&self, username: &str) -> Option<Entity> {
        for (entity, name) in self.ecs.query::<&Username>().iter() {
            if name.0 == username {
                return Some(entity);
            }
        }
        None
    }
    pub fn block_entity_at(&self, position: BlockPosition) -> Option<Entity> {
        for (entity, block_entity) in self.ecs.query::<&BlockEntity>().iter() {
            if block_entity.0 == position && block_entity.1 == position.world {
                return Some(entity);
            }
        }
        None
    }
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
    pub fn execute_command(
        &mut self,
        server: &mut Server,
        command: &str,
        executor: Entity,
    ) -> anyhow::Result<usize> {
        let commands = self.commands.clone();
        let mut commands = commands.borrow_mut();
        commands.execute(self, server, executor, command)
    }
    /// Checks if the block at the given position is solid.
    pub fn is_solid_block(&self, pos: BlockPosition) -> bool {
        if let Some(world) = self.worlds.get(&pos.world) {
            if let Some(block) = world.block_at(pos) {
                return block.is_solid();
            }
        }
        false
    }

    /// Checks if the block at the given position is solid.
    pub fn is_overwritable_block(&self, pos: BlockPosition) -> bool {
        if let Some(world) = self.worlds.get(&pos.world) {
            if let Some(block) = world.block_at(pos) {
                return block.is_overwritable();
            }
        }
        false
    }

    pub fn block_meta_at(&self, pos: BlockPosition, world: i32) -> u8 {
        if let Some(state) = self.block(pos, world) {
            state.b_metadata
        } else {
            0
        }
    }
    pub fn can_be_placed_at(&self, pos: BlockPosition) -> bool {
        if let Some(world) = self.worlds.get(&pos.world) {
            world.can_be_placed_at(pos)
        } else {
            false
        }
    }
    pub fn block_id_at(&self, pos: BlockPosition) -> u8 {
        if let Some(state) = self.block(pos, pos.world) {
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
        nlh: bool,
        update_self: bool,
    ) -> bool {
        let world = match self.worlds.get_mut(&world_id) {
            Some(w) => w,
            None => {
                return false;
            }
        };
        let v = if let Ok(mut light) = self.objects.get_mut::<LightPropagationManager>() {
            let was_successful = world.set_block_at(&mut light, pos, block, nlh);
            let mut event = BlockChangeEvent::single(pos, world_id);
            event.update_neighbors = update_neighbors;
            event.update_self = update_self;
            drop(light);
            if was_successful {
                //log::info!("Propagating event for {:?}", pos);
                self.ecs.insert_event(event);
            }
            was_successful
        } else {
            false
        };

        let origin_state = block;
        if v && update_neighbors {
            for face in Face::all_faces() {
                //log::info!("Face: {:?} w original: {:?}", face, block);
                let block = face.offset(pos);
                //log::info!("Offset: {:?}", block);
                let block_state = match self.block(block, world_id) {
                    Some(b) => b,
                    None => {
                        continue;
                    }
                };
                // TODO: Should we have different meta values update different states? Wool etc
                if block_state.b_type == 0 {
                    continue;
                }
                let block_type = ItemRegistry::global().get_block(block_state.b_type);
                if let Some(block_type) = block_type {
                    //log::info!("Updating neighbor at {:?} from {:?}", block, origin);
                    if let Err(_) = block_type.neighbor_update(
                        world_id,
                        self,
                        block,
                        block_state,
                        face,
                        origin_state,
                    ) {
                        // TODO handle
                    }
                    //to_update.push((block, world));
                }
            }
        }

        v
    }
    /// Sets the block at the given position.
    ///
    /// Triggers necessary `BlockChangeEvent`s.
    pub fn set_block(&mut self, pos: BlockPosition, block: BlockState, world_id: i32) -> bool {
        self.set_block_nb(pos, block, world_id, true, false, true)
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
                .insert_entity_event(entity, PlayerSpawnEvent(true))
                .unwrap();
        }
    }
    pub fn new(mut plugins: PluginManager) -> Self {
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
        let dir = PathBuf::from(&CONFIGURATION.level_name);
        let mut x = dir.clone();
        x.push("players/");
        fs::create_dir_all(x).unwrap();
        let mut level_dat = dir.clone();
        level_dat.push("level.dat");
        let level_dat = LevelDat::from_file(level_dat, 0);
        let level_dat = Arc::new(Mutex::new(level_dat));
        let mut hell = dir.clone();
        hell.push("DIM-1");
        let mut sky = dir.clone();
        sky.push("DIM1");
        let main_world =
            crate::world::World::from_file_mcr(&CONFIGURATION.level_name, 0, level_dat.clone())
                .unwrap();
        worlds.insert(0i32, main_world);
        worlds.insert(
            -1i32,
            crate::world::World::from_file_mcr(hell, -1, level_dat.clone()).unwrap(),
        );
        worlds.insert(
            1i32,
            crate::world::World::from_file_mcr(sky, 1, level_dat.clone()).unwrap(),
        );
        let mut commands = CommandSystem::new();
        commands.register(Command::new(
            "deop",
            "deop a player",
            4,
            vec![CommandArgumentTypes::String],
            Box::new(|game, server, executor, args| {
                let string = args.get::<String>(0)?.to_string();
                let name = game.ecs.get::<Username>(executor)?.deref().clone();
                game.objects.get_mut::<OpManager>()?.remove_op(&string);
                for (_, (username, perm_level)) in
                    game.ecs.query::<(&Username, &mut PermissionLevel)>().iter()
                {
                    if username.0 == string {
                        perm_level.0 = 1;
                    }
                }
                game.broadcast_to_ops(&name.0, &format!("De-opped {}", string));
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "tp",
            "tp a player to another",
            4,
            vec![CommandArgumentTypes::String, CommandArgumentTypes::String],
            Box::new(|game, server, executor, args| {
                let p1n = args.get::<String>(0)?.to_string();
                let p2n = args.get::<String>(1)?.to_string();
                let name = game.ecs.get::<Username>(executor)?.deref().clone();
                let p1 = game.entity_for_username(&p1n);
                let p2 = game.entity_for_username(&p2n);
                if p2.is_none() || p1.is_none() {
                    Ok(3)
                } else {
                    let p1 = p1.unwrap();
                    let p2 = p2.unwrap();
                    let p2pos = *game.ecs.get::<Position>(p2)?;
                    *game.ecs.get_mut::<Position>(p1)? = p2pos;
                    game.broadcast_to_ops(&name.0, &format!("Teleporting {} to {}", p1n, p2n));
                    Ok(0)
                }
            }),
        ));
        commands.register(Command::new(
            "op",
            "op a player",
            4,
            vec![CommandArgumentTypes::String],
            Box::new(|game, server, executor, args| {
                let string = args.get::<String>(0)?.to_string();
                let name = game.ecs.get::<Username>(executor)?.deref().clone();
                game.objects.get_mut::<OpManager>()?.add_op(string.clone());
                for (_, (username, perm_level)) in
                    game.ecs.query::<(&Username, &mut PermissionLevel)>().iter()
                {
                    if username.0.to_lowercase() == string.to_lowercase() {
                        perm_level.0 = 4;
                    }
                }
                game.broadcast_to_ops(&name.0, &format!("Opped {}", string));
                Ok(0)
            }),
        ));
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
            "say",
            "say message",
            4,
            vec![CommandArgumentTypes::StringRest],
            Box::new(|game, server, executor, args| {
                let executor = game.ecs.entity(executor)?;
                let name = executor.get::<Username>()?.0.clone();
                let msg = args.get::<Vec<String>>(0)?;
                game.broadcast_chat(format!("[{}] {}", name, msg.iter().join(" ")));
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
                let pos = *executor.get::<Position>()?;
                let time = args.get::<i32>(0)?;
                game.worlds
                    .get_mut(&pos.world)
                    .unwrap()
                    .level_dat
                    .lock()
                    .time = *time as i64;
                //game.time = *time as i64;
                let name = executor.get::<Username>()?.0.clone();
                game.broadcast_to_ops(
                    &name,
                    &format!("Set the time in DIM:{} to {}", pos.world, time),
                );
                Ok(0)
            }),
        ));

        commands.register(Command::new(
            "toggledownfall",
            "toggledownfall",
            4,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let (pos, name) = {
                    let executor = game.ecs.entity(executor)?;
                    (
                        *executor.get::<Position>()?,
                        executor.get::<Username>()?.0.clone(),
                    )
                };
                {
                    let mut ldat = game.worlds.get_mut(&pos.world).unwrap().level_dat.lock();
                    ldat.raining ^= true;
                    game.ecs.insert_event(WeatherChangeEvent {
                        world: pos.world,
                        is_raining: ldat.raining,
                        is_thundering: ldat.thundering,
                    });
                }

                //game.time = *time as i64;
                game.broadcast_to_ops(&name, &format!("Toggled downfall"));
                Ok(0)
            }),
        ));

        commands.register(Command::new(
            "skylight",
            "skylight at pos",
            4,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let pos = executor.get::<Position>()?;
                let world = pos.world;
                let pos: BlockPosition = pos.deref().clone().into();
                if let Some(state) = game.block(pos, world) {
                    let mut chatbox = executor.get_mut::<Chatbox>()?;
                    chatbox.send_message(format!("Skylight: {}", state.b_skylight).into());
                    chatbox.send_message(format!("Block light: {}", state.b_light).into());
                }
                if let Some(world) = game.worlds.get(&world) {
                    let mut chatbox = executor.get_mut::<Chatbox>()?;
                    let posoff = chunk_relative_pos(pos);
                    chatbox.send_message(
                        format!(
                            "Heightmap for {:?}: {:?}",
                            pos,
                            world
                                .chunk_map
                                .chunk_at(pos.to_chunk_coords())
                                .unwrap()
                                .heightmaps
                                .light_blocking
                                .height(posoff.0 as usize, posoff.2 as usize)
                        )
                        .into(),
                    );
                    for (idx, section) in world
                        .chunk_map
                        .chunk_at(pos.to_chunk_coords())
                        .unwrap()
                        .sections().iter().enumerate()
                    {
                        if let Some(section) = section {
                            if section.lights().len() < 10 {
                                chatbox.send_message(
                                    format!(
                                        "Lights in section #{}: {:?}",
                                        idx, section.lights()
                                    )
                                    .into(),
                                );
                            } else {
                                chatbox.send_message(
                                    format!(
                                        "Lights in section #{}: [{} lights]",
                                        idx, section.lights().len()
                                    )
                                    .into(),
                                );
                            }
                        }
                    }
                }
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "seed",
            "seed",
            1,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let mut chatbox = executor.get_mut::<Chatbox>()?;
                let world = executor.get::<Position>()?.world;
                let world = game
                    .worlds
                    .get(&world)
                    .ok_or(anyhow::anyhow!("Does not exist"))?;
                chatbox
                    .send_message(format!("Seed: [{}]", world.level_dat.lock().world_seed).into());
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "help",
            "help",
            1,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let mut chatbox = executor.get_mut::<Chatbox>()?;
                chatbox.send_message("Help -- TODO".into());
                chatbox.send_message("Op yourself with `op`.".into());
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "jvm_debug",
            "prints jvm debug info to the console",
            5,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let jvm = Jvm::attach_thread()?;
                jvm.invoke_static("com.exopteron.RustInterface", "printDebug", &[])?;
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "pvelocity",
            "pvelocity",
            4,
            vec![
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
            ],
            Box::new(|game, server, executor, mut args| {
                let xv = *args.get::<i32>(0)?;
                let yv = *args.get::<i32>(1)?;
                let zv = *args.get::<i32>(2)?;
                let executor = game.ecs.entity(executor)?;
                let pos = *executor.get::<Position>()?;
                let mut physics = executor.get_mut::<Physics>()?;
                physics.add_velocity(xv as f64, yv as f64, zv as f64);
                Ok(0)
            }),
        ));

        commands.register(Command::new(
            "setblock",
            "set block",
            4,
            vec![
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
            ],
            Box::new(|game, server, executor, mut args| {
                let x = *args.get::<i32>(0)?;
                let y = *args.get::<i32>(1)?;
                let z = *args.get::<i32>(2)?;
                let block = *args.get::<i32>(3)?;
                let meta = *args.get::<i32>(4)?;
                let executor = game.ecs.entity(executor)?;
                let world = executor.get::<Position>()?.world;
                let pos = BlockPosition::new(x, y, z, world);
                game.set_block(pos, BlockState::new(block as u8, meta as u8), world);
                Ok(0)
            }),
        ));

        commands.register(Command::new(
            "zombie",
            "zombie",
            4,
            vec![
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
            ],
            Box::new(|game, server, executor, mut args| {
                let count = *args.get::<i32>(0)?;
                let xv = *args.get::<i32>(1)?;
                let yv = *args.get::<i32>(2)?;
                let zv = *args.get::<i32>(3)?;
                let executor = game.ecs.entity(executor)?;
                let pos = *executor.get::<Position>()?;
                for _ in 0..count {
                    let mut builder = game.create_entity_builder(pos, EntityInit::Mob);
                    ZombieEntityBuilder::build(Some(pos), 20, &mut builder);
                    let entity = game.spawn_entity(builder);
                    let mut physics = game.ecs.get_mut::<Physics>(entity)?;
                    physics.add_velocity(xv as f64, yv as f64, zv as f64);
                }
                Ok(0)
            }),
        ));

        commands.register(Command::new(
            "signs",
            "signs",
            4,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let mut c = 0;
                for (k, (a, b)) in game.ecs.query::<(&mut SignData, &BlockEntity)>().iter() {
                    c += 1;
                }
                let executor = game.ecs.entity(executor)?;
                let mut chatbox = executor.get_mut::<Chatbox>()?;
                chatbox.send_message(format!("num: {}", c).into());
                Ok(0)
            }),
        ));
        // game.ecs.query::<(&mut SignData, &BlockEntity)>().iter()
        commands.register(Command::new(
            "sanddrop",
            "sanddrop",
            4,
            vec![
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
            ],
            Box::new(|game, server, executor, mut args| {
                let xv = *args.get::<i32>(0)?;
                let yv = *args.get::<i32>(1)?;
                let zv = *args.get::<i32>(2)?;
                let executor = game.ecs.entity(executor)?;
                let pos = *executor.get::<Position>()?;
                let entity =
                    FallingBlockEntityBuilder::build(game, pos, FallingBlockEntityData::Sand);
                let entity = game.spawn_entity(entity);
                let mut physics = game.ecs.get_mut::<Physics>(entity)?;
                physics.add_velocity(xv as f64, yv as f64, zv as f64);
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "itemdrop",
            "itemdrop",
            4,
            vec![
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
                CommandArgumentTypes::Int,
            ],
            Box::new(|game, server, executor, mut args| {
                let xv = *args.get::<i32>(0)?;
                let yv = *args.get::<i32>(1)?;
                let zv = *args.get::<i32>(2)?;
                let executor = game.ecs.entity(executor)?;
                let pos = *executor.get::<Position>()?;
                let entity = ItemEntityBuilder::build(game, pos, ItemStack::new(1, 1, 0));
                let entity = game.spawn_entity(entity);
                let mut physics = game.ecs.get_mut::<Physics>(entity)?;
                physics.add_velocity(xv as f64, yv as f64, zv as f64);
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "poison",
            "poison",
            1,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                executor
                    .get_mut::<StatusEffectsManager>()?
                    .add_effect(PoisonEffect::new(1, 60));
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "ver",
            "ver",
            1,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let mut chatbox = executor.get_mut::<Chatbox>()?;
                chatbox.send_message(format!("exo-beta-server-v{}", VERSION).into());
                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "dim",
            "dim",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let arg = args.get::<i32>(0)?;
                let name = executor.get::<Username>()?.deref().clone();
                if game.worlds.contains_key(arg) {
                    executor.get_mut::<Position>()?.world = *arg;

                    game.broadcast_to_ops(&name.0, &format!("Sending self to dimension {}", *arg));

                    //executor.get_mut::<Position>()?.world = *arg;
                    Ok(0)
                } else {
                    Ok(3)
                }
            }),
        ));

        commands.register(Command::new(
            "die",
            "die",
            4,
            vec![CommandArgumentTypes::Int],
            Box::new(|game, server, executor, mut args| {
                let executor = game.ecs.entity(executor)?;
                let arg = args.get::<i32>(0)?;
                executor
                    .get_mut::<Health>()?
                    .damage(*arg as i16, DamageType::Void);

                Ok(0)
            }),
        ));
        commands.register(Command::new(
            "setworldspawn",
            "set spawn",
            4,
            vec![],
            Box::new(|game, server, executor, mut args| {
                let entity_ref = game.ecs.entity(executor)?;
                let position = entity_ref.get::<Position>()?;
                let world = position.world;
                let blockpos: BlockPosition = position.deref().clone().into();
                game.worlds
                    .get_mut(&world)
                    .ok_or(anyhow::anyhow!("No such world"))?
                    .level_dat
                    .lock()
                    .spawn_point = blockpos;
                let name = entity_ref.get::<Username>()?.0.clone();
                drop(entity_ref);
                drop(position);
                game.broadcast_to_ops(
                    &name,
                    &format!("Set spawn point for world {} to {}", world, blockpos),
                );
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
                let arg = args.get::<i32>(0)?;
                let id = Gamemode::from_id(*arg as u8);
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
                //chatbox.send_message(format!("Game time: {}", game.time).into());
                chatbox.send_message(format!("Game tick count: {}", game.ticks).into());
                chatbox.send_message(
                    format!("Block entities: {}", game.ecs.count::<BlockEntity>()).into(),
                );
                chatbox.send_message(
                    format!("Player entities: {}", game.ecs.count::<Player>()).into(),
                );
                chatbox.send_message(
                    format!(
                        "Players: {}/{}",
                        server.player_count.get(),
                        server.player_count.get_max()
                    )
                    .into(),
                );
                chatbox.send_message(format!("MOTD: {}", CONFIGURATION.server_motd).into());
                chatbox.send_message(
                    format!("View distance: {}", CONFIGURATION.chunk_distance).into(),
                );
                chatbox.send_message(format!("Worlds:").into());
                for (id, world) in game.worlds.iter() {
                    chatbox.send_message(
                        format!(
                            "World {} - {} loaded chunks",
                            id,
                            world.loaded_chunks().len()
                        )
                        .into(),
                    );
                }
                if let Ok(v) = executor.get::<Position>() {
                    chatbox.send_message(format!("You are at: {:?}", v.world).into());
                }

                drop(chatbox);
                game.broadcast_to_ops(&name, &format!("Acquired debug info"));
                Ok(0)
            }),
        ));
        plugins.register_commands(&mut commands);
        //let temp_highest_point = 65;
        let mut game = Self {
            objects: objects,
            systems: Arc::new(RefCell::new(SystemExecutor::new())),
            worlds,
            ecs: Ecs::new(),
            ticks: 0,
            entity_builder: EntityBuilder::new(),
            entity_spawn_callbacks: Vec::new(),
            chunk_entities: ChunkEntities::default(),
            commands: Rc::new(RefCell::new(commands)),
            console_entity: Entity::from_bits(0),
            scheduler: Rc::new(RefCell::new(Scheduler::new())),
            tps: 0.0,
            rng: ChaCha8Rng::from_entropy(),
            plugins: Rc::new(RefCell::new(plugins)),
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
                    /*                     let client = server
                        .clients
                        .get(&player.get::<NetworkID>().unwrap())
                        .unwrap();
                    client.disconnect("Bad packet error"); */
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
        // if let Ok(nid) = self.ecs.get::<NetworkID>(entity) {
        //     IDS.lock().unwrap().push(nid.0);
        // }
        self.ecs.defer_despawn(entity);
        self.ecs.insert_entity_event(entity, EntityRemoveEvent)
    }
    fn accept_player(&mut self, server: &mut Server, id: NetworkID) -> anyhow::Result<()> {
        let clients = &mut server.clients;
        let client = clients
            .get_mut(&id)
            .ok_or(anyhow::anyhow!("Client does not exist"))?;
        let world = self.worlds.get(&0).unwrap();
        let mut pd_dir = world.world_dir.clone();
        let mut name = client.username().to_string();
        name = name.replace("\\", "");
        name = name.replace("/", "");
        name = name.replace("..", "");
        pd_dir.push("players/".to_owned() + &name + ".dat");
        let player_dat = PlayerDat::from_file(&pd_dir);
        let pos: Position = match player_dat {
            Ok(ref dat) => Position {
                x: dat.pos[0],
                stance: 67.0,
                y: dat.pos[1],
                z: dat.pos[2],
                yaw: dat.rotation[0],
                pitch: dat.rotation[1],
                on_ground: dat.on_ground,
                world: dat.dimension,
                update: true,
            },
            Err(_) => world.level_dat.lock().spawn_point.into(),
        };

        let world: &World = match player_dat {
            Ok(ref dat) => self.worlds.get(&dat.dimension).unwrap(),
            Err(_) => world,
        };

        log::info!(
            "{} logging in with entity ID {} at {}",
            client.username(),
            id.0,
            pos
        );
        println!("World id {}", world.id as i8);
        if let Ok(ref player_dat) = player_dat {
            client.write(ServerPlayPacket::LoginRequest(LoginRequest {
                entity_id: id.0,
                not_used: String16("ExoBetaServer".to_owned()),
                map_seed: world.level_dat.lock().world_seed as i64,
                server_mode: player_dat.game_type as i32,
                dimension: world.id as i8,
                difficulty: 0,
                world_height: 128,
                max_players: server.player_count.get_max() as u8,
            }))?;
        } else {
            client.write(ServerPlayPacket::LoginRequest(LoginRequest {
                entity_id: id.0,
                not_used: String16("ExoBetaServer".to_owned()),
                map_seed: world.level_dat.lock().world_seed as i64,
                server_mode: CONFIGURATION.default_gamemode as i32,
                dimension: world.id as i8,
                difficulty: 0,
                world_height: 128,
                max_players: server.player_count.get_max() as u8,
            }))?;
        }
        client.set_is_raining(world.level_dat.lock().raining);

        let spos: Position = world.level_dat.lock().spawn_point.into();
        client.write(ServerPlayPacket::SpawnPosition(SpawnPosition {
            x: spos.x.round() as i32,
            y: spos.y.round() as i32,
            z: spos.z.round() as i32,
        }))?;
        let gamemode;
        if let Ok(ref pdat) = player_dat {
            gamemode = Gamemode::from_id(pdat.game_type as u8);
        } else {
            gamemode = Gamemode::from_id(CONFIGURATION.default_gamemode);
        }
        let player = PlayerBuilder::create(
            self,
            Username(client.username().to_owned()),
            pos,
            id,
            gamemode.unwrap(),
            &player_dat,
        );
        let e = self.spawn_entity(player);
        if player_dat.is_err() {
            let e = self.ecs.entity(e)?;
            let player_dat = PlayerDat::from_entity(&e)?;
            player_dat.to_file(pd_dir)?;
        }
        log::info!("HERE");
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
    game.broadcast_chat(format!("§e{}", message));
}
