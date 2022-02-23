use hecs::EntityBuilder;

use crate::{game::{Game, Position, DamageType}, entities::{EntityInit, PreviousPosition}, ecs::entities::item::Life, physics::Physics, aabb::AABBSize, network::ids::NetworkID, protocol::packets::EnumMobType, item::{window::Window, inventory::{reference::BackingWindow, Inventory}}};

use super::{PreviousHealth, Health, PreviousHunger, Hunger};
pub struct ZombieEntity;
pub struct ZombieEntityBuilder;
impl ZombieEntityBuilder {
    pub fn build(game: &mut Game, mut position: Position) -> EntityBuilder {
        let mut builder = game.create_entity_builder(position, EntityInit::Mob);
        builder.add(position);
        builder.add(Health(20, DamageType::None, false));
        builder.add(PreviousHealth(Health(20, DamageType::None, false)));
        builder.add(PreviousPosition(position));
        builder.add(ZombieEntity);
        builder.add(EnumMobType::Zombie);
        builder.add(NetworkID::new());
        builder.add(AABBSize::new(-0.3, 0.1, -0.3, 0.3, 1.6, 0.3));
        builder.add(Physics::new(true, 0.1, 1.));
        builder.add(Hunger(20, 0.0));
        builder.add(PreviousHunger(Hunger(20, 0.0)));
        let inventory = Inventory::player();
        builder.add(inventory.new_handle());
        builder.add(Window::new(BackingWindow::Player { player: inventory }));
        builder.add(Life(0));
        builder.add(0usize); // temporary
        builder
    }
}