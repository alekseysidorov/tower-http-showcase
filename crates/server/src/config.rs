use std::time::Duration;
use std::{env, net::SocketAddr, str::FromStr};

use eyre::WrapErr as _;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct AppConfig {
    pub nodes_count: u16,
    pub response_delays: ResponseDelays,
    pub worker_id: String,
    pub listen_addr: SocketAddr,
    pub worker_delay: Duration,
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
            worker_id: "tokio-worker-1".to_owned(),
            listen_addr: SocketAddr::from(([0, 0, 0, 0], 8080)),
            worker_delay: Duration::ZERO,
        }
    }
}

impl AppConfig {
    pub fn from_env() -> eyre::Result<Self> {
        let mut config = Self::default();
        config.worker_id = env::var("WORKER_ID").unwrap_or(config.worker_id);
        config.listen_addr = parse_env("LISTEN_ADDR", config.listen_addr)?;
        config.worker_delay = Duration::from_millis(parse_env("WORKER_DELAY_MS", 0_u64)?);
        Ok(config)
    }
}

fn parse_env<T>(name: &str, default: T) -> eyre::Result<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let Some(value) = env::var_os(name) else {
        return Ok(default);
    };
    let value = value
        .into_string()
        .map_err(|_| eyre::eyre!("{name} is not valid Unicode"))?;
    value
        .parse()
        .wrap_err_with(|| format!("invalid value for {name}: {value:?}"))
}
