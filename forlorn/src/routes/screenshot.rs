use std::collections::HashMap;

use axum::{
    body::Bytes,
    extract::{Multipart, Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use tokio::fs;

use crate::{
    dto::screenshot::ScreenshotUpload,
    models::User,
    repository,
    state::AppState,
    usecases::password::verify_password,
    utils::{build_screenshot_upload, save_screenshot},
};

const MAX_SCREENSHOT_SIZE: usize = 10 * 1024 * 1024; // 10MB

async fn authenticate_user(
    state: &AppState,
    password_md5: &str,
    username: &str,
) -> Result<User, Response> {
    let user = match repository::user::fetch_by_name(&state.db, username).await {
        Ok(Some(user)) => user,
        _ => {
            return Err(StatusCode::OK.into_response());
        },
    };

    match verify_password(password_md5, &user.pw_bcrypt).await {
        Ok(true) => Ok(user),
        _ => Err(StatusCode::OK.into_response()),
    }
}

async fn parse_typed_multipart(multipart: &mut Multipart) -> Result<ScreenshotUpload, Response> {
    let mut fields: HashMap<String, Bytes> = HashMap::new();

    while let Some(field) = multipart.next_field().await.ok().flatten() {
        let name = field.name().unwrap_or_default().to_owned();
        let content = field.bytes().await.unwrap_or_default();
        fields.insert(name, content);
    }

    match build_screenshot_upload(fields) {
        Some(upload) => Ok(upload),
        None => Err((StatusCode::BAD_REQUEST, b"error: no").into_response()),
    }
}

pub async fn upload_screenshot(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let user_agent = headers
        .get("user-agent")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if user_agent != "osu!" {
        // todo: restrict?
        //       most likely a bot
        return (StatusCode::OK, b"error: oldver").into_response();
    }

    let upload = match parse_typed_multipart(&mut multipart).await {
        Ok(u) => u,
        Err(response) => return response,
    };

    let user = match authenticate_user(&state, &upload.password_md5, &upload.username).await {
        Ok(user) => user,
        Err(response) => return response,
    };

    if upload.screenshot_data.len() > MAX_SCREENSHOT_SIZE {
        return (StatusCode::BAD_REQUEST, "file too large.").into_response();
    }

    if let Some(v) = upload.version
        && v != 1
    {
        tracing::warn!("Incorrect endpoint version v{}", v);
    }

    let file_name = save_screenshot(&state, upload.screenshot_data).await;

    let _ = state.metrics.incr("screenshot.uploaded", ["status:ok"]);

    tracing::info!("{} uploaded {}", user.name, file_name);

    file_name.into_response()
}

pub async fn get_screenshot(
    State(state): State<AppState>,
    Path(filename): Path<String>,
) -> Response {
    let Some((screenshot_id, extension)) = filename.rsplit_once('.') else {
        return (StatusCode::BAD_REQUEST, "invalid filename").into_response();
    };

    if !matches!(extension, "jpg" | "jpeg" | "png") {
        return (StatusCode::BAD_REQUEST, "extension").into_response();
    }

    let file_name = format!("{}.{}", screenshot_id, extension);
    let screenshot_path = state.storage.screenshot_file(&file_name);

    if !screenshot_path.exists() {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }

    let media_type = match extension {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        _ => "application/octet-stream",
    };

    match fs::read(&screenshot_path).await {
        Ok(contents) => (
            [
                (header::CONTENT_TYPE, media_type),
                (
                    header::CONTENT_DISPOSITION,
                    &format!("inline; filename=\"{}\"", file_name),
                ),
            ],
            contents,
        )
            .into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}
