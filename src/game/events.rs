use flume::{Receiver, Sender};
use super::*;
use std::any::Any;
mod block;
pub use block::*;
use std::collections::VecDeque;
#[derive(Clone)]
pub struct EventHolder {
    pub event: Arc<Box<dyn Event>>
}
pub trait Event {
    fn as_any(&self) -> &dyn Any;
    fn set_cancelled(&mut self, state: bool);
    fn get_cancelled(&self) -> bool;
}
type HandlerFn = Box<fn(&mut Box<EventHolder>, &mut Game) -> bool>;
pub struct EventHandler {
    event_queue: VecDeque<EventHolder>,
    handlers: Vec<HandlerFn>
}
impl EventHandler {
    pub fn new() -> Self {
        Self { event_queue: VecDeque::new(), handlers: Vec::new() }
    }
    pub fn register_handler(&mut self, handler: HandlerFn) {
        self.handlers.push(handler);
    }
    pub fn cause_event(&mut self, event: Box<dyn Event>) -> anyhow::Result<()> {
        self.event_queue.push_back(EventHolder { event: Arc::new(event)} );
        Ok(())
    }
    pub fn handle_events(&mut self, game: &mut Game) {
        'main: loop {
            let event = if let Some(ev) = self.event_queue.pop_front() {
                ev
            } else {
                break;
            };
            let event = &mut Box::new(event);
            for handler in &self.handlers {
                handler(event, game);
                if event.event.get_cancelled() {
                    continue 'main;
                }
            }
        }
    }
}