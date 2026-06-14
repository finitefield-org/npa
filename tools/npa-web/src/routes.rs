use axum::{
    body::Body,
    http::{header, StatusCode},
    response::Response,
    routing::get,
    Router,
};

const HTMX_MIN_JS: &str = include_str!("../static/vendor/htmx/htmx.min.js");
const CSS_CONTENT_TYPE: &str = "text/css; charset=utf-8";
const JAVASCRIPT_CONTENT_TYPE: &str = "text/javascript; charset=utf-8";

pub fn asset_routes() -> Router {
    Router::new()
        .route("/assets/htmx.min.js", get(htmx_min_js))
        .route("/assets/app.css", get(app_css))
}

pub async fn htmx_min_js() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, JAVASCRIPT_CONTENT_TYPE)
        .body(Body::from(HTMX_MIN_JS))
        .expect("static htmx response should build")
}

pub async fn app_css() -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, CSS_CONTENT_TYPE)
        .body(Body::from(crate::style::app_css()))
        .expect("static app css response should build")
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

    #[tokio::test]
    async fn app_css_response_has_css_content_type() {
        let response = app_css().await;

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            CSS_CONTENT_TYPE
        );
    }

    #[tokio::test]
    async fn app_css_response_serves_generated_body() {
        let response = app_css().await;
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("generated app css response body should be readable");
        let body = std::str::from_utf8(&body).expect("generated app css should be UTF-8");

        assert!(body.contains(".npa-theme"));
        assert!(body.contains(".lg\\:grid-cols-2"));
        assert_eq!(body, crate::style::app_css());
    }

    #[test]
    fn asset_router_builds_with_static_routes() {
        let _router = asset_routes();
    }
}
