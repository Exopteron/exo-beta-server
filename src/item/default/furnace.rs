use hecs::Entity;
use nbt::CompoundTag;

use crate::{
    ecs::{entities::player::{Chatbox, BlockInventoryOpen}, systems::{SysResult, entities::BlockEntityTicker}, EntityRef},
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{stack::{ItemStackType, ItemStack}, item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock}, inventory::{Inventory, reference::{BackingWindow, Area}}, window::Window, inventory_slot::InventorySlot},
    protocol::packets::{Face, SoundEffectKind, WindowKind, ProgressBarKind},
    world::chunks::BlockState, network::ids::NetworkID, block_entity::{BlockEntitySaver, BlockEntityLoader}, server::Server,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};

use super::chest::drop_item;
#[derive(Clone)]
pub struct FurnaceData(pub Inventory, pub i16, pub i16);
impl FurnaceData {
    pub fn new() -> Self {
        Self(Inventory::furnace(), 0, 0)
    }
}
pub struct FurnaceBlock(pub BlockIdentifier);
impl Block for FurnaceBlock {
    fn id(&self) -> BlockIdentifier {
        self.0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(&self, game: &mut Game, entity: Entity, mut item: ItemStack, mut position: BlockPosition, face: Face, world: i32) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        item.set_damage(furnace_orient(&mut position, &game.ecs.get::<Position>(entity).unwrap()).into());
        Some(BlockPlacementEvent {
            held_item: item,
            location: position,
            face,
            world,
        })
    }
    fn interacted_with(
        &self,
        world: i32,
        game: &mut Game,
        server: &mut crate::server::Server,
        position: BlockPosition,
        mut state: BlockState,
        player_entity: Entity,
    ) -> anyhow::Result<ActionResult> {
        server.broadcast_nearby_with(position.into(), |cl| {
            cl.send_block_action(position, 1, 1);
        });
        let chest = game.block_entity_at(position).unwrap();
        game.ecs
            .insert(player_entity, BlockInventoryOpen(position))?;
        let mut chest_items = game.ecs.get_mut::<FurnaceData>(chest)?;
        let player = game.ecs.entity(player_entity)?;
        let id = *player.get::<NetworkID>()?;
        let inventory = player.get::<Inventory>()?.new_handle();
        let mut window = player.get_mut::<Window>()?;
        let new_window = Window::new(BackingWindow::Furnace {
            furnace: chest_items.0.new_handle(),
            player: inventory,
        });
        *window = new_window;
        let cl = server.clients.get(&id).unwrap();
        cl.open_window(1, WindowKind::Furnace, "Furnace".to_string(), 3);
        cl.send_window_items(&window);
        cl.update_progress_bar(1, ProgressBarKind::FireIcon, chest_items.2);
        cl.update_progress_bar(1, ProgressBarKind::ProgressArrow, chest_items.1);
        Ok(ActionResult::SUCCESS)
    }
    fn block_entity(
        &self,
        entity_builder: &mut hecs::EntityBuilder,
        state: BlockState,
        position: BlockPosition,
    ) -> bool {
        entity_builder.add(FurnaceData::new());
        entity_builder.add(BlockEntitySaver::new(
            |entity| {
                let mut tag = CompoundTag::new();
                Ok(tag)
            },
            "Furnace".to_string(),
        ));
        entity_builder.add(BlockEntityTicker(|game, server, entity, pos, state| {
            let registry = ItemRegistry::global();
            let mut data =  game.ecs.get_mut::<FurnaceData>(entity)?;
            if data.2 > 0 {
                data.2 -= 1;
            }
            let inv = data.0.new_handle();
            let mut fuel_slot = inv.item(Area::FurnaceFuel, 0).unwrap();
            let is_to_smelt = match &*inv.item(Area::FurnaceIngredient, 0).unwrap() {
                InventorySlot::Filled(i) => {
                    let recipe = registry.furnace.get_item(i.id());
                    if let Some(r) = recipe {
                        let x = inv.item(Area::FurnaceOutput, 0).unwrap();
                        if InventorySlot::Filled(r.clone()).is_mergable(&x) {
                            (true, Some(r))
                        } else {
                            (false, None)
                        }
                    } else {
                        (recipe.is_some(), recipe)
                    }
                },
                InventorySlot::Empty => (false, None),
            };
            let mut filled = None;
            if let InventorySlot::Filled(item) = &mut *fuel_slot {
                filled = Some(item.clone());
            }
            if let Some(item) = filled {
                let burnability = registry.get_burnability(item.id());
                if burnability > 0 && data.2 == 0 && is_to_smelt.0 {
                    data.2 += burnability;
                    fuel_slot.try_take(1);
                }
            }
            drop(fuel_slot);
            if is_to_smelt.0 && data.2 > 0 {
                data.1 += 1;
                if data.1 >= 200 {
                    let output = is_to_smelt.1.unwrap();
                    inv.item(Area::FurnaceIngredient, 0).unwrap().try_take(1);
                    inv.item(Area::FurnaceOutput, 0).unwrap().merge(&mut InventorySlot::Filled(output)); // TODO check b4 smelt
                    data.1 = 0;
                }
            } else if data.1 > 0 {
                data.1 -= 2;
                data.1 = data.1.max(0);
            }
            let f_data = data.clone();
            for (_, (id, data, window)) in game.ecs.query::<(&NetworkID, &BlockInventoryOpen, &Window)>().iter() {
                if data.0 == pos {
                    let client = server.clients.get(id).unwrap();
                    client.send_window_items(window);
                    client.update_progress_bar(1, ProgressBarKind::FireIcon, f_data.2.min(250));
                    client.update_progress_bar(1, ProgressBarKind::ProgressArrow, f_data.1);
                }
            }
            Ok(())
        }));
        entity_builder.add(BlockEntityLoader::new(|client, entity| {
            // let data = entity.get::<FurnaceData>()?;
            // let pos = entity.get::<BlockEntity>()?.0;
            // if data.1 > 0 {
            //     client.send_block_action(pos, 1, 1);
            // }
            // //log::info!("Sign loader sending {:?} to {}", *data, client.username());
            // client.update_sign(pos, data.deref().clone());
            Ok(())
        }));
        true
    }
    fn block_entity_loader(&self, loaders: &mut crate::block_entity::BlockEntityNBTLoaders) {
        loaders.insert(
            "Furnace",
            Box::new(|tag, blockpos, builder| {
                builder.add(FurnaceData::new());
                Ok(())
            }),
        );
    }
    fn on_break(&self, game: &mut Game, server: &mut Server, breaker: Entity, mut position: BlockPosition, face: Face, world: i32) {
        self.on_break_wrap(game, server, breaker, position, face, world);
    }
}
impl FurnaceBlock {
    fn on_break_wrap(
        &self,
        game: &mut Game,
        server: &mut Server,
        breaker: Entity,
        mut position: BlockPosition,
        face: Face,
        world: i32,
    ) -> SysResult {
        let mut entities = Vec::new();
        for (player, (inv, window)) in game
            .ecs
            .query::<(&BlockInventoryOpen, &mut Window)>()
            .iter()
        {
            if inv.0 == position {
                entities.push(player);
            }
        }
        for player_e in entities {
            let player = game.ecs.entity(player_e)?;
            let pos = *player.get::<Position>()?;
            let id = *player.get::<NetworkID>()?;
            let mut window = player.get_mut::<Window>()?;
            let dropped_item = window.cursor_item_mut().take_all();
            drop(window);
            if let InventorySlot::Filled(item) = dropped_item {
                drop_item(game, item, pos)?;
            }
            let player = game.ecs.entity(player_e)?;
            let new_window = Window::new(BackingWindow::Player {
                player: player.get::<Inventory>()?.new_handle(),
            });
            *player.get_mut::<Window>()? = new_window;
            let client = server.clients.get(&id).unwrap();
            client.close_window(1);
            let w = player.get::<Window>()?;
            client.send_window_items(&w);
            drop(player);
        }
        let be = game.block_entity_at(position).ok_or(anyhow::anyhow!("B"))?;
        let chest = game.ecs.get::<FurnaceData>(be)?.0.new_handle();
        for item in chest.to_vec() {
            if let InventorySlot::Filled(stack) = item {
                drop_item(game, stack, position.into())?;
            }
        }
        Ok(())
    }
}
pub fn furnace_orient(pos: &mut BlockPosition, placer_pos: &Position) -> u8 {
    let l = (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 3;
    match l {
        0 => {
            2
        }
        1 => {
            5
        }
        2 => {
            3
        }
        _ => {
            4
        }
    }
}