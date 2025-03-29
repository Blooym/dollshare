use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};

pub async fn delete_image_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> StatusCode {
    if !state.auth.state_for_token(authorization.token()).is_valid() {
        return StatusCode::UNAUTHORIZED;
    }

    if !state.storage.file_exists(&id).unwrap() {
        return StatusCode::NOT_FOUND;
    }

    state.storage.delete_file(&id).unwrap();
    StatusCode::OK
}
