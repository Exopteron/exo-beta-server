use std::{sync::Arc, ops::Deref};

use ahash::AHashMap;
use hecs::{Entity, EntityBuilder};
use nbt::CompoundTag;

use crate::{server::Client, game::{Game, BlockPosition}, ecs::{EntityRef, systems::SysResult}, item::item::ItemRegistry, world::chunks::BlockState};

#[derive(Clone)]
pub struct BlockEntitySaver {
    callback: Arc<Box<dyn Fn(&EntityRef) -> anyhow::Result<CompoundTag> + Sync + Send>>,
    pub be_type: String,
}
impl BlockEntitySaver {
    pub fn save(&self, entity: &EntityRef, be_type: &str, position: BlockPosition) -> anyhow::Result<CompoundTag> {
        let mut tag = (self.callback)(entity)?;
        tag.insert_str("id", be_type);
        tag.insert_i32("x", position.x);
        tag.insert_i32("y", position.y);
        tag.insert_i32("z", position.z);
        Ok(tag)
    }
    pub fn new(callback: impl Fn(&EntityRef) -> anyhow::Result<CompoundTag> + Sync + Send + 'static, be_type: String) -> Self {
        Self {
            callback: Arc::new(Box::new(callback)),
            be_type,
        }
    }
}

#[derive(Clone)]
pub struct BlockEntityLoader {
    callback: Arc<Box<dyn Fn(&Client, &EntityRef) -> SysResult + Sync + Send>>
}
impl BlockEntityLoader {
    pub fn load(&self, client: &Client, entity: &EntityRef) -> SysResult {
        (self.callback)(client, entity)
    }
    pub fn new(callback: impl Fn(&Client, &EntityRef) -> SysResult + Sync + Send + 'static) -> Self {
        Self {
            callback: Arc::new(Box::new(callback))
        }
    }
}
#[derive(Debug, Clone)]
pub struct BlockEntity(pub BlockPosition, pub i32);

#[derive(Default, Debug, Clone)]
pub struct SignData(pub [String; 4]);
pub struct NoteblockData(pub i8);
pub type BlockEntityNBTLoader = Box<dyn Fn(&CompoundTag, BlockPosition, &mut EntityBuilder) -> SysResult>;
#[derive(Clone)]
pub struct BlockEntityNBTLoaders {
    loaders: AHashMap<String, Arc<BlockEntityNBTLoader>>,   
}
impl Default for BlockEntityNBTLoaders {
    fn default() -> Self {
        let loaders = AHashMap::new();
        let this = Self { loaders };
        this
    }
}
impl BlockEntityNBTLoaders {
    pub fn run(&self, name: String, tag: &CompoundTag, pos: BlockPosition, builder: &mut EntityBuilder) -> bool {
        if let Some(loader) = self.loaders.get(&name) {
            if let Err(_) = loader(tag, pos, builder) {
                return false;
            }
            return true;
        }
        false
    }
    pub fn insert(&mut self, name: &str, loader: BlockEntityNBTLoader) {
        self.loaders.insert(name.to_string(), Arc::new(loader));
    }
}