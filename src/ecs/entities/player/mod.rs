use std::collections::HashMap;

use flume::{Receiver, Sender};
use hecs::Entity;

use crate::{ecs::Ecs, game::{ChunkCoords, Position}, network::{ids::NetworkID}, world::chunks::Chunk};

pub struct Player;
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
pub struct CurrentWorldInfo {
    pub world_id: i32,
}
impl CurrentWorldInfo {
    pub fn new(world_id: i32) -> Self {
        Self { world_id }
    }
}
pub struct PlayerBuilder {

}
impl PlayerBuilder {
    pub fn build(ecs: &mut Ecs, username: Username, position: Position, id: NetworkID, world_info: CurrentWorldInfo) -> Entity {
        ecs.spawn((Player, position, username, id, ChunkLoadQueue::default(), world_info))
    }
}