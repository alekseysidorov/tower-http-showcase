use std::time::Duration;

use rand::Rng as _;

#[derive(Debug)]
pub struct DelayIter {
    delay_range: std::ops::Range<Duration>,
}

impl DelayIter {
    pub const fn new(_seed: u16, delay_range: std::ops::Range<Duration>) -> Self {
        Self { delay_range }
    }
}

impl Iterator for DelayIter {
    type Item = Duration;

    fn next(&mut self) -> Option<Self::Item> {
        rand::thread_rng()
            .gen_range(self.delay_range.clone())
            .into()
    }
}
