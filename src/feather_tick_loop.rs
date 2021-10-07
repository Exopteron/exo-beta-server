// please forgive me for stealing i just needed something quick i'm very sorry

use std::time::Instant;
use std::time::Duration;
pub const TICK_DURATION: Duration = Duration::from_millis(1000 / 20 as u64);
pub struct TickLoop {
    function: Box<dyn FnMut() -> bool>,
}

impl TickLoop {
    pub fn new(function: impl FnMut() -> bool + 'static) -> Self {
        Self {
            function: Box::new(function),
        }
    }

    pub fn run(mut self) {
        loop {
            let start = Instant::now();
            let should_exit = (self.function)();
            if should_exit {
                return;
            }

            let elapsed = start.elapsed();
            if elapsed > TICK_DURATION {
                log::warn!("Tick took too long ({:?})", elapsed);
            } else {
                std::thread::sleep(TICK_DURATION - elapsed);
            }
        }
    }
}