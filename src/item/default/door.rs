use parking_lot::MutexGuard;

use crate::{item::{item::{block::{Block, ActionResult}, ItemRegistry, Item, ItemIdentifier}, stack::{ItemStackType, ItemStack}, window::Window, inventory_slot::InventorySlot}, protocol::packets::{Face, SoundEffectKind}, events::block_interact::BlockPlacementEvent, world::chunks::BlockState, ecs::{systems::SysResult, entities::player::Gamemode}, game::{Position, BlockPosition}, network::ids::NetworkID, server::Server};

pub struct DoorItem(pub ItemIdentifier);
impl Item for DoorItem {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        self.0
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(&self, game: &mut crate::game::Game, server: &mut Server, mut item: MutexGuard<InventorySlot>, slot: usize, user: hecs::Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        // TODO reduce in gms
        if let Some(target) = target {
            let block_pos = target.face.offset(target.position);
            let mut good_pos = false;
            if let Some(block) = game.block(block_pos, target.world) {
                if !block.is_solid() {
                    good_pos = true;
                } else {
                    good_pos = false;
                }
            }
            if let Some(block) = game.block(Face::PositiveY.offset(block_pos), target.world) {
                if !block.is_solid() {
                    good_pos = true;
                } else {
                    good_pos = false;
                }
            }
            if good_pos {
                let pos = game.ecs.get::<Position>(user)?;
                let mut val = fd(((((pos.yaw + 180.0) * 4.0) / 360.0) as f64) - 0.5) & 3;
                drop(pos);
                let mut b0 = 0;
                let mut b1 = 0;
                match val {
                    0 => {
                        b1 = 1;
                    },
                    1 => {
                        b0 = -1;
                    },
                    2 => {
                        b1 = -1;
                    },
                    3 => {
                        b0 = 1;
                    }
                    _ => (),
                }
                let num1 = game.is_solid_block(BlockPosition::new(block_pos.x - b0, block_pos.y, block_pos.z - b1, target.world), target.world) as u8 +  game.is_solid_block(BlockPosition::new(block_pos.x - b0, block_pos.y + 1, block_pos.z - b1, target.world), target.world) as u8;
                let num2 = game.is_solid_block(BlockPosition::new(block_pos.x + b0, block_pos.y, block_pos.z + b1, target.world), target.world) as u8 +  game.is_solid_block(BlockPosition::new(block_pos.x + b0, block_pos.y + 1, block_pos.z + b1, target.world), target.world) as u8;
                let f = game.block(BlockPosition::new(block_pos.x - b0, block_pos.y, block_pos.z - b1, target.world), target.world).unwrap_or(BlockState::air()).b_type == 64 || game.block(BlockPosition::new(block_pos.x - b0, block_pos.y + 1, block_pos.z - b1, target.world), target.world).unwrap_or(BlockState::air()).b_type == 64;
                let f1 = game.block(BlockPosition::new(block_pos.x + b0, block_pos.y, block_pos.z + b1, target.world), target.world).unwrap_or(BlockState::air()).b_type == 64 || game.block(BlockPosition::new(block_pos.x + b0, block_pos.y + 1, block_pos.z + b1, target.world), target.world).unwrap_or(BlockState::air()).b_type == 64;
                let mut f2 = false;
                if f && !f1 || num1 > num2 {
                    f2 = true;
                }
                if f2 {
                    val -= 1 & 3;
                    val += 4;
                }
                game.set_block(block_pos, BlockState::new(64, val as u8), target.world);
                game.set_block(Face::PositiveY.offset(block_pos), BlockState::new(64, (val + 8) as u8), target.world);
                let entity_ref = game.ecs.entity(user)?;
                if *entity_ref.get::<Gamemode>()? != Gamemode::Creative {
                    let window = entity_ref.get::<Window>()?;
                    let id = entity_ref.get::<NetworkID>()?;
                    item.try_take(1);
                    drop(item);
                    let client = server.clients.get(&id).unwrap();
                    client.send_window_items(&window);
                }
            }
        }
        Ok(())
    }
}

pub struct DoorBlock {}
impl Block for DoorBlock {
    fn dropped_items(&self, state: BlockState, held_item: InventorySlot) -> Vec<crate::item::stack::ItemStack> {
        vec![ItemStack::new(324, 1, 0)]
    }
    fn opaque(&self) -> bool {
        false
    }
    fn neighbor_update(&self, world: i32, game: &mut crate::game::Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        if !matches!(offset, Face::Invalid) {
            if (state.b_metadata & 8) != 0 {
                if let Some(b) = game.block(Face::NegativeY.offset(position), world) {
                    if b.b_type != self.id() {
                        game.break_block(position, world);
                    }
                }
            } else {
                let mut f = false;
                if let Some(b) = game.block(Face::PositiveY.offset(position), world) {
                    if b.b_type != self.id() {
                        game.break_block(position, world);
                        f = true;
                    }
                }
                if !game.is_solid_block(Face::NegativeY.offset(position), world) {
                    game.break_block(position, world);
                    f = true;
                    if game.block_id_at(Face::PositiveY.offset(position)) == self.id() {
                        game.break_block(Face::PositiveY.offset(position), world);   
                    }
                }
                if f {
                    // drop
                }
            }
        }
        Ok(())
    }
    fn id(&self) -> crate::item::item::BlockIdentifier {
        64
    }

    fn item_stack_size(&self) -> i8 {
        1
    }
    fn interacted_with(&self, world: i32, game: &mut crate::game::Game, server: &mut crate::server::Server, position: BlockPosition, state: BlockState, player: hecs::Entity) -> anyhow::Result<crate::item::item::block::ActionResult> {
        if (state.b_metadata & 8) != 0 {
            if let Some(b) = game.block(Face::NegativeY.offset(position), world) {
                if b.b_type == self.id() {
                    self.interacted_with(world, game, server, Face::NegativeY.offset(position), b, player);
                }
            }
            return Ok(ActionResult::SUCCESS);
        }
        if let Some(b) = game.block(Face::PositiveY.offset(position), world) {
            if b.b_type == self.id() {
                game.set_block(Face::PositiveY.offset(position), BlockState::new(self.id(), (state.b_metadata ^ 4) + 8), world);
            }
        }
        game.set_block(position, BlockState::new(self.id(), state.b_metadata ^ 4), world);
        let id = game.ecs.get::<NetworkID>(player).unwrap();
        server.broadcast_effect_from_entity(*id, SoundEffectKind::DoorToggle, position, world, 0);
        Ok(ActionResult::SUCCESS)
    }
}

pub fn fd(input: f64) -> i32 {
    let v = input as i32;
    if input < v as f64 {
        return v - 1;
    } else {
        return v;
    }
}