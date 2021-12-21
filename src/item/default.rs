use super::item::{ItemRegistry, Item, ItemIdentifier};

pub struct GenericBlock {
    id: ItemIdentifier,
}
impl Item for GenericBlock {
    fn id(&self) -> ItemIdentifier {
        self.id
    }

    fn stack_size(&self) -> i8 {
        64
    }

    fn durability(&self) -> Option<i16> {
        None
    }
}
pub fn register_items(registry: &mut ItemRegistry) {
    for i in 0..111 {
        registry.register_item(Box::new(GenericBlock { id: (i, 0) }));
    }
    for i in 0..16 {
        registry.register_item(Box::new(GenericBlock { id: (35, i) })); // Wool
    }
    registry.register_item(Box::new(GenericBlock { id: (17, 1) })); // Spruce Log
    registry.register_item(Box::new(GenericBlock { id: (17, 2) })); // Birch Log
    registry.register_item(Box::new(GenericBlock { id: (20, 0) })); // Glass
    registry.register_item(Box::new(GenericBlock { id: (323, 0) })); // Glass
}