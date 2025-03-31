use crate::AppState;
use axum::{Json, extract::State, response::IntoResponse};
use serde::Serialize;

#[derive(Serialize)]
pub struct StatisticsResponse {
    storage: FilesInfo,
}

#[derive(Serialize)]
pub struct FilesInfo {
    files: usize,
}

pub async fn statistics_handler(State(state): State<AppState>) -> impl IntoResponse {
    Json(StatisticsResponse {
        storage: FilesInfo {
            files: state.storage_provider.file_count().unwrap_or(0),
        },
    })
    .into_response()
}
