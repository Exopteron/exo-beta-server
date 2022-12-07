use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;
use parking_lot::MutexGuard;

use crate::{
    aabb::{AABB, AABBSize},
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType}, inventory_slot::InventorySlot,
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, block_entity::{BlockEntityLoader, SignData, BlockEntity, BlockEntitySaver},
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

use super::door::fd;
pub struct SignItem {}
impl Item for SignItem {
    fn id(&self) -> ItemIdentifier {
        323
    }

    fn stack_size(&self) -> i8 {
        64
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(&self, game: &mut Game, server: &mut crate::server::Server, item: MutexGuard<InventorySlot>, slot: usize, user: Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        if let Some(mut target) = target {
            if matches!(target.face, Face::Invalid) || matches!(target.face, Face::NegativeY) {
                return Ok(());
            }
            if !game.is_solid_block(target.position) {
                return Ok(());
            }
            match target.face {
                Face::PositiveY => {
                    target.position.y += 1;
                },
                Face::NegativeZ => {
                    target.position.z -= 1;
                }
                Face::Invalid => unreachable!(),
                Face::NegativeY => (),
                Face::PositiveZ => {
                    target.position.z += 1;
                }
                Face::NegativeX => {
                    target.position.x -= 1;
                }
                Face::PositiveX => {
                    target.position.x += 1;
                }
            }
            if game.is_solid_block(target.position) {
                return Ok(());
            }
            if matches!(target.face, Face::PositiveY) {
                let pos = *game.ecs.get::<Position>(user)?;
                game.set_block(target.position, BlockState::new(63, (fd(((((pos.yaw + 180.0) * 16.0) / 360.0) as f64) + 0.5) & 0xf) as u8), target.world);
            } else {
                game.set_block(target.position, BlockState::new(68, ((target.face as i8) - 1) as u8), target.world);
            }
        }
        Ok(())
    }
}

pub struct SignBlock {
    pub id: BlockIdentifier,
}
impl Block for SignBlock {
    fn id(&self) -> BlockIdentifier {
        self.id
    }
    fn opacity(&self) -> u8 {
        0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }

    fn is_solid(&self) -> bool {
        false
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
            if self.id == 63 {
                if !game.is_solid_block(Face::NegativeY.offset(position))
                {
                    f = true;
                }    
            } else {
                f = true;
                if state.b_metadata == 2 && game.is_solid_block(Face::PositiveZ.offset(position))
                {
                    f = false;
                }
                if state.b_metadata == 3 && game.is_solid_block(Face::NegativeX.offset(position))
                {
                    f = false;
                }
                if state.b_metadata == 4 && game.is_solid_block(Face::PositiveX.offset(position))
                {
                    f = false;
                }
                if state.b_metadata == 5 && game.is_solid_block(Face::NegativeX.offset(position))
                {
                    f = false;
                }
            }
            if f {
                game.break_block(position, world);
            }
        }
        Ok(())
    }
    // bug: TODO
    fn block_entity(&self, entity_builder: &mut hecs::EntityBuilder, state: BlockState, position: BlockPosition) -> bool {
        entity_builder.add(SignData::default());
        let component = BlockEntityLoader::new(|client, entity| {
            let data = entity.get::<SignData>()?;
            let pos = entity.get::<BlockEntity>()?.0;
            log::info!("Sign loader sending {:?} to {}", *data, client.username());
            client.update_sign(pos, data.deref().clone());
            Ok(())
        });
        entity_builder.add(BlockEntitySaver::new(|entity| {
            let mut tag = CompoundTag::new();
            let data = entity.get::<SignData>()?;
            tag.insert_str("Text1", data.0[0].clone());
            tag.insert_str("Text2", data.0[1].clone());
            tag.insert_str("Text3", data.0[2].clone());
            tag.insert_str("Text4", data.0[3].clone());
            Ok(tag)
        }, "Sign".to_string()));
        entity_builder.add(component);
        true
    }
    fn block_entity_loader(&self, loaders: &mut crate::block_entity::BlockEntityNBTLoaders) {
        loaders.insert("Sign", Box::new(|tag, blockpos, builder| {
            //log::info!("Sign loader called");
            let mut sign_data = SignData::default();
            sign_data.0[0] = tag.get_str("Text1").or_else(|_| Err(anyhow::anyhow!("No tag {} {}", line!(), file!())))?.to_string();
            sign_data.0[1] = tag.get_str("Text2").or_else(|_| Err(anyhow::anyhow!("No tag {} {}", line!(), file!())))?.to_string();
            sign_data.0[2] = tag.get_str("Text3").or_else(|_| Err(anyhow::anyhow!("No tag {} {}", line!(), file!())))?.to_string();
            sign_data.0[3] = tag.get_str("Text4").or_else(|_| Err(anyhow::anyhow!("No tag {} {}", line!(), file!())))?.to_string();
            //log::info!("Sign data: {:?}", sign_data);
            ItemRegistry::global().get_block(63).unwrap().block_entity(builder, BlockState::new(63, 0), blockpos);
            builder.add(sign_data);
            Ok(())
        }));
    }
    fn opaque(&self) -> bool {
        false
    }
}