use hecs::EntityBuilder;
use nbt::CompoundTag;

use crate::{game::{Game, Position, DamageType}, entities::{EntityInit, PreviousPosition}, ecs::{entities::{item::Life, living::EntityWorldInteraction}, EntityRef, systems::entities::living::hostile::zombie::MobPhysics, }, physics::Physics, aabb::AABBSize, network::ids::NetworkID, protocol::packets::EnumMobType, item::{window::Window, inventory::{reference::BackingWindow, Inventory}}, entity_loader::RegularEntitySaver};

use super::super::{PreviousHealth, Health, PreviousHunger, Hunger};
pub struct ChickenEntity;
pub struct ChickenEntityBuilder;
impl ChickenEntityBuilder {
    pub fn build<'a>(
        mut position: Option<Position>,
        health: i16,
        builder: &'a mut EntityBuilder,
    ) {
        if let Some(position) = position {
            builder.add(position);
            builder.add(PreviousPosition(position));
        }
        builder.add(crate::status_effects::StatusEffectsManager::default());
        builder.add(crate::network::metadata::Metadata::new());
        builder.add(EntityWorldInteraction::default());
        builder.add(Health(health, DamageType::None, false));
        builder.add(PreviousHealth(Health(health, DamageType::None, false)));
        builder.add(ChickenEntity);
        builder.add(EnumMobType::Hen);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(-0.3, 0.05, -0.3, 0.3, 0.5, 0.3));
        builder.add(Physics::new(true, 0.1, 1.));


        let inventory = Inventory::player();
        builder.add(inventory.new_handle());
        builder.add(Window::new(BackingWindow::Player { player: inventory }));
        builder.add(Hunger(20, 0.0));
        builder.add(PreviousHunger(Hunger(20, 0.0)));
        builder.add(Life(0));
        builder.add(MobPhysics);
        builder.add(0usize); // temporary

        builder.add(RegularEntitySaver::new(
            Self::chicken_saver,
            "Chicken".to_string(),
        ));
    }

    pub fn chicken_saver(entity: &EntityRef) -> anyhow::Result<CompoundTag> {
        let mut tag = CompoundTag::new();
        tag.insert_i16("Health", entity.get::<Health>()?.0);
        Ok(tag)
    }
}