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
pub struct TrapdoorBlock {}
impl Block for TrapdoorBlock {
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
            trapdoor_orient(&mut position, &face).into(),
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
        96
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
    ) -> ActionResult {
        state.b_metadata ^= 4;
        game.set_block(position, state, world);
        let id = game.ecs.get::<NetworkID>(player).unwrap();
        server.broadcast_effect_from_entity(*id, SoundEffectKind::DoorToggle, position, world, 0);
        ActionResult::SUCCESS
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
