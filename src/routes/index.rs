use axum::response::Html;

pub async fn index_handler() -> Html<&'static str> {
    Html::from(include_str!("../static/index.html"))
}
