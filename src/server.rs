use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;

#[derive(RustEmbed)]
#[folder = "frontend/dist/"]
struct Asset;

pub async fn start(port: u16) {
    let app = Router::new()
        .route("/api/scan", get(api_scan))
        .fallback(static_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Web Dashboard serving at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_scan() -> Json<crate::scanner::Report> {
    let results = crate::toolchains::scan_all().await;
    let report = crate::scanner::Report {
        timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        results,
    };
    Json(report)
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();

    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(path.as_str()) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if let Some(index) = Asset::get("index.html") {
                ([(header::CONTENT_TYPE, "text/html")], index.data).into_response()
            } else {
                (StatusCode::NOT_FOUND, "404 Not Found").into_response()
            }
        }
    }
}
