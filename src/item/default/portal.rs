use hecs::Entity;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID, aabb::AABB,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct PortalBlock;
impl Block for PortalBlock {
    fn id(&self) -> BlockIdentifier {
        90   
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn on_collide(&self, game: &mut Game, position: BlockPosition, state: BlockState, player: Entity) -> SysResult {
        log::info!("Collided");
        Ok(())
    }
    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        let mut i1 = 0;
        let mut j1 = 1;
        if game.block_id_at(position.offset(-1, 0, 0)) == self.id() || game.block_id_at(position.offset(1, 0, 0)) == self.id() {
            i1 = 1;
            j1 = 0;
        }
        let mut k1 = position.y;
        while game.block_id_at(BlockPosition::new(position.x, k1 - 1, position.z, world)) == self.id() {
            k1 -= 1;
        }
        if game.block_id_at(BlockPosition::new(position.x, k1 - 1, position.z, world)) != 49 {
            game.break_block(position, world);
            return Ok(());
        }
        let mut l1 = 1;
        while l1 < 4 && game.block_id_at(BlockPosition::new(position.x, k1 + l1, position.z, world)) == self.id() {
            l1 += 1;
        }
        if l1 != 3 || game.block_id_at(BlockPosition::new(position.x, k1 + l1, position.z, world)) != 49 {
            game.break_block(position, world);
            return Ok(());
        }
        let flag = game.block_id_at(position.offset(-1, 0, 0)) == self.id() || game.block_id_at(position.offset(1, 0, 0)) == self.id();
        let flag1 = game.block_id_at(position.offset(0, 0, -1)) == self.id() || game.block_id_at(position.offset(0, 0, 1)) == self.id();
        if flag && flag1 {
            game.break_block(position, world);
            return Ok(());
        }
        if (game.block_id_at(BlockPosition::new(position.z + i1, position.y, position.z + j1, world)) != 49 || game.block_id_at(BlockPosition::new(position.z - i1, position.y, position.z - j1, world)) != self.id()) && (game.block_id_at(BlockPosition::new(position.z - i1, position.y, position.z - j1, world)) != 49 || game.block_id_at(BlockPosition::new(position.z + i1, position.y, position.z + j1, world)) != self.id()) {
            game.break_block(position, world);
        }
        Ok(())
    }
}
impl PortalBlock {
    pub fn try_create_portal(world: i32, game: &mut Game, mut pos: BlockPosition) -> bool {
        let mut l = 0;
        let mut i1 = 0;
        if game.block_id_at(pos.offset(-1, 0, 0)) == 49 || game.block_id_at(pos.offset(1, 0, 0)) == 49 {
            l = 1;
        }
        if game.block_id_at(pos.offset(0, 0, -1)) == 49 || game.block_id_at(pos.offset(0, 0, 1)) == 49 {
            i1 = 1;
        }
        if l == i1 {
            return false;
        }
        if game.block_id_at(BlockPosition::new(pos.x - l, pos.y, pos.z - i1, world)) == 0 {
            pos.x -= l;
            pos.z -= i1;
        }
        for j1 in -1..3 {
            for l1 in -1..4 {
                let flag = j1 == -1 || j1 == 2 || l1 == -1 || l1 == 3;
                if (j1 == -1 || j1 == 2) && (l1 == -1 || l1 == 3) {
                    continue;
                }
                let j2 = game.block_id_at(BlockPosition::new(pos.x + l * j1, pos.y + l1, pos.z + i1 * j1, world));
                if flag {
                    if j2 != 49 {
                        return false;
                    }
                    continue;
                }
                if j2 != 0 && j2 != 51 {
                    return false;
                }
            }
        }
        game.schedule_next_tick(move |game| {
            for k1 in 0..2 {
                for i2 in 0..3 {
                    game.set_block_nb(BlockPosition::new(pos.x + l * k1, pos.y + i2, pos.z + i1 * k1, world), BlockState::from_id(90), world, false, false, false);
                }
            }
            None
        });
        true
    }
}