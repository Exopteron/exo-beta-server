use crate::{item::{item::Item, window::Window, inventory_slot::InventorySlot}, world::chunks::BlockState, network::ids::NetworkID};

pub struct WheatItem;
impl Item for WheatItem {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        296
    }

    fn stack_size(&self) -> i8 {
        64
    }

    fn durability(&self) -> Option<i16> {
        None
    }
}

pub struct WheatSeeds;
impl Item for WheatSeeds {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        295
    }

    fn stack_size(&self) -> i8 {
        64
    }
    

    fn durability(&self) -> Option<i16> {
        None
    }

    fn on_use(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, mut item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> crate::ecs::systems::SysResult {
        if let Some(target) = target {
            let pos = target.position;
            if game.block_id_at(pos) == 60 && game.block_id_at(pos.offset(0, 1, 0)) == 0 {

                let id = *game.ecs.get::<NetworkID>(user)?;
                let window = game.ecs.get_mut::<Window>(user)?;
                item.try_take(1);
                drop(item);
                server.clients.get(&id).unwrap().send_window_items(&window);
                drop(window);

                game.set_block(pos.offset(0, 1, 0), BlockState::from_id(59), pos.world);
            }
        }
        Ok(())
    }
}