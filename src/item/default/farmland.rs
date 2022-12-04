use hecs::Entity;
use rand::Rng;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID, aabb::AABB, status_effects::fire::is_water,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct FarmlandBlock;
impl Block for FarmlandBlock {
    fn id(&self) -> BlockIdentifier {
        60
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn tick(&self, _world: i32, game: &mut Game, mut state: BlockState, position: BlockPosition) {
        if rand::thread_rng().gen_range(0..5) == 0 {
            if Self::is_water_near(game, position) || game.can_see_sky(position.offset(0, 1, 0)) {
                state.b_metadata = 7;
                game.set_block(position, state, position.world);
            } else if state.b_metadata > 0 {
                state.b_metadata -= 1;
                game.set_block(position, state, position.world);
            }
        }
    }
    fn neighbor_update(&self, _world: i32, game: &mut Game, position: BlockPosition, mut state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        let above = position.offset(0, 1, 0);
        if game.is_solid_block(above) {
            state.b_type = 3;
            state.b_metadata = 0;
            game.set_block(position, state, position.world);
        }
        Ok(())
    }
}
impl FarmlandBlock {
    pub fn is_water_near(game: &mut Game, pos: BlockPosition) -> bool {
        for x in -4..5 {
            for y in 0..1 {
                for z in -4..5 {
                    if is_water(game.block_id_at(pos.offset(x, y, z))) {
                        return true;
                    }
                }
            }
        }
        false
    } 
}