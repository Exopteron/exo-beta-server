use crate::game::Game;

use super::SystemExecutor;

pub mod view;
pub mod loading;
pub mod block;
pub mod ticking;
pub mod light;
pub fn register(game: &mut Game, systems: &mut SystemExecutor<Game>) {
    view::register(systems);
    loading::register(game, systems);
    block::register(game, systems);
    ticking::register(game, systems);
    light::register(game, systems);
}