use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use flume::{Receiver, Sender};
use hecs::{Entity, EntityBuilder};
use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    ecs::{systems::SysResult, Ecs},
    entities::{EntityInit, PreviousPosition},
    game::{ChunkCoords, Game, Position, DamageType},
    network::ids::NetworkID,
    world::{chunks::Chunk, view::View}, item::{inventory::{Inventory, reference::BackingWindow}, window::Window}, commands::PermissionLevel, configuration::{CONFIGURATION, OpManager}, aabb::AABBSize,
};

use super::living::{Health, Hunger, PreviousHealth, PreviousHunger, Regenerator};

pub struct Player;
#[derive(Clone)]
pub struct Username(pub String);
#[derive(Default)]
pub struct ChunkLoadQueue {
    pub chunks: Vec<ChunkCoords>,
}
impl ChunkLoadQueue {
    pub fn add(&mut self, c: &ChunkCoords) {
        if !self.chunks.contains(c) {
            self.chunks.push(*c);
        }
    }
    pub fn remove(&mut self, c: &ChunkCoords) {
        self.chunks.retain(|v| {
            if v != c {
                return true;
            }
            false
        });
    }
    pub fn contains(&self, c: &ChunkCoords) -> bool {
        self.chunks.contains(c)
    }
    pub fn retain(&mut self, f: impl FnMut(&ChunkCoords) -> bool) {
        self.chunks.retain(f);
    }
}
#[derive(Clone, Copy)]
pub struct CurrentWorldInfo {
    pub world_id: i32,
}
impl CurrentWorldInfo {
    pub fn new(world_id: i32) -> Self {
        Self { world_id }
    }
}

#[derive(Clone)]
pub struct ChatMessage(pub Arc<String>);
impl Into<ChatMessage> for String {
    fn into(self) -> ChatMessage {
        ChatMessage::new(self)
    }
}
impl Into<ChatMessage> for &str {
    fn into(self) -> ChatMessage {
        ChatMessage::new(self.to_string())
    }
}
impl ChatMessage {
    pub fn new(msg: String) -> Self {
        Self(Arc::new(msg))
    }
}
pub struct Chatbox {
    messages: Vec<ChatMessage>,
}
impl Chatbox {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
    pub fn send_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }
    pub fn drain(&mut self) -> impl Iterator<Item = ChatMessage> + '_ {
        self.messages.drain(..)
    }
}

/// Whether an entity is sneaking, like in pressing shift.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sneaking(pub bool);

/// A component on players that tracks if they are sprinting or not.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Sprinting(pub bool);
impl Sprinting {
    pub fn new(value: bool) -> Self {
        Sprinting(value)
    }
}
/// The hotbar slot a player's cursor is currently on
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct HotbarSlot(usize);
pub const SLOT_HOTBAR_OFFSET: usize = 36;
impl HotbarSlot {
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    pub fn get(&self) -> usize {
        self.0
    }

    pub fn set(&mut self, id: usize) -> SysResult {
        if id > 8 {
            bail!("invalid hotbar slot id");
        }

        self.0 = id;
        Ok(())
    }
}

/// A gamemode.
#[derive(
    Copy, Clone, Debug, PartialEq, Eq, Hash, FromPrimitive, ToPrimitive,
)]
#[repr(u8)]
pub enum Gamemode {
    Survival = 0,
    Creative = 1,
}

impl Gamemode {
    /// Gets a gamemode from its ID.
    pub fn from_id(id: u8) -> Option<Self> {
        Some(match id {
            0 => Gamemode::Survival,
            1 => Gamemode::Creative,
            _ => return None,
        })
    }
    pub fn id(&self) -> i8 {
        match self {
            Self::Survival => 0,
            Self::Creative => 1
        }
    }
}

/// A player's previous gamemode
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
)]
pub struct PreviousGamemode(pub Option<Gamemode>);


impl PreviousGamemode {
    /// Gets a previous gamemode from its ID.
    pub fn from_id(id: i8) -> Self {
        PreviousGamemode(match id {
            0 => Some(Gamemode::Survival),
            1 => Some(Gamemode::Creative),
            _ => None,
        })
    }

    /// Gets this gamemode's id
    pub fn id(&self) -> i8 {
        match self.0 {
            Some(Gamemode::Survival) => 0,
            Some(Gamemode::Creative) => 1,
            None => -1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OffgroundHeight(pub f32, pub f32);
pub struct PlayerBuilder {}
impl PlayerBuilder {
    pub fn create(
        game: &mut Game,
        username: Username,
        position: Position,
        id: NetworkID,
        world_info: CurrentWorldInfo,
        gamemode: Gamemode,
    ) -> EntityBuilder {
        let op_manager = game.objects.get::<OpManager>().unwrap();
        let perm_level = match op_manager.is_op(&username.0) {
            true => 4,
            false => 1,
        };
        drop(op_manager);
        let inventory = Inventory::player();
        let window = Window::new(BackingWindow::Player {
            player: inventory.new_handle(),
        });
        let mut builder = game.create_entity_builder(position, EntityInit::Player);
        builder.add(Player);
        builder.add(position);
        builder.add(username);
        builder.add(id);
        builder.add(world_info);
        builder.add(Chatbox::new());
        builder.add(Sneaking(false));
        builder.add(Sprinting::new(false));
        builder.add(HotbarSlot::new(0));
        builder.add(PreviousGamemode::from_id(gamemode.id()));
        builder.add(gamemode);
        builder.add(window);
        builder.add(PermissionLevel(perm_level));
        builder.add(Health(20, DamageType::None));
        builder.add(PreviousHealth(Health(20, DamageType::None)));
        builder.add(Hunger(20, 0.0));
        builder.add(PreviousHunger(Hunger(20, 0.0)));
        builder.add(OffgroundHeight(0., 0.));
        builder.add(Regenerator(0));
        builder.add(AABBSize::new(-0.3, 0.05, -0.3, 0.3, 1.6, 0.3));
        builder.add(View::new(position.to_chunk_coords(), CONFIGURATION.chunk_distance));
        builder
    }
}
