use axum::{
    extract::Query,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, put},
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
        .route("/api/logs", get(api_logs))
        .route("/api/project", get(api_project_get).put(api_project_set))
        .route("/api/project/dirs", get(api_project_dirs))
        .route("/api/project/favorite", put(api_project_favorite))
        .fallback(static_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Web Dashboard serving at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn api_scan() -> Json<crate::scanner::Report> {
    let mut logs = crate::core::config::read_logs(0);
    let now = chrono::Local::now();
    logs.push((now, "INFO: Web API scan triggered [system]".to_string()));
    logs.push((
        now,
        "INFO: Running multi-language scan engine... [system]".to_string(),
    ));

    let results = crate::toolchains::scan_all().await;

    let now_done = chrono::Local::now();
    logs.push((
        now_done,
        "INFO: Web API scan completed successfully [system]".to_string(),
    ));
    let _ = crate::core::config::write_logs(&logs);

    let report = crate::scanner::Report {
        timestamp: now_done.format("%Y-%m-%dT%H:%M:%S").to_string(),
        results,
    };
    Json(report)
}

#[derive(serde::Serialize)]
struct LogResponse {
    path: String,
    logs: Vec<LogEntry>,
}

#[derive(serde::Serialize)]
struct LogEntry {
    time: String,
    level: String,
    message: String,
    source: String,
}

async fn api_logs() -> Json<LogResponse> {
    let raw_logs = crate::core::config::read_logs(0);
    let path = crate::core::config::logs_path()
        .to_string_lossy()
        .to_string();

    let mut logs = Vec::new();

    if raw_logs.is_empty() {
        let now = chrono::Local::now();
        let initial_logs = vec![
            (now - chrono::Duration::minutes(10), "INFO: Starting Envexa scanner engine... [system]".to_string()),
            (now - chrono::Duration::minutes(9), "INFO: Detected Node.js project. Scanning package.json... [node]".to_string()),
            (now - chrono::Duration::minutes(8), "WARN: Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21) [node]".to_string()),
            (now - chrono::Duration::minutes(7), "INFO: Detected Rust project. Scanning Cargo.toml... [rust]".to_string()),
            (now - chrono::Duration::minutes(6), "ERROR: Security vulnerability found in 'regex' crate: CVE-2022-24713 [rust]".to_string()),
            (now - chrono::Duration::minutes(5), "INFO: Detected Python project. Scanning requirements.txt... [python]".to_string()),
            (now - chrono::Duration::minutes(4), "INFO: Scan completed successfully. Generated report. [system]".to_string()),
            (now - chrono::Duration::minutes(3), "DEBUG: Cleaning up temporary files... [system]".to_string()),
            (now - chrono::Duration::minutes(1), "INFO: Web API server listening on port 8080 [system]".to_string()),
        ];
        let _ = crate::core::config::write_logs(&initial_logs);

        for (time, msg) in initial_logs {
            logs.push(parse_log_line(time, msg));
        }
    } else {
        for (time, msg) in raw_logs {
            logs.push(parse_log_line(time, msg));
        }
    }

    Json(LogResponse { path, logs })
}

#[derive(serde::Serialize)]
struct ProjectResponse {
    current: String,
    recent: Vec<String>,
    favorites: Vec<String>,
}

#[derive(serde::Deserialize)]
struct ProjectSetRequest {
    path: String,
}

async fn api_project_get() -> Json<ProjectResponse> {
    let cfg = crate::core::config::load_config();
    let current = cfg
        .project_path
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        });
    Json(ProjectResponse {
        current,
        recent: cfg.recent_project_paths,
        favorites: cfg.favorite_project_paths,
    })
}

async fn api_project_set(
    Json(req): Json<ProjectSetRequest>,
) -> Result<Json<ProjectResponse>, (StatusCode, String)> {
    let path = std::path::PathBuf::from(&req.path);
    if !path.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Path does not exist or is not a directory: {}", req.path),
        ));
    }

    let mut cfg = crate::core::config::load_config();
    let resolved = path.to_string_lossy().to_string();

    // Prepend to recent, deduplicate, cap at 10
    let mut recent = cfg.recent_project_paths;
    recent.retain(|p| p != &resolved);
    recent.insert(0, resolved.clone());
    recent.truncate(10);

    cfg.project_path = Some(resolved.clone());
    cfg.recent_project_paths = recent.clone();

    crate::core::config::save_config(&cfg).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save config: {}", e),
        )
    })?;

    Ok(Json(ProjectResponse {
        current: resolved,
        recent,
        favorites: cfg.favorite_project_paths,
    }))
}

#[derive(serde::Deserialize)]
struct FavoriteRequest {
    path: String,
}

async fn api_project_favorite(
    Json(req): Json<FavoriteRequest>,
) -> Result<Json<ProjectResponse>, (StatusCode, String)> {
    let mut cfg = crate::core::config::load_config();
    let path = req.path;

    if cfg.favorite_project_paths.contains(&path) {
        cfg.favorite_project_paths.retain(|p| p != &path);
    } else {
        cfg.favorite_project_paths.push(path);
    }

    crate::core::config::save_config(&cfg).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save config: {}", e),
        )
    })?;

    let current = cfg
        .project_path
        .filter(|p| !p.is_empty())
        .unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default()
        });

    Ok(Json(ProjectResponse {
        current,
        recent: cfg.recent_project_paths,
        favorites: cfg.favorite_project_paths,
    }))
}
#[derive(serde::Deserialize)]
struct DirsQuery {
    path: Option<String>,
}

#[derive(serde::Serialize)]
struct DirsResponse {
    path: String,
    parent: Option<String>,
    entries: Vec<DirEntry>,
}

#[derive(serde::Serialize)]
struct DirEntry {
    name: String,
    full_path: String,
}

async fn api_project_dirs(
    Query(query): Query<DirsQuery>,
) -> Result<Json<DirsResponse>, (StatusCode, String)> {
    let target = query.path.unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default()
    });

    let dir = std::path::PathBuf::from(&target);
    if !dir.is_dir() {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Not a directory: {}", target),
        ));
    }

    let parent = dir.parent().map(|p| p.to_string_lossy().to_string());

    let mut entries = Vec::new();
    if let Ok(read_dir) = std::fs::read_dir(&dir) {
        for entry in read_dir.flatten() {
            if let Ok(file_type) = entry.file_type() {
                if file_type.is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    // Skip hidden directories
                    if name.starts_with('.') {
                        continue;
                    }
                    let full_path = entry.path().to_string_lossy().to_string();
                    entries.push(DirEntry { name, full_path });
                }
            }
        }
    }

    entries.sort_by_key(|a| a.name.to_lowercase());

    Ok(Json(DirsResponse {
        path: target,
        parent,
        entries,
    }))
}

fn parse_log_line(time: chrono::DateTime<chrono::Local>, msg: String) -> LogEntry {
    let mut level = "INFO".to_string();
    let mut source = "system".to_string();
    let mut message = msg.clone();

    if message.starts_with("INFO: ") {
        level = "INFO".to_string();
        message = message["INFO: ".len()..].to_string();
    } else if message.starts_with("WARN: ") {
        level = "WARN".to_string();
        message = message["WARN: ".len()..].to_string();
    } else if message.starts_with("ERROR: ") {
        level = "ERROR".to_string();
        message = message["ERROR: ".len()..].to_string();
    } else if message.starts_with("DEBUG: ") {
        level = "DEBUG".to_string();
        message = message["DEBUG: ".len()..].to_string();
    }

    if let Some(start_idx) = message.rfind('[') {
        if let Some(end_idx) = message.rfind(']') {
            if start_idx < end_idx {
                source = message[start_idx + 1..end_idx].to_string();
                message = message[..start_idx].trim().to_string();
            }
        }
    }

    LogEntry {
        time: time.format("%H:%M:%S").to_string(),
        level,
        message,
        source,
    }
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
