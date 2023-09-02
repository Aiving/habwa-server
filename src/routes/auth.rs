use std::sync::Arc;

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};

use crate::data::app;

pub(crate) async fn execute(
    State(state): State<Arc<app::State>>,
    Json(payload): Json<AuthData>,
) -> Result<(StatusCode, Json<AuthResponse>), StatusCode> {
    if state.access_token.is_none() || state.encryption_key.is_none() {
        return Err(StatusCode::NOT_IMPLEMENTED);
    }

    let access_token = state.access_token.as_ref().unwrap();
    let encryption_key = state.encryption_key.as_ref().unwrap();
    let password = payload.password;

    if &crate::utils::encode_password(password, encryption_key.clone()) == access_token {
        return Ok((
            StatusCode::CREATED,
            Json(AuthResponse {
                access_token: access_token.clone(),
            }),
        ));
    }

    Err(StatusCode::UNAUTHORIZED)
}

// the input to our `create_user` handler
#[derive(Serialize)]
pub(crate) struct AuthResponse {
    access_token: String,
}

#[derive(Deserialize)]
pub(crate) struct AuthData {
    password: String,
}
