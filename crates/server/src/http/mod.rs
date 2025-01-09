use axum::{routing::get, Router};

use crate::state::SharedAppState;

mod routes;

pub fn make_router(state: SharedAppState) -> Router {
    Router::new()
        .route("/hello", get(routes::hello_world))
        .with_state(state)
}
