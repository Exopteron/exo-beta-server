use anyhow::anyhow;
#[derive(Clone)]
pub enum Element {
  SByte { byte: i8 },
  RawByte { byte: u8 },
  Int { int: i32 },
  Long { long: i64 },
  Float { float: f32 },
  Double { double: f64 },
  String8 { string: String },
  String16 { string: String },
  Short { short: i16 },
  Bytes { bytes: Vec<u8> },
}
pub struct ClassicPacketBuilder {
  elements: Vec<Element>,
}
impl ClassicPacketBuilder {
  pub fn new() -> Self {
    return Self {
      elements: Vec::new(),
    };
  }
  pub fn insert_string(&mut self, string: &str) {
    self.elements.push(Element::String8 {
      string: string.to_string(),
    });
  }
  pub fn insert_string16(&mut self, string: &str) {
    self.elements.push(Element::String16 {
      string: string.to_string(),
    });
  }
  pub fn insert_bytearray(&mut self, bytes: Vec<u8>) {
    self.elements.push(Element::Bytes { bytes: bytes });
  }
  pub fn insert_sbyte(&mut self, byte: i8) {
    self.elements.push(Element::SByte { byte: byte });
  }
  pub fn insert_byte(&mut self, byte: i8) {
    self.elements.push(Element::SByte { byte: byte });
  }
  pub fn insert_byte_raw(&mut self, byte: u8) {
    self.elements.push(Element::RawByte { byte: byte });
  }
  pub fn insert_short(&mut self, short: i16) {
    self.elements.push(Element::Short { short: short });
  }
  pub fn insert_int(&mut self, int: i32) {
    self.elements.push(Element::Int { int: int });
  }
  pub fn insert_long(&mut self, long: i64) {
    self.elements.push(Element::Long { long: long });
  }
  pub fn insert_float(&mut self, float: f32) {
    self.elements.push(Element::Float { float: float });
  }
  pub fn insert_double(&mut self, double: f64) {
    self.elements.push(Element::Double { double: double });
  }
  pub fn build(mut self, id: u8) -> anyhow::Result<Vec<u8>> {
    let mut packet = vec![id];
    packet.append(&mut self.internal_builder()?);
    return Ok(packet);
  }
  fn internal_builder(&mut self) -> anyhow::Result<Vec<u8>> {
    let mut packet = vec![];
    for element in self.elements.clone() {
      match element.clone() {
        Element::String16 { string } => {
          let utf16 = string.encode_utf16().collect::<Vec<u16>>();
          packet.append(&mut (utf16.len() as i16).to_be_bytes().to_vec());
          for short in utf16 {
              packet.append(&mut short.to_be_bytes().to_vec());
          }
        }
        Element::Float { float } => {
          packet.append(&mut float.to_be_bytes().to_vec());
        }
        Element::Double { double } => {
          packet.append(&mut double.to_be_bytes().to_vec());
        }
        Element::String8 { string } => {
          let mut string = string.as_bytes().to_vec();
          packet.append(&mut (string.len() as i16).to_be_bytes().to_vec());
          packet.append(&mut string);
        }
        Element::RawByte { byte } => {
          packet.push(byte);
        }
        Element::SByte { byte } => {
          packet.push(byte.to_le_bytes()[0]);
        }
        Element::Short { short } => {
          packet.append(&mut short.to_be_bytes().to_vec());
        }
        Element::Int { int } => {
          packet.append(&mut int.to_be_bytes().to_vec());
        }
        Element::Long { long } => {
          packet.append(&mut long.to_be_bytes().to_vec());
        }
        Element::Bytes { mut bytes } => {
          packet.append(&mut bytes);
        }
      }
    }
    return Ok(packet);
  }
}
