use serde::{Deserialize, Serialize};

pub const DEFAULT_SERVER_PORT: u16 = 8080;

#[derive(Debug, Deserialize, Serialize)]
pub struct HelloMessage {
    pub message: String,
}
