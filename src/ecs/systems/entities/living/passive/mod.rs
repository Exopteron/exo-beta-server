use crate::{ecs::systems::SystemExecutor, game::Game};

pub mod sheep;
pub mod chicken;
pub mod cow;
pub mod pig;

pub fn init_systems(g: &mut Game, s: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    sheep::init_systems(g, s)?;
    cow::init_systems(g, s)?;
    chicken::init_systems(g, s)?;
    pig::init_systems(g, s)
}