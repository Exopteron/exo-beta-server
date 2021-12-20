use super::*;
use crate::{protocol::io::String16};

packets! {
    ChatMessage {
        message String16;
    }
    PlayerPositionAndLook {
        x f64;
        y f64;
        stance f64;
        z f64;
        yaw f32;
        pitch f32;
        on_ground bool;
    }
}

