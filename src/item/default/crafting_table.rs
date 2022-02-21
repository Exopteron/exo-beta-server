use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;

use crate::{
    aabb::{AABB, AABBSize},
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType}, inventory::{Inventory, reference::BackingWindow}, window::Window,
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind, WindowKind},
    world::chunks::BlockState, block_entity::{BlockEntityLoader, SignData, BlockEntity, BlockEntitySaver, NoteblockData},
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct CraftingTable;

impl Block for CraftingTable {
    fn id(&self) -> BlockIdentifier {
        58
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn interacted_with(&self, world: i32, game: &mut Game, server: &mut crate::server::Server, position: BlockPosition, state: BlockState, player: Entity) -> anyhow::Result<ActionResult> {
        let player = game.ecs.entity(player)?;
        let id = *player.get::<NetworkID>()?;
        let inventory = player.get::<Inventory>()?.new_handle();
        let mut window = player.get_mut::<Window>()?;
        let new_window = Window::new(BackingWindow::Crafting { crafting_table: Inventory::crafting_table(), player: inventory });
        *window = new_window;
        server.clients.get(&id).unwrap().open_window(1, WindowKind::Workbench, "Crafting Table".to_string(), 9);
        Ok(ActionResult::SUCCESS)
    }
}