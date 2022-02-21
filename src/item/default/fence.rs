use hecs::Entity;

use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::ItemStackType, item::block::{ActionResult, BurnRate}},
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, network::ids::NetworkID,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
pub struct FenceBlock;
impl Block for FenceBlock {
    fn id(&self) -> BlockIdentifier {
        85
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn opaque(&self) -> bool {
        false
    }
    fn burn_rate(&self) -> Option<crate::item::item::block::BurnRate> {
        Some(BurnRate(5, 20))
    }
}
pub struct FenceGateBlock {}
impl Block for FenceGateBlock {
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
        item.set_damage(
            gate_orient(&mut position, &game.ecs.get::<Position>(entity).unwrap()).into(),
        );
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }

    fn is_solid(&self) -> bool {
        true
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
        Ok(())
    }

    fn can_place_on(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        face: Face,
    ) -> bool {
        true
    }

    fn id(&self) -> BlockIdentifier {
        107
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
        if (state.b_metadata & 4) != 0 {
            state.b_metadata = (state.b_metadata as i8 & -5) as u8;
            game.set_block(position, state, world);
        } else {
            let l = ((((((game.ecs.get::<Position>(player).unwrap().yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 3) % 4;    
            let direction = state.b_metadata & 3;
            if direction as i32 == (l + 2) % 4 {
                state.b_metadata = l as u8;
            }
            state.b_metadata |= 4;
            game.set_block(position, state, world);
        }
        let id = game.ecs.get::<NetworkID>(player).unwrap();
        server.broadcast_effect_from_entity(*id, SoundEffectKind::DoorToggle, position, world, 0);
        Ok(ActionResult::SUCCESS)
    }
}
fn gate_orient(pos: &mut BlockPosition, placer_pos: &Position) -> u8 {
    let l = (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 3;
    match l {
        0 => 0,
        1 => 1,
        2 => 0,
        _ => 1,
    }
}
