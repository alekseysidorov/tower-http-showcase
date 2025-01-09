use axum::{extract::State, Json};
use showcase_api::model::{HelloRequest, HelloResponse};

use crate::state::SharedAppState;

pub async fn hello_world(
    State(_state): State<SharedAppState>,
    Json(request): Json<HelloRequest>,
) -> Json<HelloResponse> {
    Json(HelloResponse {
        message: format!("Hello, {}!", request.name),
    })
}
