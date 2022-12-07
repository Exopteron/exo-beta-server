use hecs::{Entity, EntityBuilder};
use nbt::CompoundTag;

use crate::{game::{Game, Position}, item::stack::ItemStack, entities::{EntityInit, PreviousPosition}, network::ids::NetworkID, aabb::AABBSize, physics::Physics, ecs::EntityRef, entity_loader::RegularEntitySaver};


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
        
        builder.add(RegularEntitySaver::new(
            Self::item_saver,
            "Item".to_string(),
        ));
        builder
    }

    pub fn item_saver(entity: &EntityRef) -> anyhow::Result<CompoundTag> {
        let mut tag = CompoundTag::new();

        let item = entity.get::<ItemEntityData>()?;

        let mut item_tag = CompoundTag::new();
        item_tag.insert_i16("id", item.0.id());
        item_tag.insert_i16("Damage", item.0.damage_taken());
        item_tag.insert_i8("Count", item.0.count());

        tag.insert_compound_tag("Item", item_tag);
        Ok(tag)
    }
}