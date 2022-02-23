use std::sync::Arc;

use anyhow::bail;
use hecs::Entity;
use crate::ecs::Ecs;
use crate::ecs::entities::player::{Player, BlockInventoryOpen};
use crate::game::{Position, Game};
use crate::item::inventory::Inventory;
use crate::item::inventory::reference::BackingWindow as BackingWindow;
use crate::item::inventory_slot::InventorySlot;
use crate::item::item::{Item, ItemRegistry};
use crate::item::stack::{ItemStack, ItemStackType};
use crate::network::ids::NetworkID;
use crate::protocol::packets::client::{WindowClick, CloseWindow};
use crate::{ecs::{entities::player::Gamemode, EntityRef, systems::SysResult}, protocol::packets::client::CreativeInventoryAction, server::Server, item::window::Window};

use super::interaction::drop_item;

pub fn handle_creative_inventory_action(
    game: &mut Game,
    player: Entity,
    packet: CreativeInventoryAction,
    server: &mut Server,
) -> SysResult {
    let player = game.ecs.entity(player)?;
    let mut clicked_item;
    if packet.item_id == -1 {
        clicked_item = InventorySlot::Empty;
    } else if packet.item_id > 255 {
        match ItemRegistry::global().get_item(packet.item_id) {
            Some(i) => {
                clicked_item = InventorySlot::Filled(ItemStack::new(i.id(), packet.quantity as i8, packet.meta));
            }
            None => {
                log::info!("Unknown item {}", packet.item_id);
                clicked_item = InventorySlot::Empty;
            }
        }
    } else {
        match ItemRegistry::global().get_block(packet.item_id as u8) {
            Some(i) => {
                clicked_item = InventorySlot::Filled(ItemStack::new(i.id() as i16, packet.quantity as i8, packet.meta));
            }
            None => {
                log::info!("Unknown block {}:{}", packet.item_id, packet.meta);
                clicked_item = InventorySlot::Empty;
            }
        }
    }
    if *player.get::<Gamemode>()? != Gamemode::Creative {
        bail!("cannot use Creative Inventory Action outside of creative mode");
    }

    if packet.slot_id != -1 {
        let window = player.get::<Window>()?;
        if !matches!(window.inner(), BackingWindow::Player { .. }) {
            bail!("cannot use Creative Inventory Action in external inventories");
        }

        window
            .inner()
            .set_item(packet.slot_id as usize, clicked_item)?;
        // Sends the client updates about window changes.
        // Is required to make delete inventory button reflect in-game.
        let client_id = *player.get::<NetworkID>()?;
        let client = server.clients.get(&client_id).unwrap();
        client.send_window_items(&window);
        let world = player.get::<Position>()?.world;
        server.broadcast_equipment_change(&player)?;
    } else if let InventorySlot::Filled(item) = clicked_item {
        let pos = *player.get::<Position>()?;
        drop(player);
        drop_item(game, item, pos)?;
    }
    Ok(())
}
pub fn handle_close_window(server: &mut Server, game: &mut Game, player_e: Entity, _packet: CloseWindow) -> SysResult {
    let player = game.ecs.entity(player_e)?;
    let pos = *player.get::<Position>()?;
    let mut window = player.get_mut::<Window>()?;
    let dropped_item = window.cursor_item_mut().take_all();
    drop(window);
    if let InventorySlot::Filled(item) = dropped_item {
        drop_item(game, item, pos)?;
    }
    let player = game.ecs.entity(player_e)?;
    let new_window = Window::new(BackingWindow::Player { player: player.get::<Inventory>()?.new_handle() });
    *player.get_mut::<Window>()? = new_window;
    drop(player);
    if let Ok(v) = game.ecs.remove::<BlockInventoryOpen>(player_e) {
        if let Some(state) = game.block(v.0, v.0.world) {
            let reg = state.registry_type()?;
            reg.on_inventory_closed(game, server, state, v.0, player_e)?;
        }
    }
    Ok(())
}
pub fn handle_click_window(
    server: &mut Server,
    game: &mut Game,
    player_entity: Entity,
    packet: WindowClick,
) -> SysResult {
    if packet.slot == -999 {
        let player = game.ecs.entity(player_entity)?;
    
        let client = server.clients.get(&*player.get::<NetworkID>()?).unwrap();
        client.confirm_window_action(
            packet.window_id,
            packet.action_number as i16,
            true,
        );
    
        let mut window = player.get_mut::<Window>()?;
        let dropped_item = window.cursor_item().clone();
        window.cursor_item_mut().take_all();
        client.set_cursor_slot(window.cursor_item());
    
        client.send_window_items(&*window);
        drop(window);
        if let InventorySlot::Filled(item) = dropped_item {
            let pos = *player.get::<Position>()?;
            drop_item(game, item, pos)?;
        }
        return Ok(());
    }
    let player = game.ecs.entity(player_entity)?;
    let result = _handle_click_window(&player, &packet);

    let client = server.clients.get(&*player.get::<NetworkID>()?).unwrap();
    client.confirm_window_action(
        packet.window_id,
        packet.action_number as i16,
        result.is_ok(),
    );

    let window = player.get::<Window>()?;
    if packet.slot >= 0 {
        client.set_slot(packet.window_id, packet.slot, &*window.item(packet.slot as usize)?);
    }
    client.set_cursor_slot(window.cursor_item());
    client.send_window_items(&*window);
    drop(window);
    let world = player.get::<Position>()?.world;
    server.broadcast_equipment_change(&player)?;
    drop(player);
    sync_inventories(&mut game.ecs, server, player_entity)?;
    result
}

fn sync_inventories(ecs: &mut Ecs, server: &mut Server, player: Entity) -> SysResult {
    let window = ecs.get::<Window>(player)?;
    let inventories = all_inventories(&window)?;
    for (entity, (_, window, id)) in ecs.query::<(&Player, &Window, &NetworkID)>().iter() {
        if entity != player {
            let player_inventories = all_inventories(window)?;
            let mut matched = false;
            for val in player_inventories {
                for val2 in inventories.iter() {
                    if val.ptr_eq(val2) {
                        matched = true;
                        break;
                    }
                }
                if matched {
                    break;
                }
            }
            if matched {
                server.clients.get(id).unwrap().send_window_items(window);
            }
        }
    }
    Ok(())
}

fn all_inventories(window: &Window) -> SysResult<Vec<Inventory>> {
    let mut inventories = Vec::new();
    match window.inner() {
        BackingWindow::Player { player } => inventories.push(player.new_handle()),
        BackingWindow::Generic9x1 { block, player } => {
            inventories.push(block.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Generic9x2 { block, player } => {
            inventories.push(block.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Generic9x3 { block, player } => {
            inventories.push(block.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Generic9x4 { block, player } => {
            inventories.push(block.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Generic9x5 { block, player } => {
            inventories.push(block.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Generic9x6 { left_chest, right_chest, player } => {
            inventories.push(left_chest.new_handle());
            inventories.push(right_chest.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Generic3x3 { block, player } => {
            inventories.push(block.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Crafting { crafting_table, player } => {
            inventories.push(crafting_table.new_handle());
            inventories.push(player.new_handle());
        },
        BackingWindow::Furnace { furnace, player } => {
            inventories.push(furnace.new_handle());
            inventories.push(player.new_handle());
        },
    }
    Ok(inventories)
}


fn _handle_click_window(player: &EntityRef, packet: &WindowClick) -> SysResult {
    let mut window = player.get_mut::<Window>()?;
    let mode = match packet.shift {
        true => 1,
        false => 0,
    };
    match mode {
        0 => match packet.right_click {
            false => window.left_click(packet.slot as usize)?,
            true => window.right_click(packet.slot as usize)?,
            _ => bail!("unrecgonized click"),
        },
        1 => window.shift_click(packet.slot as usize)?,
        _ => unreachable!()
    };

    Ok(())
}