// Feather license in FEATHER_LICENSE.md
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

#[derive(Debug)]
pub struct MaxPlayersReached;

/// Maintains the server player count.
///
/// Can be cloned to create a new handle.
#[derive(Clone)]
pub struct PlayerCount {
    inner: Arc<Inner>,
}

impl PlayerCount {
    pub fn new(max_players: u32) -> Self {
        Self {
            inner: Arc::new(Inner {
                count: AtomicU32::new(0),
                max_players,
            }),
        }
    }

    pub fn try_add_player(&self) -> Result<(), MaxPlayersReached> {
        loop {
            let current_count = self.inner.count.load(Ordering::SeqCst);
            let new_count = current_count + 1;
            if new_count > self.inner.max_players {
                return Err(MaxPlayersReached);
            }

            if self
                .inner
                .count
                .compare_exchange(current_count, new_count, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(());
            }
        }
    }

    pub fn remove_player(&self) {
        self.inner.count.fetch_sub(1, Ordering::SeqCst);
    }

    pub fn get(&self) -> u32 {
        self.inner.count.load(Ordering::Acquire)
    }
    pub fn get_max(&self) -> u32 {
        self.inner.max_players
    }
}

struct Inner {
    count: AtomicU32,
    max_players: u32,
}