#![forbid(unsafe_code)]
use std::convert::TryInto;

use super::item::{Item, AtomicRegistryItem, ItemRegistry};

/// Represents an item stack.
///
/// An item stack includes an item type, an amount and a bunch of properties (enchantments, etc.)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ItemStack {
    /// The item type of this `ItemStack`.
    item: AtomicRegistryItem,

    /// The number of items in the `ItemStack`.
    count: i8,

    meta: i16,
}

impl ItemStack {
    /// Creates a new `ItemStack` with the default name (title)
    /// no lore, no damage, no repair cost and no enchantments.
    pub fn new(item: AtomicRegistryItem, count: i8, meta: i16) -> Self {
        Self { item, count, meta }
    }

    /// Returns whether the given item stack has
    /// the same type as (but not necessarily the same
    /// amount as) `self`.
    pub fn has_same_type(&self, other: &Self) -> bool {
        self.item.id() == other.item.id()
    }

    /// Returns whether the given item stack has the same damage
    /// as `self`.
    pub fn has_same_damage(&self, other: &Self) -> bool {
        self.meta == other.meta
    }

    /// Returns whether the given `ItemStack` has
    /// the same count as (but not necessarily the same
    /// type as) `self`.
    pub fn has_same_count(&self, other: &Self) -> bool {
        self.count == other.count
    }

    /// Returns whether the given `ItemStack` has the same
    /// type and count as (but not necessarily the same meta
    /// as) `self`.
    pub fn has_same_type_and_count(&self, other: &Self) -> bool {
        self.item.id() == other.item.id() && self.count == other.count
    }

    /// Returns whether the given `ItemStack` has
    /// the same type and damage as `self`.
    pub fn has_same_type_and_damage(&self, other: &Self) -> bool {
        self.item.id() == other.item.id() && self.has_same_damage(other)
    }

    /// Returns the item type for this `ItemStack`.
    pub fn item(&self) -> AtomicRegistryItem {
        self.item.clone()
    }

    /// Returns the number of items in this `ItemStack`.
    pub fn count(&self) -> i8 {
        self.count
    }

    /// Adds more items to this `ItemStack`. Returns the new count.
    pub fn add(&mut self, count: i8) -> Result<i8, ItemStackError> {
        self.set_count(self.count + count)
    }

    /// Adds more items to this `ItemStack`. Does not check if the
    /// addition will make the count to be greater than the
    /// stack size. Does not check count overflows. Returns the new count.
    pub fn unchecked_add(&mut self, count: i8) -> i8 {
        self.count = self.count + count;
        self.count
    }

    /// Removes some items from this `ItemStack`.
    pub fn remove(&mut self, count: i8) -> Result<i8, ItemStackError> {
        if self.count <= count {
            return Err(if self.count == count {
                ItemStackError::EmptyStack
            } else {
                ItemStackError::NotEnoughAmount
            });
        }
        Ok(self.count)
    }

    /// Sets the item type for this `ItemStack`. Returns the new
    /// item type or fails if the current item count exceeds the
    /// new item type stack size.
    pub fn set_item(&mut self, item: AtomicRegistryItem) -> Result<AtomicRegistryItem, ItemStackError> {
        if self.count > item.stack_size() {
            return Err(ItemStackError::ExceedsStackSize);
        }
        self.item = item;
        Ok(self.item.clone())
    }

    /// Gets the `ItemStack` and returns it.
    pub fn get_item(&self) -> ItemStack {
        ItemStack {
            count: 1.try_into().unwrap(),
            ..self.clone()
        }
    }

    /// Sets the item type for this `ItemStack`. Does not check if
    /// the new item type stack size will be lower than the current
    /// item count. Returns the new item type.
    pub fn unchecked_set_item(&mut self, item: AtomicRegistryItem) -> AtomicRegistryItem {
        self.item = item;
        self.item.clone()
    }

    /// Sets the count for this `ItemStack`. Returns the updated
    /// count or fails if the new count would exceed the stack
    /// size for that item type.
    pub fn set_count(&mut self, count: i8) -> Result<i8, ItemStackError> {
        if count == 0 {
            return Err(ItemStackError::EmptyStack);
        }
        if count > self.item.stack_size() {
            return Err(ItemStackError::ExceedsStackSize);
        } else if count > i8::MAX {
            return Err(ItemStackError::ClientOverflow);
        }
        self.count = count;
        Ok(self.count)
    }

    /// Sets the count for this `ItemStack`. It will not check if
    /// the desired count exceeds the current item type stack size.
    /// Does not check count overflows or if the parameter is zero.
    /// Returns the updated count.
    pub fn unchecked_set_count(&mut self, count: i8) -> i8 {
        self.count = count;
        self.count
    }

    /// Splits this `ItemStack` in half, returning the
    /// removed half. If the amount is odd, `self`
    /// will be left with the least items. Returns the taken
    /// half.
    pub fn take_half(self) -> (Option<ItemStack>, ItemStack) {
        let half = (self.count + 1) / 2;
        self.take(half)
    }

    /// Splits this `ItemStack` by removing the
    /// specified amount. Returns the taken part.
    pub fn take(mut self, amount: i8) -> (Option<ItemStack>, ItemStack) {
        if self.count < amount {
            return (None, self);
        }
        let count_left = self.count - amount;
        let taken = ItemStack {
            count: amount,
            ..self.clone()
        };
        self.count = count_left;
        (Some(self), taken)
    }

    /// Merges another `ItemStack` with this one.
    pub fn merge_with(&mut self, other: Self) -> Result<(), ItemStackError> {
        if !self.has_same_type_and_damage(&other) {
            return Err(ItemStackError::IncompatibleStacks);
        }
        let new_count = (self.count + other.count).min(self.item.stack_size());
        self.count = new_count;
        //other.count = NonZeroU32::new(other.count() - amount_added).unwrap();
        Ok(())
    }

    /// Transfers up to `n` items to `other`.
    pub fn transfer_to(&mut self, n: i8, other: &mut Self) -> Result<(), ItemStackError> {
        if self.count <= n || n == 0 {
            return Err(if self.count == n || n == 0 {
                ItemStackError::EmptyStack
            } else {
                ItemStackError::NotEnoughAmount
            });
        }
        let max_transfer = other.item.stack_size().saturating_sub(other.count);
        let transfer = max_transfer.min(self.count).min(n);
        if other.count + transfer > i8::MAX {
            return Err(ItemStackError::ClientOverflow);
        }

        self.count = self.count - transfer;
        other.count = other.count + transfer;
        Ok(())
    }

    pub fn drain_into_bounded(
        mut self,
        n: i8,
        other: &mut Self,
    ) -> Result<Option<Self>, ItemStackError> {
        if !self.has_same_type(other) {
            return Err(ItemStackError::IncompatibleStacks);
        }

        // Stack size is the same for both self and other because they are the same type.
        let stack_size = self.item.stack_size();
        let space_in_other = stack_size - other.count();
        let items_in_self = self.count();
        let moving_items = space_in_other.min(n).min(items_in_self);

        other.set_count(moving_items + other.count()).unwrap();

        if self.count() - moving_items == 0 {
            Ok(None)
        } else {
            self.set_count(moving_items - items_in_self).unwrap();
            Ok(Some(self))
        }
    }

    /// Damages the item by the specified amount.
    /// If this function returns `true`, then the item is broken.
    pub fn damage(&mut self, amount: i16) -> bool {
        self.meta += amount;
        if let Some(durability) = self.item.durability() {
            // This unwrap would only fail if our generated file contains an erroneous
            // default damage value.
            self.meta >= durability.try_into().unwrap()
        } else {
            false
        }
    }

    /// Returns the amount of damage the items have taken.
    pub fn damage_taken(&self) -> i16 {
        self.meta
    }

    /// Returns true is the contents of other could be merged with the contents
    /// of self. This does not look at the item count, just the kind.
    /// Items can be merged when they have the same kind, damage, and enchantment.
    /// If a item has a stacksize of one then it can never be stacked.
    pub fn stackable_types(&self, other: &Self) -> bool {
        self.has_same_type(other) &&
        // Todo: make this function check that the items have same name
        // if you rename a item, then it does not stack with items that
        // dont share the rename. Someone need to explore this further.
        self.stack_size() > 1 &&
        other.stack_size() > 1
    }

    /// How many items could be stacked together
    pub fn stack_size(&self) -> i8 {
        self.item.stack_size()
    }
}

/// An error type that may be returned when performing
/// operations over an `ItemStack`.
#[derive(Debug, Clone)]
pub enum ItemStackError {
    ClientOverflow,
    EmptyStack,
    ExceedsStackSize,
    IncompatibleStacks,
    NotEnoughAmount,
}

pub struct ItemStackBuilder {
    item: AtomicRegistryItem,
    count: i8,
    meta: i16,
}

impl Default for ItemStackBuilder {
    fn default() -> Self {
        let item = ItemRegistry::global().get_item((1, 0)).unwrap();
        Self {
            item: item,
            count: 1,
            meta: 0,
        }
    }
}

// Todo: implement all fields.
impl ItemStackBuilder {
    pub fn new() -> Self {
        let item = ItemRegistry::global().get_item((1, 0)).unwrap();
        Self {
            item: item,
            count: 1.try_into().unwrap(),
            meta: 0,
        }
    }

    pub fn with_item(item: AtomicRegistryItem) -> Self {
        Self {
            item,
            count: 1.try_into().unwrap(),
            meta: 0,
        }
    }

    pub fn item(self, item: AtomicRegistryItem) -> Self {
        Self { item, ..self }
    }

    // panics if the count is zero
    pub fn count(self, count: u32) -> Self {
        Self {
            count: count.try_into().unwrap(),
            ..self
        }
    }

    pub fn damage(mut self, damage: i16) -> Self {
        self.meta = damage;
        self
    }

    /// If damage is some, then its value is applied, else this is a no-op.
    pub fn apply_damage(self, damage: Option<i16>) -> Self {
        match damage {
            Some(damage) => self.damage(damage),
            None => self,
        }
    }

    pub fn same_meta_as(mut self, other: &Self) -> Self {
        self.meta = other.meta.clone();
        self
    }
}

impl From<ItemStackBuilder> for ItemStack {
    fn from(it: ItemStackBuilder) -> Self {
        Self {
            item: it.item,
            count: it.count,
            meta: it.meta,
        }
    }
}
