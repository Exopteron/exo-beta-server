use crate::{item::{item::{ItemIdentifier, Item}, window::Window, inventory_slot::InventorySlot}, network::ids::NetworkID};

use super::ToolMaterials;

pub struct PickaxeItem(pub ItemIdentifier, pub ToolMaterials);
impl Item for PickaxeItem {
    fn id(&self) -> ItemIdentifier {
        self.0
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        Some(self.1.max_uses())
    }
    fn tool_type(&self) -> Option<ToolMaterials> {
        Some(self.1)
    }
    fn on_dig_with(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, digger: hecs::Entity, mut item: &mut parking_lot::MutexGuard<crate::item::inventory_slot::InventorySlot>, slot: usize, target: crate::item::item::BlockUseTarget) -> crate::ecs::systems::SysResult {
        let id = *game.ecs.get::<NetworkID>(digger)?;
        let mut do_break = false;
        if let InventorySlot::Filled(item) = &mut **item {
            if item.damage(1) {
                do_break = true;
            }
        }
        if do_break {
            **item = InventorySlot::Empty;
        }
        Ok(())
    }
}