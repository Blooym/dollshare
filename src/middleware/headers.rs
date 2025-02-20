use axum::{
    extract::Request,
    http::{HeaderValue, header},
    middleware::Next,
    response::Response,
};

pub async fn header_middleware(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response.headers_mut().append(
        header::SERVER,
        HeaderValue::from_static(env!("CARGO_PKG_NAME")),
    );
    response
        .headers_mut()
        .append("X-Robots-Tag", HeaderValue::from_static("none"));
    response
}
