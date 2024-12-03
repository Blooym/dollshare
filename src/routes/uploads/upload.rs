use crate::{routes::authentication_valid, AppState};
use anyhow::Context;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
    Json,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use blake3::Hasher;
use infer::MatcherType;
use serde::Serialize;
use std::io::{self, BufReader};

#[derive(Serialize)]
pub struct CreateUploadResponse {
    url: String,
}

pub async fn create_upload_response(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> Result<Json<CreateUploadResponse>, (StatusCode, &'static str)> {
    if !authentication_valid(authorization.token(), &state.token) {
        return Err((StatusCode::UNAUTHORIZED, StatusCode::UNAUTHORIZED.as_str()));
    }

    // Get data from first multipart upload.
    let field = match multipart.next_field().await {
        Ok(field) => {
            let Some(field) = field else {
                return Err((StatusCode::BAD_REQUEST, "Multipart field name was not set"));
            };
            field
        }
        Err(_) => return Err((StatusCode::BAD_REQUEST, "Multipart field error")),
    };

    let Ok(data) = field.bytes().await else {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Upload is too big to be processed by this server.",
        ));
    };

    // Infer mimetype by magic numbers and reject
    // mimetypes that arent images or videos.
    let Some(infer) = infer::get(&data) else {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Your file was rejected because the MIME type could not be determined.",
        ));
    };
    if state.limit_to_media
        && infer.matcher_type() != MatcherType::Image
        && infer.matcher_type() != MatcherType::Video
    {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Your file was rejected because the MIME type is not 'image/*' or 'video/*'.",
        ));
    }

    // Store file by its hash to prevent duplicate uploads.
    let mut hasher = Hasher::new();
    io::copy(&mut BufReader::new(&*data), &mut hasher)
        .context("failed to copy file data into hasher")
        .unwrap();
    let file_id = format!(
        "{}.{}",
        &hex::encode(hasher.finalize().as_bytes())[..12],
        infer.extension()
    );

    state.storage.store_upload(&file_id, &data).unwrap();
    Ok(Json(CreateUploadResponse {
        url: format!(
            "{}://{}/uploads/{}",
            state.public_url.scheme(),
            state.public_url.port().map_or(
                state.public_url.host_str().unwrap().to_string(),
                |f| format!("{}:{}", state.public_url.host_str().unwrap(), f)
            ),
            file_id
        ),
    }))
}
