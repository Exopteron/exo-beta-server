use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;

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
    world::chunks::BlockState, block_entity::{BlockEntityLoader, SignData, BlockEntity, BlockEntitySaver, NoteblockData},
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct NoteBlock;

impl Block for NoteBlock {
    fn id(&self) -> BlockIdentifier {
        25
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn is_solid(&self) -> bool {
        false
    }
    fn interacted_with(&self, world: i32, game: &mut Game, server: &mut crate::server::Server, position: BlockPosition, state: BlockState, player: Entity) -> anyhow::Result<ActionResult> {
        if let Some(entity) = game.block_entity_at(position) {
            if let Ok(mut data) = game.ecs.get_mut::<NoteblockData>(entity) {
                server.broadcast_nearby_with(position.into(), |cl| {
                    cl.send_block_action(position, 0, data.0);
                });
                data.0 += 1;
                data.0 %= 24;
            }
        }
        Ok(ActionResult::SUCCESS)
    }
    fn block_entity(&self, entity_builder: &mut hecs::EntityBuilder, state: BlockState, position: BlockPosition) -> bool {
        entity_builder.add(NoteblockData(0));
        entity_builder.add(BlockEntitySaver::new(|entity| {
            let mut tag = CompoundTag::new();
            let data = entity.get::<NoteblockData>()?;
            tag.insert_i8("note", data.0);
            Ok(tag)
        }, "Music".to_string()));
        true
    }
    fn block_entity_loader(&self, loaders: &mut crate::block_entity::BlockEntityNBTLoaders) {
        loaders.insert("Music", Box::new(|tag, blockpos, builder| {
            log::info!("Note block loader called");
            let mut note_block_data = NoteblockData(0);
            note_block_data.0 = tag.get_i8("note").or_else(|_| Err(anyhow::anyhow!("No tag {} {}", line!(), file!())))?;
            log::info!("Note: {}", note_block_data.0);
            ItemRegistry::global().get_block(25).unwrap().block_entity(builder, BlockState::new(25, 0), blockpos);
            builder.add(note_block_data);
            Ok(())
        }));
    }
}