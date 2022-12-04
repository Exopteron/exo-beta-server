use std::panic::AssertUnwindSafe;

use hecs::Entity;
use parking_lot::MutexGuard;
use rand::Rng;

use crate::{
    aabb::AABB,
    ecs::{entities::player::Chatbox, systems::SysResult, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        item::block::{ActionResult, AtomicRegistryBlock, BurnRate, NonBoxedRegBlock},
        stack::{ItemStack, ItemStackType}, window::Window, inventory_slot::InventorySlot,
    },
    network::ids::NetworkID,
    protocol::packets::{Face, SoundEffectKind},
    world::chunks::BlockState, status_effects::{StatusEffectsManager, fire::FireEffect},
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

use super::portal::PortalBlock;

pub struct FlintAndSteelItem;
impl Item for FlintAndSteelItem {
    fn id(&self) -> ItemIdentifier {
        259
    }

    fn stack_size(&self) -> i8 {
        1
    }

    fn durability(&self) -> Option<i16> {
        Some(64)
    }

    fn on_use(&self, game: &mut Game, server: &mut crate::server::Server, mut item: MutexGuard<InventorySlot>, slot: usize, user: Entity, target: Option<crate::item::item::BlockUseTarget>) -> SysResult {
        if let Some(target) = target {
            let pos = target.face.offset(target.position);
            if game.block_id_at(pos) == 0 {
                let id = *game.ecs.get::<NetworkID>(user)?;
                let window = game.ecs.get_mut::<Window>(user)?;
                let mut do_break = false;
                if let InventorySlot::Filled(item) = &mut *item {
                    if item.damage(1) {
                        do_break = true;
                    }
                }
                if do_break {
                    *item = InventorySlot::Empty;
                }
                drop(item);
                server.clients.get(&id).unwrap().send_window_items(&window);
                drop(window);
                game.set_block(pos, BlockState::from_id(51), target.world); 
            }
        }
        Ok(())
    }
}

pub struct FireBlock;
impl Block for FireBlock {
    fn on_collide(&self, game: &mut Game, position: BlockPosition, state: BlockState, player: Entity) -> SysResult {
        game.ecs.get_mut::<StatusEffectsManager>(player)?.add_effect(FireEffect::new(90));
        Ok(())
    }
    fn id(&self) -> BlockIdentifier {
        51
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn can_place_over(&self) -> bool {
        true
    }
    fn added(&self, world: i32, game: &mut Game, server: &mut crate::server::Server, position: BlockPosition, state: BlockState) {
        if game.block_id_at(position.offset(0, -1, 0)) == 49 && PortalBlock::try_create_portal(world, game, position) {
            return;
        }
        let s = game.scheduler.clone();
        let mut scheduler = s.borrow_mut();
        scheduler.schedule_task(game.ticks + 40, move |game| {
            if let Some(block) = game.block(position, world) {
                if let Ok(b) = block.registry_type() {
                    b.tick(world, game, block, position);
                }
            }
            None
        });
    }
    fn tick(&self, world: i32, game: &mut Game, state: BlockState, position: BlockPosition) {
        game.break_block(position, position.world);
        return;
        std::panic::catch_unwind(AssertUnwindSafe(|| {
            let flag = false;
            if !self.can_place_on(world, game, position, Face::Invalid) {
                game.break_block(position, world);
            }
            if state.b_metadata < 15 {
                let mut state = state;
                state.b_metadata += rand::thread_rng().gen_range(0..3) / 2;
                game.set_block(position, state, world);
            }
            let s = game.scheduler.clone();
            let mut scheduler = s.borrow_mut();
            scheduler.schedule_task(game.ticks + 40, move |game| {
                if let Some(block) = game.block(position, world) {
                    if let Ok(b) = block.registry_type() {
                        b.tick(world, game, block, position);
                    }
                }
                None
            });
            if !flag
                && !Self::burnable_surround(world, game, position)
                && (game.is_solid_block(position.offset(0, -1, 0)) || state.b_metadata > 3)
            {
                game.break_block(position, world);
                return;
            }
            if !flag
                && !Self::can_catch_fire(world, game, position.offset(0, -1, 0))
                && state.b_metadata == 15
                && rand::thread_rng().gen_range(0..4) == 0
            {
                game.break_block(position, world);
                return;
            }
            Self::try_catch_fire(world, game, position.offset(1, 0, 0), 300, state.b_metadata as i32);
            Self::try_catch_fire(world, game, position.offset(-1, 0, 0), 300, state.b_metadata as i32);
            Self::try_catch_fire(world, game, position.offset(0, -1, 0), 250, state.b_metadata as i32);
            Self::try_catch_fire(world, game, position.offset(0, 1, 0), 250, state.b_metadata as i32);
            Self::try_catch_fire(world, game, position.offset(1, 0, -1), 300, state.b_metadata as i32);
            Self::try_catch_fire(world, game, position.offset(1, 0, 1), 300, state.b_metadata as i32);
            for x1 in position.x - 1..position.x + 2 {
                for y1 in position.y - 1..position.y + 2 {
                    for z1 in position.z - 1..position.z + 5 {
                        if x1 == position.x && y1 == position.y && z1 == position.z {
                            continue;
                        }
                        let mut l1 = 100;
                        if z1 > position.y + 1 {
                            l1 += (y1 - (position.y + 1)) * 100;
                        }
                        let i2 = Self::chance_of_neighbors_ef(world, game, BlockPosition::new(x1, y1, z1, world));
                        if i2 <= 0 {
                            continue;
                        }
                        let j2 = (i2 + 40) / (state.b_metadata as i32 + 30);
                        if j2 <= 0 || rand::thread_rng().gen_range(0..l1) > j2 {
                            continue;
                        }
                        let mut k2 = state.b_metadata + rand::thread_rng().gen_range(0..5) / 4;
                        if k2 > 15 {
                            k2 = 15;
                        }
                        game.set_block(BlockPosition::new(x1, y1, z1, world), BlockState::new(51, k2), world);
                    }
                }
            }
        }));
    }

    fn can_place_on(
        &self,
        world: i32,
        game: &mut Game,
        position: BlockPosition,
        face: Face,
    ) -> bool {
        let pos = face.offset(position);
        game.is_solid_block(pos) || Self::burnable_surround(world, game, pos)
    }

    fn neighbor_update(&self, world: i32, game: &mut Game, position: BlockPosition, state: BlockState, offset: Face, neighbor_state: BlockState) -> SysResult {
        if !game.is_solid_block(position.offset(0, -1, 0)) && !Self::burnable_surround(world, game, position) {
            game.break_block(position, world);
        }
        Ok(())
    }
}

impl FireBlock {
    pub fn chance_of_neighbors_ef(world: i32, game: &mut Game, pos: BlockPosition) -> i32 {
        let mut l = 0;
        if !game.block_id_at(pos) == 0 {
            return 0;
        } else {
            for face in Face::all_faces() {
                let pos = face.offset(pos);
                l = Self::chance_to_encourage_fire(world, game, pos, l);
            }
            return l;
        }
    }
    pub fn try_catch_fire(world: i32, game: &mut Game, pos: BlockPosition, l: i32, i1: i32) {
        let j1 = Self::burnable(world, game, pos).1;
        if rand::thread_rng().gen_range(0..l) < j1 && (rand::thread_rng().gen_range(0..i1 + 10) < 5)
        {
            let mut k1 = i1 + rand::thread_rng().gen_range(0..5) / 4;
            if k1 > 15 {
                k1 = 15;
            }
            game.set_block(pos, BlockState::new(51, k1 as u8), world);
        } else {
            game.break_block(pos, world);
        }
    }
    pub fn chance_to_encourage_fire(world: i32, game: &mut Game, pos: BlockPosition, l: i32) -> i32 {
        let i1 = Self::burnable(world, game, pos).0;
        if i1 > l {
            return i1;
        } else {
            return l;
        }
    }
    pub fn burnable(world: i32, game: &mut Game, pos: BlockPosition) -> BurnRate {
        if let Some(block) = game.block(pos, world) {
            if let Ok(b) = block.registry_type() {
                if let Some(b) = b.burn_rate() {
                    return b;
                }
            }
        }
        BurnRate(0, 0)
    }
    pub fn can_catch_fire(world: i32, game: &mut Game, pos: BlockPosition) -> bool {
        if let Some(block) = game.block(pos, world) {
            if let Ok(b) = block.registry_type() {
                if let Some(b) = b.burn_rate() {
                    return b.0 > 0;
                }
            }
        }
        false
    }
    pub fn burnable_surround(world: i32, game: &mut Game, pos: BlockPosition) -> bool {
        for face in Face::all_faces() {
            let pos = face.offset(pos);
            if Self::can_catch_fire(world, game, pos) {
                return true;
            }
        }
        false
    }
}
