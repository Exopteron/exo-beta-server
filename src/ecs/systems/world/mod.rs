use crate::game::Game;

use super::SystemExecutor;

pub mod view;
pub mod loading;
pub mod block;
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    log::info!("Systems being registered");
    view::register(systems);
    loading::register(game, systems);
    block::register(game, systems);
}