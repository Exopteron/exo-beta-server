use std::ops::Deref;

use hecs::Entity;
use nbt::CompoundTag;
use parking_lot::MutexGuard;

use crate::{
    aabb::{AABBSize, AABB},
    block_entity::{BlockEntity, BlockEntityLoader, BlockEntitySaver, SignData},
    ecs::{entities::player::{Chatbox, Player, Sleeping}, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        inventory_slot::InventorySlot,
        item::block::ActionResult,
        stack::{ItemStack, ItemStackType},
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind, EntityAnimationType},
    world::chunks::BlockState, server::Server, sleep, translation::TranslationManager,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

pub struct BedItem;
impl Item for BedItem {
    fn id(&self) -> ItemIdentifier {
        355
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        None
    }
    fn on_use(
        &self,
        game: &mut Game,
        server: &mut crate::server::Server,
        item: MutexGuard<InventorySlot>,
        slot: usize,
        user: Entity,
        target: Option<crate::item::item::BlockUseTarget>,
    ) -> SysResult {
        if let Some(mut target) = target {
            if !matches!(target.face, Face::PositiveY) {
                return Ok(());
            }
            target.position.y += 1;
            let pos = *game.ecs.get::<Position>(user)?;
            let i1 = (((pos.yaw * 4.) / 360.) + 0.5).floor() as i32 & 3;
            let mut byte0 = 0;
            let mut byte1 = 0;
            match i1 {
                0 => byte1 = 1,
                1 => byte0 = -1,
                2 => byte1 = -1,
                3 => byte0 = 1,
                _ => (),
            }
            if game.block_id_at(target.position) == 0
                && game.block_id_at(target.position.offset(byte0, 0, byte1)) == 0
                && game.is_solid_block(target.position.offset(0, -1, 0))
                && game.is_solid_block(target.position.offset(byte0, -1, byte1))
            {
                game.set_block_nb(target.position, BlockState::new(26, i1 as u8), target.position.world, false, false, false);
                game.set_block_nb(target.position.offset(byte0, 0, byte1), BlockState::new(26, (i1 + 8) as u8), target.position.world, false, false, false);
            }
        }
        Ok(())
    }
}

pub const NIGHT_START: i64 = 12542;
pub const NIGHT_END: i64 = 23459;

pub struct BedBlock;

impl Block for BedBlock {
    fn id(&self) -> BlockIdentifier {
        26
    }

    fn item_stack_size(&self) -> i8 {
        0
    }

    fn opacity(&self) -> u8 {
        0
    }

    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        log::info!("updated");
        let meta = state.b_metadata;
        let j1 = (meta & 3) as usize;

        let a;
        let b;
        let mut do_it = false;
        if Self::is_foot(meta) {

            a = position.offset(-MAP[j1][0], 0, -MAP[j1][1]);
            b = position;

            let block = game.block(position.offset(-MAP[j1][0], 0, -MAP[j1][1]), position.world).map(|v| v.b_type);
            if block != Some(self.id()) {
                game.break_block(position, position.world);


                do_it = true;
            }
        } else {

            a = position.offset(MAP[j1][0], 0, MAP[j1][1]);
            b = position;

            let block = game.block(position.offset(MAP[j1][0], 0, MAP[j1][1]), position.world).map(|v| v.b_type);
            if block != Some(self.id()) {
                game.break_block(position, position.world);
                do_it = true;
            }
        }



        if do_it {
            let server = game.objects.get_mut::<Server>()?;

            for (_, (sleeping, id, pos)) in game.ecs.query::<(&mut Sleeping, &NetworkID, &Position)>().iter() {
                if let Some(c) = sleeping.bed_coords() {
                    if c == a || c == b {
                        sleeping.unset_sleeping();
                        server.broadcast_nearby_with(*pos, |cl| {
                            cl.send_entity_animation(*id, EntityAnimationType::LeaveBed);
                        });
                        server.clients.get(id).unwrap().wake_up_sleeping();
                    }
                }
            }
        }


        Ok(())
    }



    fn interacted_with(&self, world: i32, game: &mut Game, server: &mut crate::server::Server, mut position: BlockPosition, state: BlockState, player: Entity) -> anyhow::Result<ActionResult> {


        let range = (NIGHT_START..NIGHT_END);
        if !range.contains(&(game.worlds.get(&world).map(|v| v.level_dat.lock().time)).unwrap()) {
            let translation = game.objects.get::<TranslationManager>().unwrap();
            game.ecs.get_mut::<Chatbox>(player)?.send_message(translation.translate("tile.bed.noSleep", None).into());
            return Ok(ActionResult::PASS);
        }
        let mut l = state.b_metadata;
        if !Self::is_foot(l) {
            let i1 = (l & 3) as usize;
            position.x += MAP[i1][0];
            position.z += MAP[i1][1];
            if game.block_id_at(position) != self.id() {
                return Ok(ActionResult::PASS);
            }
            l = game.block_meta_at(position, position.world);
        }
        if Self::is_occupied(l) {
            let mut has_player = false;
            for (_, (_, sleeping)) in game.ecs.query::<(&Player, &Sleeping)>().iter() {
                if sleeping.is_sleeping() {
                    if let Some(coords) = sleeping.bed_coords() {
                        if coords.x == position.x && coords.y == position.y && coords.z == position.z {
                            has_player = true;
                            break;
                        }
                    }
                }
            }
            if !has_player {
                Self::set_occupied(position, game, false);
            } else {
                let translation = game.objects.get::<TranslationManager>().unwrap();
                game.ecs.get_mut::<Chatbox>(player)?.send_message(translation.translate("tile.bed.occupied", None).into());
                return Ok(ActionResult::SUCCESS);
            }
        }
        Self::set_occupied(position, game, true);
        let mut sleeping = game.ecs.get_mut::<Sleeping>(player)?;
        sleeping.set_sleeping(position);
        Ok(ActionResult::SUCCESS)

    }

}
impl BedBlock {
    pub fn is_foot(num: u8) -> bool {
        (num & 8) != 0
    }
    pub fn is_occupied(num: u8) -> bool {
        (num & 8) != 0
    }
    pub fn set_occupied(pos: BlockPosition, game: &mut Game, flag: bool) {
        let l = game.block(pos, pos.world);
        if let Some(mut l) = l {
            if flag {
                l.b_metadata |= 4;
            } else {
                l.b_metadata &= -5i8 as u8;
            }
            game.set_block(pos, l, pos.world);
        }
    }
}
const MAP: [[i32; 2]; 4] = [
    [0, 1],
    [-1, 0],
    [0, -1],
    [1, 0]
];