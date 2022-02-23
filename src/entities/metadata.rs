//! This module implements the entity
//! metadata format. See <https://wiki.vg/Entity_metadata>
//! for the specification.

pub type OptChat = Option<String>;
pub type OptVarInt = Option<i32>;

// Meta index constants.
pub const META_INDEX_ENTITY_BITMASK: u8 = 0;
pub const META_INDEX_AIR: u8 = 1;
pub const META_INDEX_CUSTOM_NAME: u8 = 2;
pub const META_INDEX_IS_CUSTOM_NAME_VISIBLE: u8 = 3;
pub const META_INDEX_IS_SILENT: u8 = 4;
pub const META_INDEX_NO_GRAVITY: u8 = 5;

pub const META_INDEX_POSE: u8 = 6;

pub const META_INDEX_FALLING_BLOCK_SPAWN_POSITION: u8 = 7;
use std::collections::BTreeMap;

use bitflags::bitflags;
bitflags! {
    pub struct EntityBitMask: u8 {
        const ON_FIRE = 0x01;
        const CROUCHED = 0x02;
        const RIDING = 0x04;
        const SPRINTING = 0x08;
        const EATING = 0x10;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MetaEntry {
    Byte(i8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    VillagerData(i32, i32, i32),
}

impl MetaEntry {
    pub fn id(&self) -> i32 {
        match self {
            MetaEntry::Byte(_) => 0,
            MetaEntry::Short(_) => 1,
            MetaEntry::Int(_) => 2,
            MetaEntry::Float(_) => 3,
            MetaEntry::String(_) => 4,
            MetaEntry::VillagerData(_, _, _) => 6,
        }
    }
}



pub trait ToMetaEntry {
    fn to_meta_entry(&self) -> MetaEntry;
}

impl ToMetaEntry for u8 {
    fn to_meta_entry(&self) -> MetaEntry {
        MetaEntry::Byte(*self as i8)
    }
}

impl ToMetaEntry for i8 {
    fn to_meta_entry(&self) -> MetaEntry {
        MetaEntry::Byte(*self)
    }
}

impl ToMetaEntry for i32 {
    fn to_meta_entry(&self) -> MetaEntry {
        MetaEntry::Int(*self)
    }
}

impl ToMetaEntry for f32 {
    fn to_meta_entry(&self) -> MetaEntry {
        MetaEntry::Float(*self)
    }
}


#[derive(Clone, Debug)]
pub struct EntityMetadata {
    pub values: BTreeMap<u8, MetaEntry>,
}

impl EntityMetadata {
    pub fn new() -> Self {
        Self {
            values: BTreeMap::new(),
        }
    }

    /// Returns an entity metadata with the defaults for an `Entity`.
    pub fn entity_base() -> Self {
        Self::new()
            .with(META_INDEX_ENTITY_BITMASK, EntityBitMask::empty().bits())
            .with(META_INDEX_AIR, 0i32)
    }

    pub fn with_many(mut self, values: &[(u8, MetaEntry)]) -> Self {
        for val in values {
            self.values.insert(val.0, val.1.clone());
        }

        self
    }

    pub fn set(&mut self, index: u8, entry: impl ToMetaEntry) {
        self.values.insert(index, entry.to_meta_entry());
    }

    pub fn with(mut self, index: u8, entry: impl ToMetaEntry) -> Self {
        self.set(index, entry);
        self
    }

    pub fn get(&self, index: u8) -> Option<MetaEntry> {
        self.values.get(&index).cloned()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u8, &MetaEntry)> {
        self.values.iter().map(|(key, entry)| (*key, entry))
    }
}

impl Default for EntityMetadata {
    fn default() -> Self {
        Self::new()
    }
}
