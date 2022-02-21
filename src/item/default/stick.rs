use crate::item::item::Item;

pub struct StickItem;
impl Item for StickItem {
    fn id(&self) -> crate::item::item::ItemIdentifier {
        280
    }

    fn stack_size(&self) -> i8 {
        64
    }

    fn durability(&self) -> Option<i16> {
        None
    }
}