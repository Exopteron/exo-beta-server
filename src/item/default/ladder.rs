use hecs::Entity;

use crate::{
    aabb::{AABB, AABBSize},
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType},
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
// TODO colliding on a ladder should reset fall damage
pub struct LadderBlock {}
impl Block for LadderBlock {
    fn id(&self) -> BlockIdentifier {
        65
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn collision_box(&self, state: BlockState, position: BlockPosition) -> Option<AABB> {
        let f = 0.125;
        let mut aabbsize = AABBSize::new(
            position.x as f64,
            position.y as f64,
            position.z as f64,
            position.x as f64 + 1.,
            position.y as f64 + 1.,
            position.z as f64 + 1.,
        );
        if state.b_metadata == 2 {
            aabbsize.set_bounds(0., 0., 1. - f, 1., 1., 1.);
        }
        if state.b_metadata == 3 {
            aabbsize.set_bounds(0., 0., 0., 1., 1., f);
        }
        if state.b_metadata == 4 {
            aabbsize.set_bounds(1. - f, 0., 0., 1., 1., 1.);
        }
        if state.b_metadata == 5 {
            aabbsize.set_bounds(0., 0., 0., f, 1., 1.);
        }
        Some(aabbsize.get_from_block(&position))
    }
    fn place(
        &self,
        game: &mut Game,
        player: Entity,
        mut item: ItemStack,
        mut position: crate::game::BlockPosition,
        face: crate::protocol::packets::Face,
        world: i32,
    ) -> Option<crate::events::block_interact::BlockPlacementEvent> {
        position = face.offset(position);
        let mut meta = 0;
        if (meta == 0 || matches!(face, Face::NegativeZ))
            && game.is_solid_block(Face::PositiveZ.offset(position), world)
        {
            meta = 2;
        }
        if (meta == 0 || matches!(face, Face::PositiveZ))
            && game.is_solid_block(Face::NegativeZ.offset(position), world)
        {
            meta = 3;
        }
        if (meta == 0 || matches!(face, Face::NegativeX))
            && game.is_solid_block(Face::PositiveX.offset(position), world)
        {
            meta = 4;
        }
        if (meta == 0 || matches!(face, Face::PositiveX))
            && game.is_solid_block(Face::NegativeX.offset(position), world)
        {
            meta = 5;
        }
        item.set_damage(meta);
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }
    fn neighbor_update(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        position: BlockPosition,
        state: crate::world::chunks::BlockState,
        offset: Face,
        neighbor_state: crate::world::chunks::BlockState,
    ) -> SysResult {
        if !matches!(offset, Face::Invalid) {
            let mut f = false;
            if state.b_metadata == 2 && game.is_solid_block(Face::PositiveZ.offset(position), world)
            {
                f = true;
            }
            if state.b_metadata == 3 && game.is_solid_block(Face::NegativeX.offset(position), world)
            {
                f = true;
            }
            if state.b_metadata == 4 && game.is_solid_block(Face::PositiveX.offset(position), world)
            {
                f = true;
            }
            if state.b_metadata == 5 && game.is_solid_block(Face::NegativeX.offset(position), world)
            {
                f = true;
            }
            if !f {
                game.break_block(position, world);
            }
        }
        Ok(())
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn can_place_on(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        face: Face,
    ) -> bool {
        let position = face.offset(position);
        if game.is_solid_block(Face::NegativeX.offset(position), world) {
            return true;
        }
        if game.is_solid_block(Face::PositiveX.offset(position), world) {
            return true;
        }
        if game.is_solid_block(Face::NegativeZ.offset(position), world) {
            return true;
        }
        return game.is_solid_block(Face::PositiveZ.offset(position), world);
    }
    fn opaque(&self) -> bool {
        false
    }
}
