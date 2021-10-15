use bytes::*;
pub trait NetMessage {
    fn get_packet_id(&self) -> i8;
    fn to_bytes(&self) -> Bytes;
}