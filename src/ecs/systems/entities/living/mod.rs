use crate::{ecs::systems::SystemExecutor, game::Game};

pub mod hostile;
pub mod passive;
pub mod damage;

pub fn init_systems(g: &mut Game, s: &mut SystemExecutor<Game>) -> anyhow::Result<()> {
    passive::init_systems(g, s)?;
    hostile::init_systems(g, s)?;
    damage::init_systems(g, s)
}