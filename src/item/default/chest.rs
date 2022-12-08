use std::f32::consts::PI;

use hecs::Entity;
use nbt::CompoundTag;
use rand::Rng;

use crate::{
    block_entity::{BlockEntity, BlockEntityLoader, BlockEntitySaver},
    ecs::{
        entities::{
            item::ItemEntityBuilder,
            player::{BlockInventoryOpen, Chatbox},
        },
        systems::SysResult,
        EntityRef,
    },
    events::block_interact::BlockPlacementEvent,
    game::{BlockPosition, Game, Position},
    item::{
        inventory::{reference::{BackingWindow, Area}, Inventory},
        inventory_slot::InventorySlot,
        item::block::{ActionResult, AtomicRegistryBlock, NonBoxedRegBlock},
        stack::{ItemStack, ItemStackType},
        window::Window,
    },
    network::ids::NetworkID,
    physics::Physics,
    player_dat::NBTItem,
    protocol::packets::{Face, SoundEffectKind, WindowKind},
    server::Server,
    world::chunks::BlockState,
};

use crate::item::item::{block::Block, BlockIdentifier, Item, ItemIdentifier, ItemRegistry};
pub struct ChestData(pub Inventory, pub usize);
impl ChestData {
    pub fn new() -> Self {
        Self(Inventory::chest(), 0)
    }
}
pub struct ChestBlock;
impl Block for ChestBlock {
    fn id(&self) -> BlockIdentifier {
        54
    }
    fn opacity(&self) -> u8 {
        0
    }

    fn item_stack_size(&self) -> i8 {
        64
    }
    fn place(
        &self,
        game: &mut Game,
        entity: Entity,
        mut item: ItemStack,
        mut position: BlockPosition,
        face: Face,
        world: i32,
    ) -> Option<BlockPlacementEvent> {
        position = face.offset(position);
        item.set_damage(chest(&mut position, &game.ecs.get::<Position>(entity).unwrap()).into());
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
        let chest = game.block_entity_at(position).ok_or(anyhow::anyhow!("a"))?;
        game.ecs
            .insert(player_entity, BlockInventoryOpen(position))?;
        let mut chest_items = game.ecs.get_mut::<ChestData>(chest)?;
        chest_items.1 += 1;
        let player = game.ecs.entity(player_entity)?;
        let id = *player.get::<NetworkID>()?;
        let inventory = player.get::<Inventory>()?.new_handle();
        let mut window = player.get_mut::<Window>()?;
        let new_window = Window::new(BackingWindow::Generic9x3 {
            block: chest_items.0.new_handle(),
            player: inventory,
        });
        *window = new_window;
        let cl = server.clients.get(&id).unwrap();
        cl.open_window(1, WindowKind::Chest, "Chest".to_string(), 27);
        cl.send_window_items(&window);
        Ok(ActionResult::SUCCESS)
    }
    fn block_entity(
        &self,
        entity_builder: &mut hecs::EntityBuilder,
        state: BlockState,
        position: BlockPosition,
    ) -> bool {
        entity_builder.add(ChestData::new());
        entity_builder.add(BlockEntitySaver::new(
            |entity| {
                let mut tag = CompoundTag::new();

                let mut items = vec![];

                let data = entity.get::<ChestData>()?;
                for (idx, item) in data.0.to_vec().iter().enumerate() {
                    if let InventorySlot::Filled(item) = item {
                        items.push(
                            NBTItem {
                                id: item.id(),
                                count: item.count(),
                                damage: item.damage_taken(),
                                slot: idx as i8,
                            }
                            .to_old(),
                        )
                    }
                }
                tag.insert_compound_tag_vec("Items", items);

                Ok(tag)
            },
            "Chest".to_string(),
        ));
        entity_builder.add(BlockEntityLoader::new(|client, entity| {
            log::info!("LOader");
            let data = entity.get::<ChestData>()?;
            let pos = entity.get::<BlockEntity>()?.0;
            if data.1 > 0 {
                client.send_block_action(pos, 1, 1);
            }
            // //log::info!("Sign loader sending {:?} to {}", *data, client.username());
            // client.update_sign(pos, data.deref().clone());
            Ok(())
        }));
        true
    }
    fn block_entity_loader(&self, loaders: &mut crate::block_entity::BlockEntityNBTLoaders) {
        loaders.insert(
            "Chest",
            Box::new(|tag, blockpos, builder| {
                let items: Vec<NBTItem> = tag
                    .get_compound_tag_vec("Items")
                    .map_err(|_| anyhow::anyhow!("No tag {} {}", line!(), file!()))?
                    .into_iter()
                    .flat_map(|v| NBTItem::from_old(v))
                    .collect();

                let mut chest = ChestData::new();
                for item in items {
                    let _ = chest.0.item(Area::Storage, item.slot as usize).map(|mut v| *v = InventorySlot::Filled(item.into()));
                }

                builder.add(chest);
                Ok(())
            }),
        );
    }
    fn opaque(&self) -> bool {
        false
    }
    fn on_inventory_closed(
        &self,
        game: &mut Game,
        server: &mut Server,
        state: BlockState,
        position: BlockPosition,
        player: Entity,
    ) -> SysResult {
        let block_entity = game.block_entity_at(position).unwrap();
        let mut chest = game.ecs.get_mut::<ChestData>(block_entity)?;
        chest.1 -= 1;
        if chest.1 == 0 {
            server.broadcast_nearby_with(position.into(), |cl| {
                cl.send_block_action(position, 1, 0);
            });
        }
        Ok(())
    }
    fn on_break(
        &self,
        game: &mut Game,
        server: &mut Server,
        breaker: Entity,
        mut position: BlockPosition,
        face: Face,
        world: i32,
    ) {
        self.on_break_wrap(game, server, breaker, position, face, world);
    }
}
pub fn drop_item(game: &mut Game, item: ItemStack, position: Position) -> SysResult {
    let mut itempos = position;
    itempos.on_ground = false;
    itempos.y += 1.;
    itempos.y += 0.22;
    let mut item = ItemEntityBuilder::build(game, itempos, item, 5);
    let mut physics = item
        .get_mut::<Physics>()
        .ok_or(anyhow::anyhow!("No physics?"))?;
    let mut f = 0.3;
    let mut f1 = 0.;
    let mut velocity = physics.get_velocity_mut();
    velocity.x =
        (-((position.yaw / 180.) * PI).sin() * ((position.pitch / 180.) * PI).cos() * f) as f64;
    velocity.z =
        (((((position.yaw / 180.) * PI).cos()) * (position.pitch / 180. * PI).cos()) * f) as f64;
    velocity.y = ((-((position.pitch / 180.) * PI).sin()) * f + 0.1) as f64;
    f = 0.02;
    f1 = rand::thread_rng().gen::<f32>() * std::f32::consts::PI * 2.0;
    f *= rand::thread_rng().gen::<f32>();
    velocity.x += (f1 as f64).cos() * f as f64;
    velocity.y +=
        ((rand::thread_rng().gen::<f32>() - rand::thread_rng().gen::<f32>()) * 0.1) as f64;
    velocity.z += (f1 as f64).sin() * f as f64;
    game.spawn_entity(item);
    Ok(())
}
impl ChestBlock {
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
        let be = game.block_entity_at(position).ok_or(anyhow::anyhow!("G"))?;
        let chest = game.ecs.get::<ChestData>(be)?.0.new_handle();
        for item in chest.to_vec() {
            if let InventorySlot::Filled(stack) = item {
                drop_item(game, stack, position.into())?;
            }
        }
        Ok(())
    }
}
pub fn chest(pos: &mut BlockPosition, placer_pos: &Position) -> u8 {
    let l = (((((placer_pos.yaw * 4.0) / 360.) as f64) + 0.5).floor() as i32) & 3;
    match l {
        0 => 2,
        1 => 5,
        2 => 3,
        _ => 4,
    }
}
