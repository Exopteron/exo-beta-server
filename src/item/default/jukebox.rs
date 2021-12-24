use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;

use crate::{
    aabb::{AABBSize, AABB},
    block_entity::{BlockEntity, BlockEntityLoader, BlockEntitySaver, SignData},
    ecs::{entities::player::{Chatbox, SLOT_HOTBAR_OFFSET, HotbarSlot}, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType}, window::Window,
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, server::Server,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct JukeboxBlock;

impl Block for JukeboxBlock {
    fn id(&self) -> BlockIdentifier {
        84
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn interacted_with(
        &self,
        world: i32,
        game: &mut Game,
        server: &mut crate::server::Server,
        position: BlockPosition,
        state: BlockState,
        player: Entity,
    ) -> ActionResult {
        log::debug!("Interacted");
        let entity_ref = game.ecs.entity(player).unwrap();
        let window = entity_ref.get::<Window>().unwrap();
        let hotbar_slot = entity_ref.get::<HotbarSlot>().unwrap();
        let slot = SLOT_HOTBAR_OFFSET + hotbar_slot.get();
        let slot = window.inner().item(slot).unwrap().clone();
        let id = match slot.item_kind() {
            Some(i) => i.id(),
            None => 0,
        };
        if id != 2256 && id != 2257 {
            log::debug!("Not. It is {}", id);
            return ActionResult::SUCCESS;
        }
        drop(window);
        drop(id);
        drop(slot);
        drop(entity_ref);
        log::debug!("Is here");
        if let Some(entity) = game.block_entity_at(position, world) {
            if let Ok(mut data) = game.ecs.get_mut::<JukeboxData>(entity) {
                data.0 = (id - 2255) as i32;
                server.broadcast_nearby_with(position.into(), |cl| {
                    log::debug!("Send");
                    cl.send_effect(position, SoundEffectKind::RecordPlay, id as i32);
                });
            } else {
                log::debug!("None.");
            }
        } else {
            log::debug!("No BE");
        }
        ActionResult::SUCCESS
    }
    fn on_break(&self, game: &mut Game, server: &mut Server, breaker: Entity, mut position: BlockPosition, face: Face, world: i32) {
        server.broadcast_nearby_with(position.into(), |cl| {
            cl.send_effect(position, SoundEffectKind::RecordPlay, 0);
        });
    }
    fn block_entity(
        &self,
        entity_builder: &mut hecs::EntityBuilder,
        state: BlockState,
        position: BlockPosition,
    ) -> bool {
        entity_builder.add(JukeboxData(0));
        entity_builder.add(BlockEntitySaver::new(
            |entity| {
                let mut tag = CompoundTag::new();
                let data = entity.get::<JukeboxData>()?;
                tag.insert_i32("Record", data.0);
                Ok(tag)
            },
            "Record".to_string(),
        ));
        true
    }
    fn block_entity_loader(&self, loaders: &mut crate::block_entity::BlockEntityNBTLoaders) {
        loaders.insert(
            "Record",
            Box::new(|tag, blockpos, builder| {
                log::info!("Note block loader called");
                let mut note_block_data = JukeboxData(0);
                note_block_data.0 = tag
                    .get_i32("Record")
                    .or_else(|_| Err(anyhow::anyhow!("No tag")))?;
                log::info!("Note: {}", note_block_data.0);
                ItemRegistry::global().get_block(84).unwrap().block_entity(
                    builder,
                    BlockState::new(84, 0),
                    blockpos,
                );
                builder.add(note_block_data);
                Ok(())
            }),
        );
    }
}
pub struct JukeboxData(pub i32);