use anyhow::bail;
use crate::item::inventory::reference::BackingWindow as BackingWindow;
use crate::item::inventory_slot::InventorySlot;
use crate::item::item::{Item, ItemRegistry};
use crate::item::stack::{ItemStack, ItemStackType};
use crate::network::ids::NetworkID;
use crate::protocol::packets::client::WindowClick;
use crate::{ecs::{entities::player::Gamemode, EntityRef, systems::SysResult}, protocol::packets::client::CreativeInventoryAction, server::Server, item::window::Window};

pub fn handle_creative_inventory_action(
    player: EntityRef,
    packet: CreativeInventoryAction,
    server: &mut Server,
) -> SysResult {
    let mut clicked_item;
    if packet.item_id > 255 {
        match ItemRegistry::global().get_item(packet.item_id) {
            Some(i) => {
                clicked_item = InventorySlot::Filled(ItemStack::new(ItemStackType::Item(i), packet.quantity as i8, packet.meta));
            }
            None => {
                log::info!("Unknown item {}", packet.item_id);
                clicked_item = InventorySlot::Empty;
            }
        }
    } else {
        match ItemRegistry::global().get_block((packet.item_id as u8, packet.meta as u8)) {
            Some(i) => {
                clicked_item = InventorySlot::Filled(ItemStack::new(ItemStackType::Block(i), packet.quantity as i8, packet.meta));
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
    }
    server.broadcast_equipment_change(&player)?;
    Ok(())
}

pub fn handle_click_window(
    server: &mut Server,
    player: EntityRef,
    packet: WindowClick,
) -> SysResult {
    if packet.slot == -999 {
        // drop
        return Ok(());
    }
    let result = _handle_click_window(&player, &packet);

    let client = server.clients.get(&*player.get::<NetworkID>()?).unwrap();
    client.confirm_window_action(
        packet.window_id,
        packet.action_number as i16,
        result.is_ok(),
    );

    let window = player.get::<Window>()?;

    if packet.slot >= 0 {
        client.set_slot(0, packet.slot, &*window.item(packet.slot as usize)?);
    }
    client.set_cursor_slot(window.cursor_item());

    client.send_window_items(&*window);
    drop(window);
    server.broadcast_equipment_change(&player)?;
    result
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