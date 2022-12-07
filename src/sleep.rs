pub const NUM_TICKS_TO_SLEEP: u128 = 101;

pub struct SleepManager {
    pub num_ticks: u128
}

impl SleepManager {
    pub fn new() -> Self {
        Self {
            num_ticks: 0
        }
    }
    pub fn reset(&mut self) {
        self.num_ticks = 0;
    }

    pub fn update(&mut self) -> bool {
        self.num_ticks += 1;
        if self.num_ticks >= NUM_TICKS_TO_SLEEP {
            self.num_ticks = 0;
            true
        } else {
            false
        }
    }
}