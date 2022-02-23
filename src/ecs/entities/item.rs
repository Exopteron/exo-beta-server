use hecs::{Entity, EntityBuilder};

use crate::{game::{Game, Position}, item::stack::ItemStack, entities::{EntityInit, PreviousPosition}, network::ids::NetworkID, aabb::AABBSize, physics::Physics};


pub struct ItemEntity;

#[derive(Clone)]
pub struct ItemEntityData(pub ItemStack);

#[derive(Clone, Copy)]
pub struct Life(pub u128);
pub struct ItemEntityBuilder;
impl ItemEntityBuilder {
    pub fn build(game: &mut Game, position: Position, item: ItemStack) -> EntityBuilder {
        let mut builder = game.create_entity_builder(position, EntityInit::Item);
        builder.add(position);
        builder.add(PreviousPosition(position));
        builder.add(ItemEntityData(item));
        builder.add(ItemEntity);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(-0.3, 0., -0.3, 0.3, 0.3, 0.3));
        builder.add(Physics::new(true, 0.1, 0.));
        builder.add(Life(0));
        builder
    }
}