use crate::{
    ecs::entities::{living::Hunger, player::ItemInUse},
    item::{
        item::{Item, ItemIdentifier},
        window::Window,
    },
    network::{ids::NetworkID, metadata::Metadata}, entities::metadata::EntityBitMask, game::Position,
};

pub struct FoodItem(pub ItemIdentifier, pub i16);

impl Item for FoodItem {
    fn id(&self) -> ItemIdentifier {
        self.0
    }

    fn stack_size(&self) -> i8 {
        64
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(
        &self,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>,
        slot: usize,
        user: hecs::Entity,
        target: Option<crate::item::item::BlockUseTarget>,
    ) -> crate::ecs::systems::SysResult {
        if target.is_none() {
            let hunger = game.ecs.get::<Hunger>(user)?;
            if hunger.0 < 20 {
                let mut iiu = game.ecs.get_mut::<ItemInUse>(user)?;
                iiu.0 = item.clone();
                iiu.1 = 32;
                let mut metadata = game.ecs.get_mut::<Metadata>(user)?;
                metadata.flags.set(EntityBitMask::EATING, true);
                let entityref = game.ecs.entity(user)?;
                let network_id = *entityref.get::<NetworkID>()?;
                server.broadcast_nearby_with(*entityref.get::<Position>()?, |client| {
                    client.send_entity_metadata(true, network_id, metadata.clone());
                });
            }
        }
        Ok(())
    }
    fn on_stop_using(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, eater: hecs::Entity, item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>, slot: usize) -> crate::ecs::systems::SysResult {
        let mut metadata = game.ecs.get_mut::<Metadata>(eater)?;
        metadata.flags.set(EntityBitMask::EATING, false);
        let entityref = game.ecs.entity(eater)?;
        let network_id = *entityref.get::<NetworkID>()?;
        server.broadcast_nearby_with(*entityref.get::<Position>()?, |client| {
            client.send_entity_metadata(true, network_id, metadata.clone());
        });
        Ok(())
    }
    fn on_eat(
        &self,
        game: &mut crate::game::Game,
        server: &mut crate::server::Server,
        eater: hecs::Entity,
        mut item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>,
        slot: usize,
    ) -> crate::ecs::systems::SysResult {
        let mut metadata = game.ecs.get_mut::<Metadata>(eater)?;
        metadata.flags.set(EntityBitMask::EATING, false);
        let entityref = game.ecs.entity(eater)?;
        let network_id = *entityref.get::<NetworkID>()?;
        server.broadcast_nearby_with(*entityref.get::<Position>()?, |client| {
            client.send_entity_metadata(true, network_id, metadata.clone());
        });
        item.try_take(1);
        drop(item);
        let mut hunger = game.ecs.get_mut::<Hunger>(eater)?;
        hunger.0 += self.1;
        let id = *game.ecs.get::<NetworkID>(eater)?;
        let window = game.ecs.get::<Window>(eater)?;
        server.clients.get(&id).unwrap().send_window_items(&window);
        let entity_ref = game.ecs.entity(eater)?;
        server.broadcast_equipment_change(&entity_ref)?;
        Ok(())
    }
}
