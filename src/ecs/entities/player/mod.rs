use std::{collections::HashMap, sync::Arc};

use flume::{Receiver, Sender};
use hecs::{Entity, EntityBuilder};

use crate::{ecs::Ecs, game::{ChunkCoords, Position, Game}, network::{ids::NetworkID}, world::{chunks::Chunk, view::View}, entities::{EntityInit, PreviousPosition}};

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
    pub fn retain(&mut self, f: impl FnMut(&ChunkCoords,) -> bool) {
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
    messages: Vec<ChatMessage>
}
impl Chatbox {
    pub fn new() -> Self {
        Self { messages: Vec::new() }
    }
    pub fn send_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }
    pub fn drain(&mut self) -> impl Iterator<Item = ChatMessage> + '_ {
        self.messages.drain(..)
    }
}

/// Whether an entity is sneaking, like in pressing shift.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
)]
pub struct Sneaking(pub bool);

/// A component on players that tracks if they are sprinting or not.
#[derive(
    Copy,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Hash,
)]
pub struct Sprinting(pub bool);
impl Sprinting {
    pub fn new(value: bool) -> Self {
        Sprinting(value)
    }
}
pub struct PlayerBuilder {

}
impl PlayerBuilder {
    pub fn create(game: &mut Game, username: Username, position: Position, id: NetworkID, world_info: CurrentWorldInfo) -> EntityBuilder {
        let mut builder = game.create_entity_builder(position, EntityInit::Player);
        builder.add(Player);
        builder.add(position);
        builder.add(username);
        builder.add(id);
        builder.add(world_info);
        builder.add(Chatbox::new());
        builder.add(Sneaking(false));
        builder.add(Sprinting::new(false));
        builder.add(View::new(position.to_chunk_coords(), 8));
        builder
    }
}