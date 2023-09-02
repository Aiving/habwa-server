use std::{io::Cursor, sync::Arc};

use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use tokio::fs;

use crate::{
    data::app,
    loaders::{self, forge::Mod},
};

pub(crate) async fn execute(
    State(state): State<Arc<app::State>>,
    headers: HeaderMap,
) -> Result<(StatusCode, Json<Vec<Mod>>), StatusCode> {
    let authorization = headers.get("Authorization");

    if state.access_token.clone().is_some_and(|token| {
        authorization.is_none()
            || authorization.is_some_and(|header| header.to_str().unwrap() != token)
    }) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok((StatusCode::OK, Json(state.mods.lock().await.clone())))
}

struct ModFile {
    filename: String,
    data: Bytes,
}

pub(crate) async fn upload(
    State(state): State<Arc<app::State>>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Vec<Mod>>), StatusCode> {
    let authorization = headers.get("Authorization");

    if state.access_token.clone().is_some_and(|token| {
        authorization.is_none()
            || authorization.is_some_and(|header| header.to_str().unwrap() != token)
    }) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let mut files = vec![];

    while let Some(field) = multipart
        .next_field()
        .await
        .expect("failed to get next field in multipart")
    {
        let name = field.name().unwrap().to_string();
        let filename = field.file_name().unwrap().to_string();
        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();

        if name == "file" && content_type == "application/x-java-archive" {
            files.push(ModFile { filename, data });
        }
    }

    if files.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut mods = vec![];

    for file in files {
        let data = file.data.to_vec();
        let is_mod = loaders::forge::is_mod(data.as_ref()).await;

        if !is_mod {
            return Err(StatusCode::BAD_REQUEST);
        }

        fs::write(state.path.join(format!("mods/{}", file.filename)), &data)
            .await
            .expect("failed to upload a mod");

        mods.append(&mut loaders::forge::load_mod(Cursor::new(data)).await)
    }

    state.mods.lock().await.append(&mut mods);

    Ok((StatusCode::OK, Json(state.mods.lock().await.clone())))
}
