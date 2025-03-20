use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct GetUploadQuery {
    /// Decryption key for the upload.
    key: String,
}

pub async fn get_upload_handler(
    query: Query<GetUploadQuery>,
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match state.storage.get_upload(&id, &query.key) {
        Ok(bytes) => (
            [(
                header::CONTENT_TYPE,
                mime_guess::from_path(&id)
                    .first_or_octet_stream()
                    .essence_str(),
            )],
            (bytes),
        )
            .into_response(),
        Err(_) => (
            StatusCode::BAD_REQUEST,
            "This file does not exist or the decryption key is invalid.",
        )
            .into_response(),
    }
}
