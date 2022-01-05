use hecs::Entity;
mod fence;
mod trapdoor;
mod door;
mod ladder;
mod sign;
mod furnace;
mod sugar_cane;
mod glass;
mod slab;
mod dispenser;
mod cobweb;
mod bush;
mod note_block;
mod music_disc;
mod jukebox;
mod water_bucket;
mod grass_block;
mod ice;
mod vines;
mod pumpkin;
mod chest;
mod lever;
mod fire;
mod portal;
use crate::{
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::stack::ItemStackType,
    protocol::packets::Face,
    world::chunks::BlockState,
};

use self::{fence::{FenceGateBlock, FenceBlock}, trapdoor::TrapdoorBlock, door::{DoorItem, DoorBlock}, ladder::LadderBlock, sign::{SignItem, SignBlock}, furnace::FurnaceBlock, sugar_cane::SugarCane, glass::GlassBlock, slab::SlabBlock, dispenser::DispenserBlock, cobweb::CobwebBlock, bush::GenericBush, note_block::NoteBlock, music_disc::MusicDiscItem, jukebox::JukeboxBlock, water_bucket::WaterBucketItem, grass_block::GrassBlock, ice::IceBlock, vines::VinesBlock, pumpkin::PumpkinBlock, chest::ChestBlock, lever::LeverBlock, fire::{FireBlock, FlintAndSteelItem}, portal::PortalBlock};

use super::item::{block::{Block, fluid::water::{MovingWaterBlock, NotFlowingWaterBlock}, BurnRate}, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct RedstoneTorchBlock {}
impl Block for RedstoneTorchBlock {
    fn id(&self) -> BlockIdentifier {
        76
    }

    fn item_stack_size(&self) -> i8 {
        64
    }

    fn place(
        &self,
        game: &mut Game,
        player: Entity,
        item: super::stack::ItemStack,
        position: BlockPosition,
        face: Face,
        world: i32,
    ) -> Option<BlockPlacementEvent> {
        (TorchBlock {}).place(game, player, item, position, face, world)
    }

    fn is_solid(&self) -> bool {
        false
    }

    fn neighbor_update(
        &self,
        world: i32,
        game: &mut crate::game::Game,
        position: BlockPosition,
        state: BlockState,
        offset: Face,
        neighbor_state: BlockState,
    ) -> SysResult {
        (TorchBlock {}).neighbor_update(world, game, position, state, offset, neighbor_state)
    }
    fn opaque(&self) -> bool {
        false
    }
}
pub struct StairBlock {
    id: BlockIdentifier,
}
impl Block for StairBlock {
    fn place(&self, game: &mut Game, entity: Entity, mut item: super::stack::ItemStack, mut position: BlockPosition, face: Face, world: i32) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        item.set_damage(stair_orient(&mut position, &game.ecs.get::<Position>(entity).unwrap()).into());
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

    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        Ok(())
    }

    fn can_place_on(&self, world: i32, game: &mut Game, position: BlockPosition, face: Face) -> bool {
        true
    }

    fn id(&self) -> BlockIdentifier {
        self.id
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
}
pub struct TorchBlock {}
impl Block for TorchBlock {
    fn id(&self) -> super::item::BlockIdentifier {
        50
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(
        &self,
        game: &mut Game,
        player: Entity,
        mut item: super::stack::ItemStack,
        mut position: crate::game::BlockPosition,
        face: crate::protocol::packets::Face,
        world: i32,
    ) -> Option<crate::events::block_interact::BlockPlacementEvent> {
        position = face.offset(position);
        item.set_damage(torch_orient(&mut position, &face).into());
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
        let back_face = torch_orient_back(state.b_metadata).reverse();
        //log::info!("Pos: {:?}", position);
        let stood_on = back_face.offset(position);
        //log::info!("Pos oriented {:?}: {:?}", back_face, stood_on);
        let us_block = game
            .block(stood_on, world)
            .ok_or(anyhow::anyhow!("No block"))?;
        //log::info!("I am stood on {:?}", us_block);
        let solid = if let Ok(t) = us_block.registry_type() {
            t.is_solid()
        } else {
            false
        };
        if !solid {
            //log::info!("Setting {:?} to air", stood_on);
            let success = game.set_block_nb(position, BlockState::air(), world, true, false);
            //log::info!("Success? {}", success);
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
        let us_block = game
            .block(position, world);
        if let Some(us_block) = us_block {
            if let Ok(us_block) = us_block.registry_type() {
                if us_block.is_solid() {
                    return true;
                }
            }
        }
        false
    }
    fn opaque(&self) -> bool {
        false
    }
}
fn torch_orient_back(input: u8) -> Face {
    match input {
        5 => Face::PositiveY,
        4 => Face::NegativeZ,
        3 => Face::PositiveZ,
        2 => Face::NegativeX,
        1 => Face::PositiveX,
        _ => Face::Invalid,
    }
}
fn stair_orient(pos: &mut BlockPosition, placer_pos: &Position) -> u8 {
    let l = (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 3;
    match l {
        0 => {
            2
        }
        1 => {
            1
        }
        2 => {
            3
        }
        _ => {
            0
        }
    }
}
fn torch_orient(pos: &mut BlockPosition, face: &Face) -> u8 {
    match face {
        Face::PositiveY => 5,
        Face::NegativeZ => 4,
        Face::PositiveZ => 3,
        Face::NegativeX => 2,
        Face::PositiveX => 1,
        _ => 0,
    }
}
pub struct GenericNonSolidBlock {
    id: BlockIdentifier,
}
impl Block for GenericNonSolidBlock {
    fn id(&self) -> super::item::BlockIdentifier {
        self.id.clone()
    }

    fn item_stack_size(&self) -> i8 {
        64
    }

    fn is_solid(&self) -> bool {
        false
    }
}
pub struct GenericSolidBlock {
    id: BlockIdentifier,
}
impl Block for GenericSolidBlock {
    fn id(&self) -> super::item::BlockIdentifier {
        self.id.clone()
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
}
pub struct AirBlock {}
impl Block for AirBlock {
    fn id(&self) -> BlockIdentifier {
        0
    }

    fn item_stack_size(&self) -> i8 {
        0
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn can_place_over(&self) -> bool {
        true
    }
    fn opaque(&self) -> bool {
        false
    }
}
pub struct LogBlock;
impl Block for LogBlock {
    fn id(&self) -> BlockIdentifier {
        17
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn burn_rate(&self) -> Option<super::item::block::BurnRate> {
        Some(BurnRate(5, 5))
    }
}
pub struct GenericBurnable(pub BlockIdentifier);
impl Block for GenericBurnable {
    fn id(&self) -> BlockIdentifier {
        self.0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn burn_rate(&self) -> Option<BurnRate> {
        Some(BurnRate(50, 50))
    }
}
pub fn register_items(registry: &mut ItemRegistry) {
    registry.register_block(GenericBurnable(1));
    registry.register_block(GenericBurnable(3));
    registry.register_block(TorchBlock {});
    registry.register_block(AirBlock {});
    registry.register_block(RedstoneTorchBlock {});
    registry.register_block(GenericSolidBlock { id: 7 });
    registry.register_block(StairBlock { id: 53 });
    registry.register_block(StairBlock { id: 67 });
    registry.register_block(StairBlock { id: 108 });
    registry.register_block(StairBlock { id: 109 });
    registry.register_block(FenceGateBlock {});
    registry.register_block(TrapdoorBlock {});
    registry.register_item(DoorItem(324));
    registry.register_item(DoorItem(330));
    registry.register_block(DoorBlock {});
    registry.register_block(LadderBlock {});
    registry.register_item(SignItem {});
    registry.register_block(SignBlock { id: 63 });
    registry.register_block(SignBlock { id: 68 });
    registry.register_block(FurnaceBlock(61));
    registry.register_block(FurnaceBlock(62));
    registry.register_block(SugarCane);
    registry.register_block(GlassBlock);
    registry.register_block(SlabBlock);
    registry.register_block(DispenserBlock);
    registry.register_block(CobwebBlock);
    registry.register_block(GenericBush(6));
    registry.register_block(GenericBush(31));
    registry.register_block(GenericBush(32));
    registry.register_block(GenericBush(37));
    registry.register_block(GenericBush(38));
    registry.register_block(GenericBush(39));
    registry.register_block(GenericBush(40));
    registry.register_block(NoteBlock);
    registry.register_item(MusicDiscItem(2256));
    registry.register_item(MusicDiscItem(2257));
    registry.register_block(JukeboxBlock);
    registry.register_block(GrassBlock);
    registry.register_block(FenceBlock);
    registry.register_block(IceBlock);
    registry.register_block(VinesBlock);
    registry.register_block(PumpkinBlock(86));
    registry.register_block(PumpkinBlock(91));
    registry.register_block(ChestBlock);
    registry.register_block(LeverBlock);
    registry.register_block(FireBlock);
    registry.register_item(FlintAndSteelItem);
    registry.register_block(LogBlock);
    registry.register_block(PortalBlock);
    //registry.register_block(MovingWaterBlock(8));
    //registry.register_block(NotFlowingWaterBlock);
    //registry.register_item(WaterBucketItem);
    for i in 1..255 {
        if i == 50 || i == 7 {
            continue;
        }
        registry.register_block(GenericSolidBlock { id: i });
    }
    /*     for i in 0..111 {
        registry.register_item(Box::new(GenericBlock { id: (i, 0) }));
    }
    for i in 0..16 {
        registry.register_item(Box::new(GenericBlock { id: (35, i) })); // Wool
    }
    registry.register_item(Box::new(GenericBlock { id: (17, 1) })); // Spruce Log
    registry.register_item(Box::new(GenericBlock { id: (17, 2) })); // Birch Log
    registry.register_item(Box::new(GenericBlock { id: (20, 0) })); // Glass
    registry.register_item(Box::new(GenericBlock { id: (323, 0) })); // Glass */
}
