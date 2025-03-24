use axum::{
    http::header,
    response::{Html, IntoResponse},
};
use axum_extra::response::{Css, JavaScript};

pub async fn index_handler() -> Html<&'static str> {
    Html::from(include_str!("../static/index.html"))
}

pub async fn favicon_handler() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "image/x-icon")],
        (include_bytes!("../static/favicon.ico")),
    )
}

pub async fn index_js_handler() -> JavaScript<&'static str> {
    JavaScript::from(include_str!("../static/index.js"))
}

pub async fn index_css_handler() -> Css<&'static str> {
    Css::from(include_str!("../static/index.css"))
}
