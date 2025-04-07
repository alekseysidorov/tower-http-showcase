use std::time::Duration;

use humantime_serde;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AppConfig {
    pub nodes_count: u16,
    pub response_delays: ResponseDelays,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResponseDelays {
    #[serde(with = "humantime_serde")]
    pub min: Duration,
    #[serde(with = "humantime_serde")]
    pub max: Duration,
}

impl Default for ResponseDelays {
    fn default() -> Self {
        Self {
            min: Duration::from_millis(1),
            max: Duration::from_millis(250),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            nodes_count: 16,
            response_delays: Default::default(),
        }
    }
}
