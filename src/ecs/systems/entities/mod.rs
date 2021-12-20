use crate::{game::Game, server::Server};

use super::{SystemExecutor, SysResult};

pub mod player;
pub fn default_systems(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    systems.group::<Server>().add_system(tick_clients);
}
/// Ticks `Client`s.
fn tick_clients(_game: &mut Game, server: &mut Server) -> SysResult {
    for client in server.clients.iter() {
        client.1.tick();
    }
    Ok(())
}
