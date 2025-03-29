use crate::AppState;
use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct StatisticsResponse {
    storage: FilesInfo,
}

#[derive(Serialize)]
pub struct FilesInfo {
    files: usize,
}

pub async fn statistics_handler(
    State(state): State<AppState>,
    TypedHeader(authorization): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
    if !state.auth.state_for_token(authorization.token()).is_valid() {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(StatisticsResponse {
        storage: FilesInfo {
            files: state.storage.file_count().unwrap_or(0),
        },
    })
    .into_response()
}
