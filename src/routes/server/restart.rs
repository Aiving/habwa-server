use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
};
// use rcon_client::{RCONClient, RCONConfig, RCONRequest};
// use tokio::process::Command;

use crate::data::app;

pub(crate) async fn execute(
    State(state): State<Arc<app::State>>,
    headers: HeaderMap,
) -> Result<StatusCode, StatusCode> {
    let authorization = headers.get("Authorization");

    if state.access_token.is_none()
        || state.access_token.clone().is_some_and(|token| {
            authorization.is_some_and(|header| header.to_str().unwrap() == token)
        })
    {
        // state
        //     .rcon
        //     .lock()
        //     .await
        //     .execute(RCONRequest::new("stop".to_string()))
        //     .expect("failed to stop server");

        // Command::new("sh")
        //     .arg("-C")
        //     .arg(state.path.join("run.sh").to_str().unwrap())
        //     .spawn()
        //     .expect("failed to restart server");

        // state.rcon.lock().await.socket.shutdown(how)

        // *(state.rcon.lock().await) = RCONClient::new(RCONConfig {
        //     url: format!(
        //         "{}:{}",
        //         state.properties.clone().ip.unwrap_or("0.0.0.0".to_string()),
        //         state.properties.rcon.port
        //     ),
        //     // Optional
        //     read_timeout: None,
        //     write_timeout: None,
        // })
        // .expect("failed intialize connection to rcon");

        return Ok(StatusCode::OK);
    }

    Err(StatusCode::UNAUTHORIZED)
}
