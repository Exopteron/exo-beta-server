use std::ops::Deref;

use hecs::Entity;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{item::block::ActionResult, stack::ItemStackType},
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
fn inv(input: i32) -> i32 {
    match input {
        0 => 0,
        1 => 5,
        2 => 4,
        3 => 3,
        4 => 2,
        5 => 1,
        _ => -1
    }
}
pub struct LeverBlock;
impl Block for LeverBlock {
    fn opaque(&self) -> bool {
        false
    }
    fn place(
        &self,
        game: &mut Game,
        entity: Entity,
        mut item: crate::item::stack::ItemStack,
        mut position: BlockPosition,
        face: Face,
        world: i32,
    ) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        let placer_pos = game.ecs.get::<Position>(entity).unwrap().deref().clone();
        let j1 = item.damage_taken() & 8;
        let mut b0 = -1;
        if matches!(face, Face::NegativeY) && game.is_solid_block(position.offset(0, 1, 0)) {
            b0 = 0;
        }
        if matches!(face, Face::PositiveY) && game.is_solid_block(position.offset(0, -1, 0)) {
            b0 = 5;
        }
        if matches!(face, Face::NegativeZ) && game.is_solid_block(position.offset(0, 0, 1)) {
            b0 = 4;
        }
        if matches!(face, Face::PositiveZ) && game.is_solid_block(position.offset(0, 0, -1)) {
            b0 = 3;
        }
        if matches!(face, Face::NegativeX) && game.is_solid_block(position.offset(1, 0, 0)) {
            b0 = 2;
        }
        if matches!(face, Face::PositiveX) && game.is_solid_block(position.offset(-1, 0, 0)) {
            b0 = 1;
        }
        let b0 = b0 + j1;
        item.set_damage(b0);
        let i1 = b0 & 7;
        let j1 = b0 & 8;
        if i1 == inv(1) as i16 {
            if (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 1 == 0 {
                item.set_damage(5 | j1 as i16);
            } else {
                item.set_damage(6 | j1 as i16);
            }
        } else if i1 == inv(0) as i16 {
            if (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 1 == 0 {
                item.set_damage(7 | j1 as i16);
            } else {
                item.set_damage(0 | j1 as i16);
            }
        }
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }

    fn is_solid(&self) -> bool {
        false
    }

    fn neighbor_update(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        state: BlockState,
        offset: Face,
        neighbor_state: BlockState,
    ) -> SysResult {
        if self.is_attached(world, game, position) {
            let i1 = state.b_metadata & 7;
            let mut f = false;
            if !game.is_solid_block(position.offset(-1, 0, 0)) && i1 == 1 {
                f = true;
            }
            if !game.is_solid_block(position.offset(1, 0, 0)) && i1 == 2 {
                f = true;
            }
            if !game.is_solid_block(position.offset(0, 0, -1)) && i1 == 3 {
                f = true;
            }
            if !game.is_solid_block(position.offset(0, 0, 1)) && i1 == 4 {
                f = true;
            }
            if !game.is_solid_block(position.offset(0, -1, 0)) && i1 == 5 {
                f = true;
            }
            if !game.is_solid_block(position.offset(0, -1, 0)) && i1 == 6 {
                f = true;
            }
            if !game.is_solid_block(position.offset(0, 1, 0)) && i1 == 0 {
                f = true;
            }
            if !game.is_solid_block(position.offset(0, 1, 0)) && i1 == 7 {
                f = true;
            }
            if f {
                game.break_block(position, world);
            }
        }
        Ok(())
    }
    fn can_place_on(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        face: Face,
    ) -> bool {
        let position = face.offset(position);
        if game.is_solid_block(position.offset(-1, 0, 0)) {
            return true;
        }
        if game.is_solid_block(position.offset(1, 0, 0)) {
            return true;
        }
        if game.is_solid_block(position.offset(0, 0, -1)) {
            return true;
        }
        if game.is_solid_block(position.offset(0, 0, 1)) {
            return true;
        }
        return game.is_solid_block(position.offset(0, -1, 0))
    }

    fn id(&self) -> BlockIdentifier {
        69
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn interacted_with(
        &self,
        world: i32,
        game: &mut Game,
        server: &mut crate::server::Server,
        position: BlockPosition,
        mut state: BlockState,
        player: Entity,
    ) -> anyhow::Result<ActionResult> {
        let v = state.b_metadata;
        let j1 = v & 7;
        let k1 = 8 - (v & 8);
        state.b_metadata = j1 + k1;
        game.set_block(position, state, world);
        //server.broadcast_effect(SoundEffectKind::, position, 0);
        Ok(ActionResult::SUCCESS)
    }
}
impl LeverBlock {
    fn is_attached(&self, world: i32, game: &mut Game, position: BlockPosition) -> bool {
        if !self.can_place_on(world, game, position, Face::Invalid) {
            game.break_block(position, world);
            return false;
        }
        true
    }
}
fn trapdoor_orient(pos: &mut BlockPosition, face: &Face) -> u8 {
    match face {
        Face::NegativeZ => 0,
        Face::PositiveZ => 1,
        Face::NegativeX => 2,
        Face::PositiveX => 3,
        _ => 0,
    }
}
