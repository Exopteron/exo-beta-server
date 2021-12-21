use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use ahash::AHashMap;

use crate::game::ChunkCoords;

use super::chunk_lock::ChunkHandle;

#[cfg(not(test))]
const CACHE_TIME: Duration = Duration::from_secs(30);
#[cfg(test)]
const CACHE_TIME: Duration = Duration::from_millis(500);

/// This struct contains chunks that were unloaded but remain in memory in case they are needed.
#[derive(Default)]
pub struct ChunkCache {
    map: AHashMap<ChunkCoords, (Instant, ChunkHandle)>, // expire time + handle
    unload_queue: VecDeque<ChunkCoords>,
}
impl ChunkCache {
    pub fn new() -> Self {
        Self {
            map: AHashMap::new(),
            unload_queue: VecDeque::new(),
        }
    }
    /// Purges all unused chunk handles. Handles that exist elsewhere in the memory are not removed.
    pub fn purge_unused(&mut self) {
        let mut to_remove: Vec<ChunkCoords> = vec![];
        for (pos, (_, arc)) in self.map.iter() {
            if Arc::strong_count(arc) == 1 {
                to_remove.push(*pos)
            }
        }
        for i in to_remove {
            self.map.remove(&i);
        }
    }
    /// Purges all chunk handles in the cache, including those that exist elswhere.
    pub fn purge_all(&mut self) {
        self.map.clear();
        self.unload_queue.clear();
    }
    fn ref_count(&self, pos: &ChunkCoords) -> Option<usize> {
        self.map.get(pos).map(|(_, arc)| Arc::strong_count(arc))
    }
    /// Purges all chunks that have been in unused the cache for longer than `CACHE_TIME`. Refreshes this timer for chunks that are in use at the moment.
    pub fn purge_old_unused(&mut self) {
        while let Some(&pos) = self.unload_queue.get(0) {
            if !self.contains(&pos) {
                // Might be caused by a manual purge
                self.unload_queue.pop_front();
                continue;
            }
            if self.map.get(&pos).unwrap().0 > Instant::now() {
                // Subsequent entries are 'scheduled' for later
                break;
            }
            self.unload_queue.pop_front();
            if self.ref_count(&pos).unwrap() > 1 {
                // Another copy of this handle already exists
                self.unload_queue.push_back(pos);
                self.map.entry(pos).and_modify(|(time, _)| {
                    *time = Instant::now() + CACHE_TIME;
                });
            } else {
                self.map.remove_entry(&pos);
            }
        }
    }
    /// Returns whether the chunk at the position is cached.
    pub fn contains(&self, pos: &ChunkCoords) -> bool {
        self.map.contains_key(pos)
    }
    /// Inserts a chunk handle into the cache, returning the previous handle if there was one.
    pub fn insert(&mut self, pos: ChunkCoords, handle: ChunkHandle) -> Option<ChunkHandle> {
        self.unload_queue.push_back(pos);
        self.map
            .insert(pos, (Instant::now() + CACHE_TIME, handle))
            .map(|(_, handle)| handle)
    }
    /// Inserts a chunk handle into the cache. Reads the chunk's position by locking it. Blocks.
    pub fn insert_read_pos(&mut self, handle: ChunkHandle) -> Option<ChunkHandle> {
        let pos = handle.read().position();
        self.insert(pos, handle)
    }
    /// Removes the chunk handle at the given position, returning the handle if it was cached.
    pub fn remove(&mut self, pos: ChunkCoords) -> Option<ChunkHandle> {
        self.map.remove(&pos).map(|(_, handle)| handle)
    }
    /// Returns the chunk handle at the given position, if there was one.
    pub fn get(&mut self, pos: ChunkCoords) -> Option<ChunkHandle> {
        self.map.get(&pos).map(|(_, handle)| handle.clone())
    }
    pub fn len(&self) -> usize {
        self.map.len()
    }
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}