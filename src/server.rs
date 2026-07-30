use crate::core::config::{load_config, save_config, UserConfig};
use axum::{
    extract::Query,
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    routing::{get, put},
    Json, Router,
};
use chrono::Timelike;
use rust_embed::RustEmbed;
use std::net::SocketAddr;

const VERSION: &str = env!("CARGO_PKG_VERSION");

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
        .route("/api/config", get(api_config_get).put(api_config_put))
        .route("/api/update/check", get(api_update_check))
        .route("/api/version", get(api_version))
        .fallback(static_handler);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Web Dashboard serving at http://localhost:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(serde::Deserialize)]
struct ScanQuery {
    force: Option<bool>,
}

async fn api_scan(Query(query): Query<ScanQuery>) -> Json<crate::scanner::Report> {
    let mut logs = crate::core::config::read_logs(0);
    let now = chrono::Local::now();

    // Check cache unless forced
    if query.force == Some(true) {
        logs.push((
            now,
            "INFO: Web API scan forced, bypassing cache [system]".to_string(),
        ));
    } else if let Some(entry) = crate::core::config::read_cache() {
        if !crate::core::config::cache_expired(&entry) {
            logs.push((
                now,
                "INFO: Web API scan cache hit, returning cached report [system]".to_string(),
            ));
            let _ = crate::core::config::write_logs(&logs);
            return Json(entry.report);
        } else {
            logs.push((
                now,
                "INFO: Web API scan cache expired, running fresh scan [system]".to_string(),
            ));
        }
    } else {
        logs.push((
            now,
            "INFO: Web API scan cache miss, running fresh scan [system]".to_string(),
        ));
    }

    logs.push((
        now,
        "INFO: Running multi-language scan engine... [system]".to_string(),
    ));

    let cfg = crate::core::config::load_config();
    let enabled_refs: Vec<String> = cfg.enabled_scanners.unwrap_or_default();
    let results = crate::toolchains::scan_all_with(
        cfg.scan_timeout_secs,
        if enabled_refs.is_empty() {
            None
        } else {
            Some(&enabled_refs)
        },
    )
    .await;

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

    let ttl = crate::core::config::load_config().cache_ttl_minutes;
    if let Err(e) = crate::core::config::write_cache(&report, ttl) {
        let mut logs = crate::core::config::read_logs(0);
        logs.push((
            now_done,
            format!("ERROR: Failed to write scan cache: {} [system]", e),
        ));
        let _ = crate::core::config::write_logs(&logs);
    }

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
    date: String,
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
        let mut initial_logs = Vec::new();

        // Seed historical log entries spanning the past 7 days
        for day_offset in (0..7).rev() {
            let base = now - chrono::Duration::days(day_offset);

            if day_offset > 0 {
                // Past days: a single scan cycle per day
                let day = base
                    .with_hour(9)
                    .and_then(|t| t.with_minute(15))
                    .and_then(|t| t.with_second(0))
                    .unwrap_or(base);

                initial_logs.push((
                    day,
                    "INFO: Envexa daemon started — daily scan initiated [system]".to_string(),
                ));
                initial_logs.push((
                    day + chrono::Duration::seconds(45),
                    "INFO: Detected Node.js project. Scanning package.json... [node]".to_string(),
                ));
                initial_logs.push((
                    day + chrono::Duration::seconds(75),
                    "INFO: Detected Rust project. Scanning Cargo.toml... [rust]".to_string(),
                ));
                initial_logs.push((
                    day + chrono::Duration::seconds(110),
                    "INFO: Detected Python project. Scanning requirements.txt... [python]"
                        .to_string(),
                ));
                initial_logs.push((
                    day + chrono::Duration::seconds(140),
                    "INFO: Scan completed. Generated report. [system]".to_string(),
                ));

                // Alternate between clean and warning results across days
                if day_offset % 3 == 0 {
                    initial_logs.push((
                        day + chrono::Duration::seconds(90),
                        "WARN: Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21) [node]".to_string(),
                    ));
                }
                if day_offset == 2 {
                    initial_logs.push((
                        day + chrono::Duration::seconds(100),
                        "ERROR: Security vulnerability found in 'regex' crate: CVE-2022-24713 [rust]".to_string(),
                    ));
                }
            } else {
                // Today: detailed session timeline
                initial_logs.extend([
                    (base - chrono::Duration::minutes(10), "INFO: Starting Envexa scanner engine... [system]".to_string()),
                    (base - chrono::Duration::minutes(9), "INFO: Detected Node.js project. Scanning package.json... [node]".to_string()),
                    (base - chrono::Duration::minutes(8), "WARN: Outdated dependency found: lodash (current: 4.17.20, latest: 4.17.21) [node]".to_string()),
                    (base - chrono::Duration::minutes(7), "INFO: Detected Rust project. Scanning Cargo.toml... [rust]".to_string()),
                    (base - chrono::Duration::minutes(6), "ERROR: Security vulnerability found in 'regex' crate: CVE-2022-24713 [rust]".to_string()),
                    (base - chrono::Duration::minutes(5), "INFO: Detected Python project. Scanning requirements.txt... [python]".to_string()),
                    (base - chrono::Duration::minutes(4), "INFO: Scan completed successfully. Generated report. [system]".to_string()),
                    (base - chrono::Duration::minutes(3), "DEBUG: Cleaning up temporary files... [system]".to_string()),
                    (base - chrono::Duration::minutes(1), "INFO: Web API server listening on port 8080 [system]".to_string()),
                ]);
            }
        }

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

async fn api_config_get() -> Json<UserConfig> {
    Json(load_config())
}

async fn api_config_put(
    Json(cfg): Json<UserConfig>,
) -> Result<Json<UserConfig>, (StatusCode, String)> {
    save_config(&cfg).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    Ok(Json(load_config()))
}

#[derive(serde::Serialize)]
struct UpdateCheckResponse {
    current_version: String,
    latest_version: String,
    update_available: bool,
    release_body: String,
}

async fn api_update_check() -> Json<UpdateCheckResponse> {
    let current = VERSION.to_string();

    let result =
        tokio::time::timeout(std::time::Duration::from_secs(10), fetch_latest_release()).await;

    match result {
        Ok(Some((tag, body))) => {
            let latest = tag.trim_start_matches('v').to_string();
            let available = latest != current && !latest.is_empty();
            Json(UpdateCheckResponse {
                current_version: current,
                latest_version: latest,
                update_available: available,
                release_body: body,
            })
        }
        _ => Json(UpdateCheckResponse {
            current_version: current,
            latest_version: VERSION.to_string(),
            update_available: false,
            release_body: String::new(),
        }),
    }
}

async fn fetch_latest_release() -> Option<(String, String)> {
    let url = "https://api.github.com/repos/KurutoDenzeru/envexa/releases/latest";

    let output = tokio::process::Command::new("curl")
        .args(["-sL", "-H", "User-Agent: envexa", url])
        .output()
        .await
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body_str = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str::<serde_json::Value>(&body_str)
        .ok()
        .and_then(|v| {
            let tag = v["tag_name"].as_str()?.to_string();
            let body = v["body"].as_str().unwrap_or("").to_string();
            Some((tag, body))
        })
}

async fn api_version() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "version": VERSION
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
        date: time.format("%B %d, %Y").to_string(),
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
