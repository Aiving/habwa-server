use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use rcon_client::RCONRequest;
use serde::Serialize;

use crate::{
    data::{app, server::Player, UUID_REX},
    utils,
};

#[derive(Serialize)]
pub(crate) struct Players {
    online: Vec<Player>,
    offline: Vec<Player>,
}

pub(crate) async fn execute(
    State(state): State<Arc<app::State>>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Players>), StatusCode> {
    let authorization = headers.get("Authorization");

    if state.access_token.clone().is_some_and(|token| {
        authorization.is_none()
            || authorization.is_some_and(|header| header.to_str().unwrap() != token)
    }) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut online_uuids = vec![];

    if let Some(client) = &mut *state.rcon.lock().await {
        let body = client
            .execute(RCONRequest::new("list uuids".to_string()))
            .unwrap()
            .body;

        for (_, [uuid]) in UUID_REX.captures_iter(&body).map(|c| c.extract()) {
            online_uuids.push(uuid.to_string());
        }
    }

    let players = utils::get_players(state.path.clone(), online_uuids).await;
    let online = players
        .iter()
        .filter(|player| player.last_join_time.is_none())
        .cloned()
        .collect();
    let offline: Vec<Player> = players
        .iter()
        .filter(|player| player.last_join_time.is_some())
        .cloned()
        .collect();

    Ok((StatusCode::OK, Json(Players { online, offline })))
}
