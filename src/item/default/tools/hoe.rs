use crate::{item::{item::{Item, ItemIdentifier}, window::Window, inventory_slot::InventorySlot}, world::chunks::BlockState, network::ids::NetworkID};

use super::ToolMaterials;

pub struct HoeItem(pub ItemIdentifier, pub ToolMaterials);

impl Item for HoeItem {
    fn tool_type(&self) -> Option<ToolMaterials> {
        Some(self.1)
    }
    fn id(&self) -> crate::item::item::ItemIdentifier {
        self.0
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        Some(self.1.max_uses())
    }

    fn on_use(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, mut item: parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> crate::ecs::systems::SysResult {
        if let Some(target) = target {
            let pos = target.position;
            let block_1 = game.block_id_at(pos);
            let block_2 = game.block_id_at(pos.offset(0, 1, 0));
            if block_2 == 0 && (block_1 == 2 || block_1 == 3) {
                
                let id = *game.ecs.get::<NetworkID>(user)?;
                let window = game.ecs.get_mut::<Window>(user)?;
                let mut do_break = false;
                if let InventorySlot::Filled(item) = &mut *item {
                    if item.damage(1) {
                        do_break = true;
                    }
                }
                if do_break {
                    *item = InventorySlot::Empty;
                }
                drop(item);
                server.clients.get(&id).unwrap().send_window_items(&window);
                drop(window);
                
                game.set_block(pos, BlockState::from_id(60), pos.world);
            }
        }
        Ok(())
    }
}