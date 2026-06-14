use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};

const HTMX_MIN_JS: &str = include_str!("../static/vendor/htmx/htmx.min.js");
const JAVASCRIPT_CONTENT_TYPE: &str = "text/javascript; charset=utf-8";

pub fn asset_routes() -> Router {
    Router::new().route("/assets/htmx.min.js", get(htmx_min_js))
}

pub async fn htmx_min_js() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JAVASCRIPT_CONTENT_TYPE)
        .body(Body::from(HTMX_MIN_JS))
        .expect("static htmx response should build")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    #[tokio::test]
    async fn htmx_asset_response_has_javascript_content_type() {
        let response = htmx_min_js().await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            JAVASCRIPT_CONTENT_TYPE
        );
    }

    #[tokio::test]
    async fn htmx_asset_response_serves_vendored_body() {
        let response = htmx_min_js().await;
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("vendored htmx response body should be readable");
        let body = std::str::from_utf8(&body).expect("vendored htmx should be UTF-8");

        assert!(body.starts_with("var htmx=function()"));
        assert!(body.contains("version:\"2.0.9\""));
        assert_eq!(body, HTMX_MIN_JS);
    }

    #[test]
    fn asset_router_builds_with_htmx_route() {
        let _router = asset_routes();
    }
}
