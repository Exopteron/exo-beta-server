pub struct NibbleArray {
    flag: bool,
    backing: Vec<u8>,
}
impl NibbleArray {
    pub fn new() -> Self {
        Self { flag: false, backing: Vec::new() }
    }
    pub fn push(&mut self, v: u8) {
        if self.flag {
            let v2 = self.backing.pop().unwrap();
            self.push(make_nibble_byte(v2, v).unwrap());
        } else {
            self.push(v);
        }
        self.flag ^= true;
    }
    pub fn get_backing(&self) -> &[u8] {
        &self.backing
    }
}
#[inline(always)]
fn make_nibble_byte(mut a: u8, mut b: u8) -> Option<u8> {
    if a > 15 || b > 15 {
        return None;
    }
    b <<= 4;
    b &= 0b11110000;
    a &= 0b00001111;
    Some(a | b)
}
#[inline(always)]
fn decompress_nibble(input: u8) -> (u8, u8) {
    let b = input & 0b11110000;
    let b = b >> 4;
    let a = input & 0b00001111;
    (a, b)
}