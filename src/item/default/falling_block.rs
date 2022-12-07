use hecs::Entity;

use crate::{
    ecs::{entities::{player::Chatbox, falling_block::{FallingBlockEntityData, FallingBlockEntityBuilder}}, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID, aabb::AABB,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct FallingBlock(pub FallingBlockEntityData);
impl Block for FallingBlock {
    fn id(&self) -> BlockIdentifier {
        self.0.block_id()
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn added(&self, world: i32, game: &mut Game, server: &mut crate::server::Server, position: BlockPosition, state: BlockState) {
        let kind = self.0;
        game.schedule_at(game.ticks + 3, move |g| {
            Self::try_fall(kind, g, position);
            None
        });
    }
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        let kind = self.0;
        game.schedule_at(game.ticks + 3, move |g| {
            Self::try_fall(kind, g, position);
            None
        });
        Ok(())
    }
}
impl FallingBlock {
    pub fn try_fall(kind: FallingBlockEntityData, game: &mut Game, pos: BlockPosition) {
        if game.block_id_at(pos) == kind.block_id() && Self::can_fall(game, pos.offset(0, -1, 0)) && pos.y >= 0 {
            game.set_block_nb(pos, BlockState::air(), pos.world, true, false, true);
            game.schedule_next_tick(move |g| {
                g.set_block_nb(pos, BlockState::air(), pos.world, false, true, false);
                None
            });
            let mut pos: Position = pos.into();
            pos.x += 0.5;
            pos.y += 0.5;
            pos.z += 0.5;
            let entity = FallingBlockEntityBuilder::build(game, pos, kind);
            game.spawn_entity(entity);
        }
    }
    pub fn can_fall(game: &mut Game, pos: BlockPosition) -> bool {
        let id = game.block_id_at(pos);
        if id == 0 {
            return true;
        }
        if id == 51 {
            return true;
        }
        false
    }
}