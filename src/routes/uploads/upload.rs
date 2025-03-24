use crate::{AppState, cryptography::Cryptography, mime, routes::authentication_valid};
use axum::{
    Json,
    extract::{Multipart, State},
    http::StatusCode,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use mime_guess::Mime;
use serde::Serialize;
use std::str::FromStr;
use tracing::error;

#[derive(Serialize)]
pub struct CreateUploadResponse {
    url: String,
    id: String,
    key: String,
    mimetype: &'static str,
}

pub async fn create_upload_handler(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
    mut multipart: Multipart,
) -> Result<Json<CreateUploadResponse>, (StatusCode, &'static str)> {
    if !authentication_valid(authorization.token(), &state.auth_tokens) {
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
    // uploads that are not images or videos if enabled.
    let Some(infer) = infer::get(&data) else {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Your file was rejected because the MIME type could not be determined.",
        ));
    };
    if !mime::is_mime_allowed(
        &Mime::from_str(infer.mime_type()).unwrap(),
        &state.upload_allowed_mimetypes,
    ) {
        return Err((
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Your file was rejected because uploading file of this type is not permitted.",
        ));
    }

    // Store file by hash to prevent duplicating uploads.
    let filename = format!(
        "{}.{}",
        Cryptography::hash_from_bytes(&data, &state.persisted_salt)
            .unwrap()
            .get(..10)
            .unwrap(),
        infer.extension()
    );

    match state.storage.store_upload(&filename, &data) {
        Ok(decryption_key) => Ok(Json(CreateUploadResponse {
            mimetype: infer.mime_type(),
            url: format!(
                "{}://{}/uploads/{}?key={}",
                state.public_base_url.scheme(),
                state.public_base_url.port().map_or(
                    state.public_base_url.host_str().unwrap().to_string(),
                    |f| format!("{}:{}", state.public_base_url.host_str().unwrap(), f,)
                ),
                filename,
                decryption_key
            ),
            key: decryption_key,
            id: filename,
        })),
        Err(err) => {
            error!("Error while encrypting or writing file {filename}: {err:?}");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Your file could not be encrypted/written to storage successfully.",
            ))
        }
    }
}
