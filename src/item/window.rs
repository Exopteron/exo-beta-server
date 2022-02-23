use std::mem;

use crate::ecs::systems::SysResult;
use crate::item::crafting::print_grid;
use anyhow::{anyhow, bail};

use super::crafting::Grid;
use super::inventory_slot::InventorySlot::Empty;
use super::item::ItemRegistry;
use super::stack::ItemStack;
use super::{
    inventory::{
        reference::{Area, BackingWindow},
        WindowError,
    },
    inventory_slot::InventorySlot,
};
use parking_lot::MutexGuard;

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
fn unwrap_slot(slot: InventorySlot) -> Option<ItemStack> {
    match slot {
        InventorySlot::Filled(i) => Some(i),
        _ => None,
    }
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
    fn is_valid_recipe(&self) -> bool {
        let closure: Box<dyn Fn() -> anyhow::Result<bool>> = Box::new(|| {
            let mut grid = self.get_grid()?;
            let solution = ItemRegistry::global().solver.solve(&mut grid);
            let mut flag = false;
            if let Some(solution) = solution {
                if solution.id() != 35 {
                    // temp fix
                    flag = true;
                }
            }
            Ok(flag)
        });
        closure().unwrap_or(false)
    }
    fn get_grid(&self) -> anyhow::Result<Grid> {
        let mut grid = Grid::default();
        match self.inner() {
            BackingWindow::Player { player } => {
                let mut items = Vec::new();
                for i in 1..5 {
                    items.push(self.item(i)?);
                }
                grid[0][0] = unwrap_slot(items[0].clone());
                grid[0][1] = unwrap_slot(items[1].clone());
                grid[1][0] = unwrap_slot(items[2].clone());
                grid[1][1] = unwrap_slot(items[3].clone());
                drop(items);
            }
            BackingWindow::Crafting {
                crafting_table,
                player,
            } => {
                let mut items = Vec::new();
                for i in 1..10 {
                    items.push(self.item(i)?);
                }
                grid[0][0] = unwrap_slot(items[0].clone());
                grid[0][1] = unwrap_slot(items[1].clone());
                grid[0][2] = unwrap_slot(items[2].clone());
                grid[1][0] = unwrap_slot(items[3].clone());
                grid[1][1] = unwrap_slot(items[4].clone());
                grid[1][2] = unwrap_slot(items[5].clone());

                grid[2][0] = unwrap_slot(items[6].clone());
                grid[2][1] = unwrap_slot(items[7].clone());
                grid[2][2] = unwrap_slot(items[8].clone());
                drop(items);
            }
            _ => return Err(anyhow::anyhow!("NO grid")),
        }
        Ok(grid)
    }
    fn check_crafting(&mut self) -> anyhow::Result<bool> {
        let mut grid = self.get_grid()?;
        let solution = ItemRegistry::global().solver.solve(&mut grid);
        let mut flag = false;
        if let Some(solution) = solution {
            if solution.id() != 35 {
                // temp fix
                *self.item(0)? = InventorySlot::Filled(solution);
                flag = true;
            }
        }
        if !flag {
            *self.item(0)? = InventorySlot::Empty;
        }
        Ok(flag)
    }
    fn clear_grid(&self) -> SysResult {
        if self.is_valid_recipe() {
            match self.inner() {
                BackingWindow::Player { .. } => {
                    for i in 1..5 {
                        self.item(i)?.try_take(1);
                    }
                }
                BackingWindow::Crafting { .. } => {
                    for i in 1..10 {
                        self.item(i)?.try_take(1);
                    }
                }
                _ => (),
            }
        }
        Ok(())
    }
    /// Left-click a slot in the window.
    pub fn left_click(&mut self, slot_idx: usize) -> SysResult {
        let mut t = self.inner.item(slot_idx)?;
        let slot = &mut *t;
        let cursor_slot = &mut self.cursor_item;
        let (_, area, _) = self.inner.index_to_slot(slot_idx).unwrap();
        if can_insert(&area) {
            // Cases:
            // * Either the cursor slot or the clicked slot is empty; swap the two.
            // * Both slots are present but are of different types; swap the two.
            // * Both slots are present and have the same type; merge the two.

            if slot.is_filled() && cursor_slot.is_filled() && cursor_slot.is_mergable(slot) {
                slot.merge(cursor_slot);
            } else {
                mem::swap(cursor_slot, slot);
            }
        } else if slot.is_filled() && cursor_slot.is_filled() && cursor_slot.is_mergable(slot) {
            cursor_slot.merge(slot);
            if let Area::CraftingOutput = area {
                self.clear_grid()?;
            }
        } else if cursor_slot.is_empty() {
            *cursor_slot = mem::replace(slot, InventorySlot::Empty);
            if let Area::CraftingOutput = area {
                self.clear_grid()?;
            }
        }
        drop(t);
        if let Area::CraftingOutput = area {
            self.check_crafting()?;
        }
        if let Area::CraftingInput = area {
            self.check_crafting()?;
        }
        Ok(())
    }

    /// Right-clicks a slot in the window.
    pub fn right_click(&mut self, slot_index: usize) -> SysResult {
        let (_, area, _) = self.inner.index_to_slot(slot_index).unwrap();
        let mut t = self.inner.item(slot_index)?;
        let slot = &mut *t;
        let cursor_slot = &mut self.cursor_item;

        // Cases:
        // * Cursor slot is present and clicked slot has the same item type; drop one item in the clicked slot.
        // * Clicked slot is present but cursor slot is not; move half the items into the cursor slot.
        // * Both slots are present but differ in type; swap the two.

        match (slot.is_filled(), cursor_slot.is_filled()) {
            (true, true) => {
                if slot.is_mergable(cursor_slot) && can_insert(&area) {
                    cursor_slot.transfer_to(1, slot);
                } else if !can_insert(&area) && cursor_slot.is_mergable(slot) {
                    let num_sent = slot.transfer_to(64, cursor_slot);
                    if let Area::CraftingOutput = area {
                        if num_sent > 0 {
                            self.clear_grid()?;
                        }
                    }
                } else if can_insert(&area) {
                    mem::swap(slot, cursor_slot);
                } else {
                    *cursor_slot = mem::replace(slot, InventorySlot::Empty);
                    if let Area::CraftingOutput = area {
                        self.clear_grid()?;
                    }
                }
            }
            (true, false) => {
                if let Area::CraftingOutput = area {
                    *cursor_slot = slot.take_all();
                    self.clear_grid()?;
                } else {
                    *cursor_slot = slot.take_half();
                }
            }
            (false, true) => {
                if can_insert(&area) {
                    *slot = cursor_slot.try_take(1);
                }
            }
            (false, false) => {}
        }
        drop(t);
        if let Area::CraftingOutput = area {
            self.check_crafting()?;
        }
        if let Area::CraftingInput = area {
            self.check_crafting()?;
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
    pub fn insert_item(&mut self, mut slot: InventorySlot) -> anyhow::Result<()> {
        let slot_item = &mut slot;

        let (inventory, slot_area, _) = self.inner.index_to_slot(10).unwrap();
        let areas_to_try = [Area::Hotbar, Area::Storage];

        for &area in &areas_to_try {
            if !will_accept(area, slot_item) {
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
        let mut i = 0;
        if slot_item.is_filled() {
            for &area in &areas_to_try {
                if !will_accept(area, slot_item) {
                    continue;
                }

                // If we still haven't moved all the items, transfer to any empty space
                i = 0;
                while let Some(mut stack) = inventory.item(area, i) {
                    if stack.is_empty() {
                        stack.merge(slot_item);
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
    fn shift_click_in_player_window(&mut self, slot: usize) -> SysResult {
        let mut t = self.inner.item(slot)?;
        let slot_item = &mut *t;

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
        let mut moved = false;
        for &area in &areas_to_try {
            if area == slot_area || !will_accept(area, slot_item) {
                continue;
            }

            // Find slot with same type first
            let mut i = 0;
            while let Some(mut stack) = inventory.item(area, i) {
                if slot_item.is_mergable(&stack) && stack.is_filled() {
                    stack.merge(slot_item);
                    moved = true;
                }
                i += 1;
            }

            if slot_item.is_empty() {
                drop(t);
                if let Area::CraftingOutput = slot_area {
                    if moved {
                        for i in 1..5 {
                            self.item(i)?.try_take(1);
                        }
                    }
                    if self.check_crafting()? {
                        self.shift_click_in_player_window(slot)?;
                    }
                }
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
                        stack.merge(slot_item);
                        moved = true;
                    }
                    i += 1;
                }

                if slot_item.is_empty() {
                    break;
                }
            }
        }
        drop(t);
        if let Area::CraftingOutput = slot_area {
            if moved {
                self.clear_grid()?;
            }
            if self.check_crafting()? {
                self.shift_click_in_player_window(slot)?;
            }
        }

        Ok(())
    }

    fn shift_click_in_generic_window(&mut self, slot: usize) -> SysResult {
        let (_, slot_area, _) = self.inner.index_to_slot(slot).unwrap();
        let (inventory, _, _) = self.inner.index_to_slot(28).unwrap();
        let mut t = self.inner.item(slot)?;
        println!("Past here");
        let slot_item = &mut *t;
        let areas_to_try = [Area::Hotbar, Area::Storage];
        let mut moved = false;
        for &area in &areas_to_try {
            if !will_accept(area, slot_item) {
                continue;
            }

            // Find slot with same type first
            let mut i = 0;
            loop {
                match inventory.try_item(area, i) {
                    Ok(v) => {
                        if let Some(mut stack) = v {
                            if slot_item.is_mergable(&stack) && stack.is_filled() {
                                stack.merge(slot_item);
                                moved = true;
                            }
                            i += 1;
                        } else {
                            break;
                        }
                    }
                    Err(_) => {
                        i += 1;
                        continue;
                    }
                }
            }

            if slot_item.is_empty() {
                drop(t);
                return Ok(());
            }
        }

        if slot_item.is_filled() {
            //log::info!("Filled");
            for &area in &areas_to_try {
                if !will_accept(area, slot_item) {
                    continue;
                }

                // If we still haven't moved all the items, transfer to any empty space
                let mut i = 0;
                loop {
                    match inventory.try_item(area, i) {
                        Ok(v) => {
                            if let Some(mut stack) = v {
                                if stack.is_empty() {
                                    stack.merge(slot_item);
                                    moved = true;
                                }
                                i += 1;
                            } else {
                                break;
                            }
                        }
                        Err(_) => {
                            i += 1;
                            continue;
                        }
                    }
                }
                if slot_item.is_empty() {
                    break;
                }
            }
        }
        drop(t);
        Ok(())
    }

    fn shift_click_in_crafting_window(&mut self, slot: usize) -> SysResult {
        let (_, slot_area, _) = self.inner.index_to_slot(slot).unwrap();
        let (inventory, _, _) = self.inner.index_to_slot(12).unwrap();
        let mut t = self.inner.item(slot)?;
        let slot_item = &mut *t;
        let areas_to_try = [
            Area::Helmet,
            Area::Chestplate,
            Area::Leggings,
            Area::Boots,
            Area::CraftingInput,
            Area::Hotbar,
            Area::Storage,
        ];
        let mut moved = false;
        for &area in &areas_to_try {
            if area == slot_area || !will_accept(area, slot_item) {
                continue;
            }

            // Find slot with same type first
            let mut i = 0;
            while let Some(mut stack) = inventory.item(area, i) {
                if slot_item.is_mergable(&stack) && stack.is_filled() {
                    stack.merge(slot_item);
                    moved = true;
                }
                i += 1;
            }

            if slot_item.is_empty() {
                drop(t);
                if let Area::CraftingOutput = slot_area {
                    if moved {
                        self.clear_grid()?;
                    }
                    if self.check_crafting()? {
                        self.shift_click_in_crafting_window(slot)?;
                    }
                }
                return Ok(());
            }
        }

        if slot_item.is_filled() {
            //log::info!("Filled");
            for &area in &areas_to_try {
                if area == slot_area || !will_accept(area, slot_item) {
                    continue;
                }

                // If we still haven't moved all the items, transfer to any empty space
                let mut i = 0;
                while let Some(mut stack) = inventory.item(area, i) {
                    if stack.is_empty() {
                        stack.merge(slot_item);
                        moved = true;
                    }
                    i += 1;
                }

                if slot_item.is_empty() {
                    break;
                }
            }
        }
        drop(t);
        if let Area::CraftingOutput = slot_area {
            if moved {
                self.clear_grid()?;
            }
            if self.check_crafting()? {
                self.shift_click_in_crafting_window(slot)?;
            }
        }

        Ok(())
    }

    fn shift_click_in_furnace(&mut self, _slot: usize) -> SysResult {
        Ok(())
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

    /// Mutably borrows the item currently held in the cursor.
    pub fn cursor_item_mut(&mut self) -> &mut InventorySlot {
        &mut self.cursor_item
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

fn can_insert(area: &Area) -> bool {
    match area {
        Area::CraftingOutput => false,
        Area::FurnaceOutput => false,
        _ => true,
    }
}
