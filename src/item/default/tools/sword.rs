use crate::{item::{item::{ItemIdentifier, Item}, window::Window, inventory_slot::InventorySlot}, network::{ids::NetworkID, metadata::Metadata}, ecs::entities::player::Blocking, entities::metadata::EntityBitMask};

use super::ToolMaterials;

pub struct SwordItem(pub ItemIdentifier, pub ToolMaterials);
impl Item for SwordItem {
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
    fn damage_amount(&self) -> i16 {
        4 + self.1.harvest_level() as i16
    }
    fn on_use(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, item: parking_lot::MutexGuard<InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> crate::ecs::systems::SysResult {
        game.ecs.get_mut::<Blocking>(user)?.0 = true;
        let mut metadata = game.ecs.get_mut::<Metadata>(user)?;
        metadata.flags.set(EntityBitMask::EATING, true);
        metadata.dirty = true;
        Ok(())
    }
    fn on_stop_using(&self, game: &mut crate::game::Game, server: &mut crate::server::Server, eater: hecs::Entity, item: parking_lot::MutexGuard<InventorySlot>, slot: usize) -> crate::ecs::systems::SysResult {
        game.ecs.get_mut::<Blocking>(eater)?.0 = false;
        let mut metadata = game.ecs.get_mut::<Metadata>(eater)?;
        metadata.flags.set(EntityBitMask::EATING, false);
        metadata.dirty = true;
        Ok(())
    }
}