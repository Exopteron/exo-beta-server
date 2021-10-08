use super::*;
pub struct BlockPlacementEvent {
    pub cancelled: bool,
    pub player: Arc<PlayerRef>,
    pub packet: crate::network::packet::PlayerBlockPlacement,
}
impl Event for BlockPlacementEvent {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn get_cancelled(&self) -> bool {
        self.cancelled
    }
    fn set_cancelled(&mut self, state: bool) {
        self.cancelled = state;
    }
}