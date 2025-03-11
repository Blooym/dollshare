use crate::{AppState, routes::authentication_valid};
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
    if !authentication_valid(authorization.token(), &state.tokens) {
        return StatusCode::UNAUTHORIZED;
    }

    if !state.storage.upload_exists(&id).unwrap() {
        return StatusCode::NOT_FOUND;
    }

    state.storage.delete_upload(&id).unwrap();
    StatusCode::OK
}
