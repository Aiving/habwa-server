use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};

use crate::data::{app, server};

pub(crate) async fn execute(
    State(state): State<Arc<app::State>>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<server::Properties>), StatusCode> {
    let authorization = headers.get("Authorization");

    if state.access_token.is_none()
        || state.access_token.clone().is_some_and(|token| {
            authorization.is_some_and(|header| header.to_str().unwrap() == token)
        })
    {
        return Ok((StatusCode::OK, Json(state.properties.clone())));
    }

    Err(StatusCode::UNAUTHORIZED)
}
