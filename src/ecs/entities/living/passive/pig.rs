use hecs::EntityBuilder;
use nbt::CompoundTag;

use crate::{game::{Game, Position, DamageType}, entities::{EntityInit, PreviousPosition}, ecs::{entities::item::Life, EntityRef, systems::entities::living::hostile::zombie::MobPhysics, }, physics::Physics, aabb::AABBSize, network::ids::NetworkID, protocol::packets::EnumMobType, item::{window::Window, inventory::{reference::BackingWindow, Inventory}}, entity_loader::RegularEntitySaver};

use super::super::{PreviousHealth, Health, PreviousHunger, Hunger};
pub struct PigEntity;
pub struct PigEntityBuilder;
impl PigEntityBuilder {
    pub fn build<'a>(
        mut position: Option<Position>,
        health: i16,
        builder: &'a mut EntityBuilder,
    ) {
        if let Some(position) = position {
            builder.add(position);
            builder.add(PreviousPosition(position));
        }

        builder.add(Health(health, DamageType::None, false));
        builder.add(PreviousHealth(Health(health, DamageType::None, false)));
        builder.add(PigEntity);
        builder.add(EnumMobType::Pig);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(-0.3, 0.05, -0.3, 0.3, 0.8, 0.3));
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
            Self::pig_saver,
            "Sheep".to_string(),
        ));
    }

    pub fn pig_saver(entity: &EntityRef) -> anyhow::Result<CompoundTag> {
        let mut tag = CompoundTag::new();
        tag.insert_i16("Health", entity.get::<Health>()?.0);
        Ok(tag)
    }
}