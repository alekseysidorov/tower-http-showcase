use std::sync::Arc;

#[derive(Debug)]
pub struct AppState;

pub type SharedAppState = Arc<AppState>;
