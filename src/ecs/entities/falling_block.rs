use hecs::{Entity, EntityBuilder};

use crate::{game::{Game, Position}, item::stack::ItemStack, entities::{EntityInit, PreviousPosition}, network::ids::NetworkID, aabb::AABBSize, physics::Physics, world::chunks::BlockState};

use super::item::Life;


pub struct FallingBlockEntity;

#[derive(Clone, Copy)]
pub enum FallingBlockEntityData {
    Gravel,
    Sand
}
impl FallingBlockEntityData {
    pub fn block_id(&self) -> u8 {
        match self {
            Self::Gravel => 13,
            Self::Sand => 12
        }
    }
}
pub struct FallingBlockEntityBuilder;
impl FallingBlockEntityBuilder {
    pub fn build(game: &mut Game, mut position: Position, kind: FallingBlockEntityData) -> EntityBuilder {
        position.update = false;
        let mut builder = game.create_entity_builder(position, EntityInit::FallingBlock);
        builder.add(position);
        builder.add(PreviousPosition(position));
        builder.add(kind);
        builder.add(FallingBlockEntity);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(0., 0., 0., 0.5, 0.5, 0.5));
        builder.add(Physics::new(true, 0.1, 0.));
        builder.add(Life(0));
        builder
    }
}