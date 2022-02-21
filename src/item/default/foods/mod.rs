use crate::{item::{item::{ItemIdentifier, Item}, window::Window}, ecs::entities::{player::ItemInUse, living::Hunger}, network::ids::NetworkID};

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
    fn on_use(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> crate::ecs::systems::SysResult {
        if target.is_none() {
            let hunger = game.ecs.get::<Hunger>(user)?;
            if hunger.0 < 20 {
                let mut iiu = game.ecs.get_mut::<ItemInUse>(user)?;
                iiu.0 = item.clone();
                iiu.1 = 32;
            }
        }
        Ok(())
    }
    fn on_eat(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, eater: hecs::Entity, mut item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>, slot: usize) -> crate::ecs::systems::SysResult {
        item.try_take(1);
        drop(item);
        let mut hunger = game.ecs.get_mut::<Hunger>(eater)?;
        hunger.0 += self.1;
        let id = *game.ecs.get::<NetworkID>(eater)?;
        let window = game.ecs.get::<Window>(eater)?;
        server.clients.get(&id).unwrap().send_window_items(&window);
        Ok(())
    }
}