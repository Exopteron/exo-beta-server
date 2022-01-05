use crate::{game::{Game, Position}, item::stack::ItemStack, entities::EntityInit, network::ids::NetworkID, aabb::AABBSize, physics::Physics};

use super::player::{CurrentWorldInfo, PreviousWorldInfo};

pub struct ItemEntity;

#[derive(Clone)]
pub struct ItemEntityData(pub ItemStack);

#[derive(Clone, Copy)]
pub struct Life(pub u128);
pub struct ItemEntityBuilder;
impl ItemEntityBuilder {
    pub fn build(game: &mut Game, position: Position, item: ItemStack) {
        let mut builder = game.create_entity_builder(position, EntityInit::Item);
        builder.add(position);
        builder.add(ItemEntityData(item));
        builder.add(ItemEntity);
        builder.add(NetworkID::new());
        builder.add(CurrentWorldInfo::new(position.world));
        builder.add(PreviousWorldInfo(CurrentWorldInfo::new(position.world), CurrentWorldInfo::new(position.world)));
        builder.add(AABBSize::new(0., 0., 0., 0.5, -0.5, 0.5));
        builder.add(Physics::new(true, 0.1));
        builder.add(Life(0));
        game.spawn_entity(builder);
    }
}