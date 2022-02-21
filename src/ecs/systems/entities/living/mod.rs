use crate::{ecs::systems::SystemExecutor, game::Game};

pub mod zombie;

pub fn init_systems(s: &mut SystemExecutor<Game>) {
    zombie::init_systems(s);
}