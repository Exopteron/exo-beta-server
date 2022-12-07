use std::{sync::Arc, ops::Deref};

use ahash::AHashMap;
use hecs::{Entity, EntityBuilder};
use nbt::CompoundTag;

use crate::{server::Client, game::{Game, BlockPosition, Position}, ecs::{EntityRef, systems::SysResult}, item::item::ItemRegistry, world::chunks::BlockState, physics::Physics};

#[derive(Clone)]
pub struct RegularEntitySaver {
    callback: Arc<Box<dyn Fn(&EntityRef) -> anyhow::Result<CompoundTag> + Sync + Send>>,
    pub entity_type: String,
}
impl RegularEntitySaver {
    pub fn save(&self, entity: &EntityRef, entity_type: &str) -> anyhow::Result<CompoundTag> {
        let mut tag = (self.callback)(entity)?;
        tag.insert_str("id", entity_type);

        let position = entity.get::<Position>()?;
        tag.insert_f64_vec("Pos", vec![position.x, position.y, position.z]);

        tag.insert_f32_vec("Rotation", vec![position.yaw, position.pitch]);

        tag.insert_bool("OnGround", position.on_ground);

        let v = entity.get::<Physics>()?;
        tag.insert_f64_vec("Motion", v.get_velocity().to_array().to_vec());
        Ok(tag)
    }
    pub fn new(callback: impl Fn(&EntityRef) -> anyhow::Result<CompoundTag> + Sync + Send + 'static, entity_type: String) -> Self {
        Self {
            callback: Arc::new(Box::new(callback)),
            entity_type,
        }
    }
}

#[derive(Clone)]
pub struct RegEntityLoader {
    callback: Arc<Box<dyn Fn(&Client, &EntityRef) -> SysResult + Sync + Send>>
}
impl RegEntityLoader {
    pub fn load(&self, client: &Client, entity: &EntityRef) -> SysResult {
        (self.callback)(client, entity)
    }
    pub fn new(callback: impl Fn(&Client, &EntityRef) -> SysResult + Sync + Send + 'static) -> Self {
        Self {
            callback: Arc::new(Box::new(callback))
        }
    }
}

pub type RegEntityNBTLoader = Box<dyn Fn(&CompoundTag, &mut EntityBuilder) -> SysResult>;
#[derive(Clone)]
pub struct RegEntityNBTLoaders {
    pub loaders: AHashMap<String, Arc<RegEntityNBTLoader>>,   
}
impl Default for RegEntityNBTLoaders {
    fn default() -> Self {
        let loaders = AHashMap::new();
        let this = Self { loaders };
        this
    }
}
impl RegEntityNBTLoaders {
    pub fn run(&self, name: String, tag: &CompoundTag, builder: &mut EntityBuilder) -> anyhow::Result<bool> {
        if let Some(loader) = self.loaders.get(&name) {
            loader(tag, builder)?;
            return Ok(true);
        }
        Ok(false)
    }
    pub fn insert(&mut self, name: &str, loader: RegEntityNBTLoader) {
        self.loaders.insert(name.to_string(), Arc::new(loader));
    }
}