use crate::{game::Game, ecs::systems::SystemExecutor};

pub mod creeper;
pub mod skeleton;
pub mod spider;
pub mod zombie;

pub fn init_systems(g: &mut Game, s: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    zombie::init_systems(g, s)?;
    skeleton::init_systems(g, s)?;
    spider::init_systems(g, s)?;
    creeper::init_systems(g, s)
}