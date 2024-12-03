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
use infer::MatcherType;
use serde::Serialize;

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
    let field = multipart
        .next_field()
        .await
        .context("Failed to get field")
        .unwrap()
        .context("Field data was None")
        .unwrap();
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
    if state.limit_to_media {
        if infer.matcher_type() != MatcherType::Image && infer.matcher_type() != MatcherType::Video
        {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Your file was rejected because the MIME type is not 'image/*' or 'video/*'.",
            ));
        }
    }

    // Store file by its sha256 hash to prevent duplicate uploads.
    let id = format!("{}.{}", sha256::digest(&*data), infer.extension());
    state.storage.store_upload(&id, &data).unwrap();
    Ok(Json(CreateUploadResponse {
        url: format!(
            "{}://{}/uploads/{}",
            state.public_url.scheme(),
            state.public_url.port().map_or(
                state.public_url.host_str().unwrap().to_string(),
                |f| format!("{}:{}", state.public_url.host_str().unwrap(), f)
            ),
            id
        ),
    }))
}
