use crate::entities::metadata::EntityBitMask;

#[derive(Debug, Clone)]
pub struct Metadata {
    pub flags: EntityBitMask,
}
impl Metadata {
    pub fn new() -> Self {
        Self { flags: EntityBitMask::empty() }
    }
    pub fn finish(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(0x00);
        bytes.push(self.flags.bits());
        bytes.push(0x7F);
        bytes
    }
}
