use std::mem;

use anyhow::{anyhow, bail};
use crate::ecs::systems::SysResult;

use super::{inventory::{reference::{BackingWindow as BackingWindow, Area}, WindowError}, inventory_slot::InventorySlot};
use parking_lot::MutexGuard;
use super::inventory_slot::InventorySlot::Empty;

/// A player's window. Wraps one or more inventories and handles
/// conversion between protocol and slot indices.
///
/// Also provides high-level methods to interact with the inventory,
/// like [`Window::right_click`], [`Window::shift_click`], etc.
#[derive(Debug)]
pub struct Window {
    /// The backing window (contains the `Inventory`s)
    inner: BackingWindow,
    /// The item currently held by the player's cursor.
    cursor_item: InventorySlot,
    /// Current painting state (mouse drag)
    paint_state: Option<PaintState>,
}

impl Window {
    /// Creates a window from the backing window representation.
    pub fn new(inner: BackingWindow) -> Self {
        Self {
            inner,
            cursor_item: Empty,
            paint_state: None,
        }
    }

    /// Left-click a slot in the window.
    pub fn left_click(&mut self, slot: usize) -> SysResult {
        let slot = &mut *self.inner.item(slot)?;
        let cursor_slot = &mut self.cursor_item;

        // Cases:
        // * Either the cursor slot or the clicked slot is empty; swap the two.
        // * Both slots are present but are of different types; swap the two.
        // * Both slots are present and have the same type; merge the two.

        if slot.is_filled() && cursor_slot.is_filled() && cursor_slot.is_mergable(slot) {
            slot.merge(cursor_slot);
        } else {
            mem::swap(cursor_slot, slot);
        }

        Ok(())
    }

    /// Right-clicks a slot in the window.
    pub fn right_click(&mut self, slot_index: usize) -> SysResult {
        let slot = &mut *self.inner.item(slot_index)?;
        let cursor_slot = &mut self.cursor_item;

        // Cases:
        // * Cursor slot is present and clicked slot has the same item type; drop one item in the clicked slot.
        // * Clicked slot is present but cursor slot is not; move half the items into the cursor slot.
        // * Both slots are present but differ in type; swap the two.

        match (slot.is_filled(), cursor_slot.is_filled()) {
            (true, true) => {
                if slot.is_mergable(cursor_slot) {
                    cursor_slot.transfer_to(1, slot);
                } else {
                    mem::swap(slot, cursor_slot);
                }
            }
            (true, false) => {
                *cursor_slot = slot.take_half();
            }
            (false, true) => {
                *slot = cursor_slot.try_take(1);
            }
            (false, false) => {}
        }

        Ok(())
    }

    /// Shift-clicks the given slot. (Either right or left click.)
    pub fn shift_click(&mut self, slot: usize) -> SysResult {
        // If we are shift clicking on a empty slot, then nothing happens.
        {
            let slot_inventory = &mut *self.inner.item(slot)?;
            if slot_inventory.is_empty() {
                // Shift clicking on a empty inventory slot does nothing.
                return Ok(());
            }
        }

        match &self.inner {
            BackingWindow::Player { player: _ } => self.shift_click_in_player_window(slot),

            BackingWindow::Generic9x1 {
                block: _,
                player: _,
            }
            | BackingWindow::Generic9x2 {
                block: _,
                player: _,
            }
            | BackingWindow::Generic9x3 {
                block: _,
                player: _,
            }
            | BackingWindow::Generic9x4 {
                block: _,
                player: _,
            }
            | BackingWindow::Generic9x5 {
                block: _,
                player: _,
            }
            | BackingWindow::Generic3x3 {
                block: _,
                player: _,
            }
            | BackingWindow::Generic9x6 {
                left_chest: _,
                right_chest: _,
                player: _,
            } => self.shift_click_in_generic_window(slot),

            BackingWindow::Crafting {
                crafting_table: _,
                player: _,
            } => self.shift_click_in_crafting_window(slot),
            BackingWindow::Furnace {
                furnace: _,
                player: _,
            } => self.shift_click_in_furnace(slot),
        }
    }

    fn shift_click_in_player_window(&mut self, slot: usize) -> SysResult {
        let mut slot_item = &mut *self.inner.item(slot)?;

        let (inventory, slot_area, _) = self.inner.index_to_slot(slot).unwrap();
        let areas_to_try = [
            Area::Helmet,
            Area::Chestplate,
            Area::Leggings,
            Area::Boots,
            Area::CraftingInput,
            Area::Hotbar,
            Area::Storage,
        ];

        for &area in &areas_to_try {
            if area == slot_area || !will_accept(area, slot_item) {
                continue;
            }

            // Find slot with same type first
            let mut i = 0;
            while let Some(mut stack) = inventory.item(area, i) {
                if slot_item.is_mergable(&stack) && stack.is_filled() {
                    stack.merge(slot_item);
                }
                i += 1;
            }

            if slot_item.is_empty() {
                return Ok(());
            }
        }

        if slot_item.is_filled() {
            for &area in &areas_to_try {
                if area == slot_area || !will_accept(area, slot_item) {
                    continue;
                }

                // If we still haven't moved all the items, transfer to any empty space
                let mut i = 0;
                while let Some(mut stack) = inventory.item(area, i) {
                    if stack.is_empty() {
                        stack.merge(&mut slot_item);
                    }
                    i += 1;
                }

                if slot_item.is_empty() {
                    break;
                }
            }
        }

        Ok(())
    }

    fn shift_click_in_generic_window(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_crafting_window(&mut self, _slot: usize) -> SysResult {
        // TODO: If you shift click an item in the crafting table, then you craft
        // as many as possible. So the items are crafted and put in Area::CraftingOutput
        // We don't currently have a working crafting system, and once we have we probably
        // need to change the function signature to get access to the crafting system.
        todo!()
    }

    fn shift_click_in_furnace(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_blast_furnace(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_smoker(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_enchantment(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_brewing_window(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_beacon(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_anvil(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_hopper(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_shulker_box(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    fn shift_click_in_cartography_window(&mut self, _slot: usize) -> SysResult {
        todo!()
    }
    fn shift_click_in_grindstone(&mut self, _slot: usize) -> SysResult {
        todo!()
    }
    fn shift_click_in_lectern(&mut self, _slot: usize) -> SysResult {
        todo!()
    }
    fn shift_click_in_loom(&mut self, _slot: usize) -> SysResult {
        todo!()
    }
    fn shift_click_in_stonecutter(&mut self, _slot: usize) -> SysResult {
        todo!()
    }

    /// Starts a left mouse paint operation.
    pub fn begin_left_mouse_paint(&mut self) {
        self.paint_state = Some(PaintState::new(Mouse::Left));
    }

    /// Starts a right mouse paint operation.
    pub fn begin_right_mouse_paint(&mut self) {
        self.paint_state = Some(PaintState::new(Mouse::Right));
    }

    /// Adds a slot to the current paint operation.
    pub fn add_paint_slot(&mut self, slot: usize) -> SysResult {
        if let Some(state) = &mut self.paint_state {
            state.add_slot(slot)
        } else {
            Err(anyhow!("no paint operation was active"))
        }
    }

    /// Completes and executes the current paint operation.
    pub fn end_paint(&mut self) -> SysResult {
        if let Some(state) = self.paint_state.take() {
            state.finish(self)
        } else {
            Err(anyhow!("no paint operation was active"))
        }
    }

    /// Gets the item currently held in the cursor.
    pub fn cursor_item(&self) -> &InventorySlot {
        &self.cursor_item
    }

    pub fn item(&self, index: usize) -> Result<MutexGuard<InventorySlot>, WindowError> {
        self.inner.item(index)
    }

    /// Sets an [`InventorySlot`] at the index.
    /// # Error
    /// Returns an error if the index is [`WindowError::OutOfBounds`]
    pub fn set_item(&self, index: usize, item: InventorySlot) -> Result<(), WindowError> {
        self.inner.set_item(index, item)
    }

    pub fn inner(&self) -> &BackingWindow {
        &self.inner
    }
}

/// Determines whether the given area will accept the given item
/// for shift-click transfer.
fn will_accept(area: Area, stack: &InventorySlot) -> bool {
    match area {
        Area::Storage => true,
        Area::CraftingOutput => false,
        Area::CraftingInput => false,
        Area::Helmet => false,
        Area::Chestplate => false,
        Area::Leggings => false,
        Area::Boots => false,
        Area::Hotbar => true,
        Area::FurnaceIngredient => true,
        Area::FurnaceFuel => true,
        Area::FurnaceOutput => false,
    }
}

/// State for a paint operation (left mouse or right mouse drag).
#[derive(Debug)]
struct PaintState {
    mouse: Mouse,
    slots: Vec<usize>,
}

impl PaintState {
    pub fn new(mouse: Mouse) -> Self {
        Self {
            mouse,
            slots: Vec::new(),
        }
    }

    pub fn add_slot(&mut self, slot: usize) -> SysResult {
        self.slots.push(slot);
        if self.slots.len() > 1000 {
            bail!("too many paint slots! malicious client?");
        }
        Ok(())
    }

    pub fn finish(self, window: &mut Window) -> SysResult {
        match self.mouse {
            Mouse::Left => self.handle_left_drag(window),
            Mouse::Right => self.handle_right_drag(window),
        }
        Ok(())
    }

    /**
        Splits cursor items evenly into every selected slot.
        Remainder of even split ends up in `window.cursor_item`.
    */
    fn handle_left_drag(&self, window: &mut Window) {
        // If the cursor has no item then there are no items to share.
        if window.cursor_item().is_empty() {
            return;
        }

        // Number of slots that can contain cursors item kind.
        let slots = self
            .slots
            .iter()
            .filter(|s| {
                // unwrap is safe because index is valid.
                let slot = &*window.inner.item(**s).unwrap();
                slot.is_mergable(window.cursor_item())
            })
            .count() as i8;

        // If slots is 0 that means there are no slots to put items into.
        // So the cursor keeps all the items.
        if slots == 0 {
            return;
        };

        let items_for_cursor = window.cursor_item().count();
        // This can't be zero because items_cursor is the count of an ItemStack and ItemStack is NonZeroU32.
        let items_per_slot = (items_for_cursor / slots).max(1);
        self.move_items_into_slots(window, items_per_slot);
    }

    /// Tries to move items_per_slot items from cursor to the slots that can contain the item
    fn move_items_into_slots(&self, window: &mut Window, items_per_slot: i8) {
        for s in &self.slots {
            let slot = &mut *window.inner.item(*s).unwrap();
            if !slot.is_mergable(window.cursor_item()) {
                continue;
            }

            window.cursor_item.transfer_to(items_per_slot, slot);
            if window.cursor_item().is_empty() {
                break;
            };
        }
    }

    fn handle_right_drag(&self, window: &mut Window) {
        self.move_items_into_slots(window, 1)
    }
}

#[derive(Debug)]
enum Mouse {
    Left,
    Right,
}