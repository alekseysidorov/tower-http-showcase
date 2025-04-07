use std::time::Duration;

use rand::Rng as _;

#[derive(Debug)]
pub struct DelayIter {
    #[allow(dead_code)]
    node_id: u32,
    current_delay: (u32, Duration),
    delay_range: std::ops::Range<Duration>,
}

impl DelayIter {
    const CONST_DELAY_PERIOD: u32 = 50;

    pub fn new(node_id: u32, delay_range: std::ops::Range<Duration>) -> Self {
        Self {
            node_id,
            current_delay: (
                Self::CONST_DELAY_PERIOD,
                rand::rng().random_range(delay_range.clone()),
            ),
            delay_range,
        }
    }
}

impl Iterator for DelayIter {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_delay.0 > 0 {
            self.current_delay.0 -= 1;
        } else {
            self.current_delay.0 = Self::CONST_DELAY_PERIOD;
            self.current_delay.1 = rand::rng().random_range(self.delay_range.clone());
        }

        Some(self.current_delay.1)
    }
}
