use crate::server::Server;
use crate::game::{Game, Message, DamageType, ChunkCoords};
use crate::network::ids::{EntityID, IDS};
use crate::network::packet::{ClientPacket, ServerPacket};
use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::configuration::CONFIGURATION;
pub mod entities;
pub struct Systems {
    systems: Vec<(String, Box<dyn FnMut(&mut crate::game::Game) -> anyhow::Result<()> + 'static>)>,
}
impl Systems {
    pub fn new() -> Self {
        Self { systems: Vec::new() }
    }
    pub fn add_system(&mut self, name: &str, system: impl FnMut(&mut crate::game::Game) -> anyhow::Result<()> + 'static) {
        self.systems.push((name.to_string(), Box::new(system)));
    }
    pub fn run(&mut self, game: &mut crate::game::Game) {
        for system in &mut self.systems {
            let start_time = Instant::now();
            if let Err(e) = system.1(game) {
                log::error!("System {} returned an error. Details: {:?}", system.0, e);
            }
            if CONFIGURATION.logging.profiler {
                log::info!("[Profiler] System {} took {}ms. ({}ns)", system.0, start_time.elapsed().as_millis(), start_time.elapsed().as_nanos());
            }
        }
    }
}
pub fn default_systems(s: &mut Systems) {
    entities::player::init_systems(s);
}