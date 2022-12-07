use std::{collections::HashMap, sync::Arc};

use anyhow::bail;
use flume::{Receiver, Sender};
use hecs::{Entity, EntityBuilder};
use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    ecs::{systems::SysResult, Ecs},
    entities::{EntityInit, PreviousPosition},
    game::{ChunkCoords, Game, Position, DamageType, BlockPosition},
    network::{ids::NetworkID, metadata::Metadata},
    world::{chunks::Chunk, view::View, LevelDat}, item::{inventory::{Inventory, reference::BackingWindow}, window::Window, stack::ItemStack, inventory_slot::InventorySlot}, commands::PermissionLevel, configuration::{CONFIGURATION, OpManager}, aabb::AABBSize, status_effects::StatusEffectsManager, physics::Physics, player_dat::PlayerDat,
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
pub struct HitCooldown(pub u128);

#[derive(Clone, Debug)]
pub struct ItemInUse(pub InventorySlot, pub u128);

#[derive(Clone, Debug)]
pub struct BlockInventoryOpen(pub BlockPosition);

#[derive(Clone, Debug)]
pub struct LastPositionUpdate(pub u128);

#[derive(Clone, Debug)]
pub struct Blocking(pub bool);


#[derive(Clone, Debug)]
pub struct Sleeping {
    bed_pos: Option<BlockPosition>,
    sleeping: bool,
    changed_flag: bool
}
impl Sleeping {
    pub fn new() -> Self {
        Self { bed_pos: None, sleeping: false, changed_flag: false }
    }
    pub fn unset_sleeping(&mut self) {
        self.bed_pos = None;
        self.sleeping = false;
        self.changed_flag = true;
    }
    pub fn set_sleeping(&mut self, chunk: BlockPosition) {
        self.bed_pos = Some(chunk);
        self.sleeping = true;
        self.changed_flag = true;
    }
    pub fn changed(&self) -> bool {
        self.changed_flag
    }
    pub fn reset_changed(&mut self) {
        self.changed_flag = false;
    }
    pub fn is_sleeping(&self) -> bool {
        self.sleeping
    }
    pub fn bed_coords(&self) -> Option<BlockPosition> {
        self.bed_pos
    } 
}
pub struct PlayerBuilder {}
impl PlayerBuilder {
    pub fn create(
        game: &mut Game,
        username: Username,
        position: Position,
        id: NetworkID,
        gamemode: Gamemode,
        player_dat: &anyhow::Result<PlayerDat>,
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
        if let Ok(player_dat) = player_dat {
            for item in player_dat.inventory.iter() {
                if let Ok(mut w_item) = window.item(item.slot as usize) {
                    *w_item = InventorySlot::Filled((*item).into());
                }
            }
        }
        let mut builder = game.create_entity_builder(position, EntityInit::Player);
        builder.add(ItemInUse(InventorySlot::Empty, 0));
        builder.add(inventory);
        builder.add(Metadata::new());
        builder.add(HitCooldown(0));
        builder.add(Physics::new(false, 0.1, 0.));
        builder.add(StatusEffectsManager::default());
        builder.add(Player);
        builder.add(position);
        builder.add(username);
        builder.add(id);
        builder.add(Chatbox::new());
        builder.add(Sneaking(false));
        builder.add(Sprinting::new(false));
        builder.add(HotbarSlot::new(0));
        builder.add(PreviousGamemode::from_id(gamemode.id()));
        builder.add(gamemode);
        builder.add(window);
        builder.add(PermissionLevel(perm_level));
        builder.add(LastPositionUpdate(game.ticks));
        builder.add(Blocking(false));
        builder.add(Sleeping::new());
        if let Ok(p) = player_dat {
            builder.add(Health(p.health, DamageType::None, false));
            builder.add(PreviousHealth(Health(-1, DamageType::None, false)));
            builder.add(Hunger(p.food_level as i16, p.food_saturation));
            builder.add(PreviousHunger(Hunger(-1, p.food_saturation)));
        } else {
            builder.add(Health(20, DamageType::None, false));
            builder.add(PreviousHealth(Health(20, DamageType::None, false)));
            builder.add(Hunger(20, 0.0));
            builder.add(PreviousHunger(Hunger(20, 0.0)));
        }
        builder.add(Regenerator(0));
        builder.add(AABBSize::new(-0.3, 0.05, -0.3, 0.3, 1.6, 0.3));
        builder.add(View::new(position.to_chunk_coords(), CONFIGURATION.chunk_distance));
        builder
    }
}
