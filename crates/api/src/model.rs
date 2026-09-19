use fractal_core::ComplexRegion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HelloRequest {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct HelloResponse {
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeKind {
    Tokio,
    Compio,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderTileRequest {
    pub tile_id: u32,
    pub region: ComplexRegion,
    pub width: u32,
    pub height: u32,
    pub max_iterations: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderTileResponse {
    pub tile_id: u32,
    pub worker_id: String,
    pub runtime: RuntimeKind,
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
    pub render_duration_ms: f64,
    pub artificial_delay_ms: f64,
}
