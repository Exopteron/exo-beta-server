use std::{path::PathBuf, fs::File, ops::Deref};

use serde::*;
use hemnbt::Blob;

use crate::{ecs::{systems::SysResult, EntityRef, entities::{living::{Hunger, Health}, player::Gamemode}}, game::Position, physics::Physics, item::{inventory::Inventory, inventory_slot::InventorySlot, stack::ItemStack}};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct NBTItem {
    #[serde(rename = "Slot")]
    pub slot: i8,
    pub id: i16,
    #[serde(rename = "Damage")]
    pub damage: i16,
    #[serde(rename = "Count")]
    pub count: i8,
}
impl From<NBTItem> for ItemStack {
    fn from(other: NBTItem) -> Self {
        ItemStack::new(other.id, other.count, other.damage)
    }
}
#[derive(Serialize, Deserialize)]
pub struct PlayerDat {
    #[serde(rename = "Pos")]
    pub pos: Vec<f64>,
    #[serde(rename = "Motion")]
    pub motion: Vec<f64>,
    #[serde(rename = "Rotation")]
    pub rotation: Vec<f32>,
    #[serde(rename = "Dimension")]
    pub dimension: i32,
    #[serde(rename = "foodLevel")]
    pub food_level: i32,
    #[serde(rename = "foodSaturationLevel")]
    pub food_saturation: f32,
    #[serde(rename = "Health")]
    pub health: i16,
    #[serde(rename = "OnGround")]
    pub on_ground: bool,
    #[serde(rename = "playerGameType")]
    pub game_type: i32,
    #[serde(rename = "Inventory")]
    pub inventory: Vec<NBTItem>,
}

impl PlayerDat {
    pub fn from_file(file: impl Into<PathBuf>) -> anyhow::Result<Self> {
        let mut file = File::open(file.into())?;
        let dat: Self = hemnbt::de::from_reader(&mut file)?;
        Ok(dat)
    }
    pub fn from_entity(entity: &EntityRef) -> anyhow::Result<Self> {
        let pos = *entity.get::<Position>()?;
        let motion = *entity.get::<Physics>()?.get_velocity();
        let food = *entity.get::<Hunger>()?;
        let health = entity.get::<Health>()?.deref().clone();
        let gamemode = *entity.get::<Gamemode>()?;
        let inv = entity.get::<Inventory>()?.new_handle();
        let mut nbt_inv = Vec::new();
        for (slot, item) in inv.to_vec().into_iter().enumerate() {
            if let InventorySlot::Filled(item) = item {
                nbt_inv.push(NBTItem { slot: slot as i8, id: item.id(), damage: item.damage_taken(), count: item.count() });
            }
        }
        Ok(Self {pos: vec![pos.x, pos.y, pos.z], motion: vec![motion.x, motion.y, motion.z], rotation: vec![pos.yaw, pos.pitch], dimension: pos.world, food_level: food.0 as i32, food_saturation: food.1, health: health.0, on_ground: pos.on_ground, game_type: gamemode.id() as i32, inventory: nbt_inv })
    }
    pub fn to_file(&self, file: impl Into<PathBuf>) -> SysResult {
        let file = file.into();
        let mut file = File::create(file)?;
        hemnbt::ser::to_writer(&mut file, self, None)?;
        Ok(())
    }
}