use hecs::Entity;
use rand::Rng;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock, BurnRate}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
pub struct GrassBlock;

impl Block for GrassBlock {
    fn burn_rate(&self) -> Option<crate::item::item::block::BurnRate> {
        Some(BurnRate(50, 50))   
    }
    fn id(&self) -> BlockIdentifier {
        2
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, mut state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
/*         if game.block(position.offset(0, 1, 0), world).ok_or(anyhow::anyhow!("noblock"))?.is_solid() {
            state.b_type = 3;
            state.b_metadata = 0;
            game.set_block(position, state, world);
        } */
        Ok(())
    }
    fn tick(&self, world: i32, game: &mut Game, state: BlockState, position: BlockPosition) {

    }
}