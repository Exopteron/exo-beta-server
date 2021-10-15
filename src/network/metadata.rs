#[derive(Debug, Clone)]
pub struct Metadata {
    pub bytes: Vec<u8>,
}
impl Metadata {
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
    }
    pub fn insert_byte(&mut self, byte: u8) {
        self.bytes.push(0x00);
        self.bytes.push(byte);
    }
    pub fn insert_byte_idx(&mut self, byte: u8, mut idx: u8) {
        let mut index = 0;
        idx &= 0b00011111;
        index |= idx;
        self.bytes.push(index);
        self.bytes.push(byte);
    }
    pub fn finish(&self) -> Vec<u8> {
        let mut bytes = self.bytes.clone();
        bytes.push(0x7F);
        bytes
    }
}
