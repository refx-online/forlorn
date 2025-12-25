use std::collections::HashMap;

use axum::{
    body::Bytes,
    extract::{Multipart, State},
    http::StatusCode,
    response::{IntoResponse, Response},
};
use md5::{Digest, Md5};

use crate::{
    dto::error::GetError, models::ClientError, repository, state::AppState,
    utils::build_error_upload,
};

async fn parse_typed_multipart(multipart: &mut Multipart) -> Result<GetError, Response> {
    let mut fields: HashMap<String, Bytes> = HashMap::new();

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let name = field.name().unwrap_or_default().to_owned();
        let content = field.bytes().await.unwrap_or_default();
        fields.insert(name, content);
    }

    match build_error_upload(fields) {
        Some(upload) => Ok(upload),
        None => Err((StatusCode::BAD_REQUEST, "error: invalid data").into_response()),
    }
}

pub async fn get_error(
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let error = match parse_typed_multipart(&mut multipart).await {
        Ok(e) => e,
        Err(response) => return response,
    };

    let user_id: i32 = error.user_id.unwrap_or(-1);

    let mut client_error = ClientError::from_error(error);

    let username = match repository::user::fetch_by_id(&state.db, &user_id).await {
        Ok(Some(user)) => user.name(),
        _ => "Offline user".into(),
    };

    if user_id != -1 {
        client_error.username = username;
    }

    tracing::info!(
        "{} sent exception: {:?} ({:?})",
        client_error.username,
        client_error.feedback,
        client_error.exception
    );

    tracing::info!(
        "{} sent stacktrace ({:?})",
        client_error.username,
        client_error.stacktrace
    );

    let _ = state.metrics.incr("error.experienced", ["status:ok"]);

    if let Some(screenshot_data) = client_error.screenshot_data.take() {
        let ext = if screenshot_data.len() > 10
            && (&screenshot_data[6..10] == b"JFIF" || &screenshot_data[6..10] == b"Exif")
        {
            "jpeg"
        } else if screenshot_data.starts_with(b"\x89PNG\r\n\x1a\n") {
            "png"
        } else {
            return (StatusCode::BAD_REQUEST, "file type").into_response();
        };

        let mut hasher = Md5::new();
        hasher.update(&screenshot_data);

        let hash = format!("{:x}", hasher.finalize());

        let file_name = format!("{}.{}", &hash[..8], ext);
        let path = state.storage.screenshot_file(&file_name);

        tokio::spawn(async move {
            let _ = tokio::fs::write(&path, &screenshot_data).await;
        });
    }

    tokio::spawn(async move {
        let _ = repository::error::insert(&state.db, &client_error).await;
    });

    (StatusCode::OK).into_response()
}
