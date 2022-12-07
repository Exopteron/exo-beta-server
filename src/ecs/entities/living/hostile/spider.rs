use std::{
    borrow::Cow,
    ops::{Deref, DerefMut},
};

use hecs::EntityBuilder;
use nbt::CompoundTag;

use crate::{
    aabb::AABBSize,
    ecs::{entities::item::Life, EntityRef, systems::entities::living::hostile::zombie::MobPhysics},
    entities::{EntityInit, PreviousPosition},
    entity_loader::RegularEntitySaver,
    game::{DamageType, Game, Position},
    item::{
        inventory::{reference::BackingWindow, Inventory},
        window::Window,
    },
    network::ids::NetworkID,
    physics::Physics,
    protocol::packets::EnumMobType,
};

use super::super::{Health, Hunger, PreviousHealth, PreviousHunger};
pub struct SpiderEntity;
pub struct SpiderEntityBuilder;


impl SpiderEntityBuilder {
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
        builder.add(SpiderEntity);
        builder.add(EnumMobType::Spider);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(-0.5, 0.05, -0.5, 0.5, 0.5, 0.5));
        builder.add(Physics::new(true, 0.1, 1.));
        builder.add(Hunger(20, 0.0));
        builder.add(PreviousHunger(Hunger(20, 0.0)));
        let inventory = Inventory::player();
        builder.add(inventory.new_handle());
        builder.add(Window::new(BackingWindow::Player { player: inventory }));
        builder.add(Life(0));
        builder.add(MobPhysics);
        builder.add(0usize); // temporary

        builder.add(RegularEntitySaver::new(
            Self::spider_saver,
            "Spider".to_string(),
        ));
    }

    pub fn spider_saver(entity: &EntityRef) -> anyhow::Result<CompoundTag> {
        let mut tag = CompoundTag::new();
        tag.insert_i16("Health", entity.get::<Health>()?.0);
        Ok(tag)
    }
}
