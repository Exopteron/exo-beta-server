use hecs::EntityBuilder;

use crate::{game::{Game, Position, DamageType}, entities::{EntityInit, PreviousPosition}, ecs::entities::item::Life, physics::Physics, aabb::AABBSize, network::ids::NetworkID, protocol::packets::EnumMobType};

use super::{PreviousHealth, Health};
pub struct ZombieEntity;
pub struct ZombieEntityBuilder;
impl ZombieEntityBuilder {
    pub fn build(game: &mut Game, mut position: Position) -> EntityBuilder {
        let mut builder = game.create_entity_builder(position, EntityInit::Mob);
        builder.add(position);
        builder.add(Health(20, DamageType::None));
        builder.add(PreviousHealth(Health(20, DamageType::None)));
        builder.add(PreviousPosition(position));
        builder.add(ZombieEntity);
        builder.add(EnumMobType::Zombie);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(-0.3, 0.0, -0.3, 0.3, 1.6, 0.3));
        builder.add(Physics::new(true, 0.1));
        builder.add(Life(0));
        builder.add(0usize); // temporary
        builder
    }
}