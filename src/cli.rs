use clap::Parser;

use crate::config;
use crate::scanner::{self, Report};
use crate::toolchains;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(
    name = "envexa",
    version = VERSION,
    about = "Dev environment health scanner",
    after_help = "Run with no arguments to launch the interactive TUI."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[command(about = "Full health scan (CLI output, for scripting)")]
    Scan {
        #[arg(long, default_value = "7")]
        ttl: u64,
    },
    #[command(about = "Self-update to latest release")]
    Update,
}

pub async fn run() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => run_cmd(cmd).await,
        None => {
            // This path is only reached with --help/--version flags,
            // or if no-args routes here (but main.rs routes no-args to TUI)
            Ok(())
        }
    }
}

async fn run_cmd(cmd: Commands) -> Result<(), anyhow::Error> {
    match cmd {
        Commands::Scan { ttl } => {
            let results = toolchains::scan_all().await;
            let report = Report {
                timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                results,
            };
            if let Err(e) = config::write_cache(&report, ttl) {
                eprintln!("Warning: failed to write cache: {e}");
            }
            println!("{}", scanner::format_report(&report));
        }
        Commands::Update => {
            if cfg!(debug_assertions) {
                println!("envexa development build — update checking is disabled");
                println!(
                    "Run `cargo build --release` for production and `envexa update` to update."
                );
            } else {
                self_update().await;
            }
        }
    }
    Ok(())
}

async fn self_update() {
    let tag = match fetch_latest_tag().await {
        Some(t) => t,
        None => {
            eprintln!("Failed to check for updates. Try manually: https://github.com/KurutoDenzeru/envexa/releases");
            return;
        }
    };

    let latest_ver = tag.trim_start_matches('v');
    if latest_ver == VERSION || latest_ver.is_empty() {
        println!("Already up to date (v{VERSION})");
        return;
    }

    println!("Updating from v{VERSION} to {tag}...");

    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let target = match (os, arch) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        _ => {
            eprintln!("Unsupported platform: {os}-{arch}");
            return;
        }
    };

    let ext = if os == "windows" { ".exe" } else { "" };
    let asset_name = format!("envexa-{target}{ext}");
    let download_url =
        format!("https://github.com/KurutoDenzeru/envexa/releases/download/{tag}/{asset_name}");

    let current = std::env::current_exe().unwrap_or_default();
    let tmp = current.with_extension("tmp");

    let status = std::process::Command::new(if cfg!(windows) { "powershell" } else { "curl" })
        .args(if cfg!(windows) {
            vec![
                "-OutFile".into(),
                tmp.to_string_lossy().to_string(),
                download_url.clone(),
            ]
        } else {
            vec![
                "-sLo".into(),
                tmp.to_string_lossy().to_string(),
                download_url.clone(),
            ]
        })
        .status();

    match status {
        Ok(s) if s.success() => {}
        _ => {
            eprintln!("Failed to download binary for {target}");
            eprintln!("Download manually: {download_url}");
            return;
        }
    }

    if !tmp.exists() {
        eprintln!("Download failed (no file written)");
        return;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755));
    }

    if std::fs::rename(&tmp, &current).is_err() {
        eprintln!("Failed to replace binary (try with elevated permissions)");
        let _ = std::fs::remove_file(&tmp);
        return;
    }

    println!("Updated to {tag}. Restart envexa to use the new version.");
}

async fn fetch_latest_tag() -> Option<String> {
    let url = "https://api.github.com/repos/KurutoDenzeru/envexa/releases/latest";

    let output = std::process::Command::new(if cfg!(windows) { "powershell" } else { "curl" })
        .args(if cfg!(windows) {
            vec![
                "-Command".into(),
                format!("(Invoke-WebRequest -Uri '{url}' -Headers @{{'User-Agent'='envexa'}} | ConvertFrom-Json).tag_name"),
            ]
        } else {
            vec![
                "-sL".into(),
                "-H".into(),
                "User-Agent: envexa".into(),
                url.into(),
            ]
        })
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let body = String::from_utf8_lossy(&output.stdout).to_string();

    if cfg!(windows) {
        Some(body.trim().to_string())
    } else {
        serde_json::from_str::<serde_json::Value>(&body)
            .ok()
            .and_then(|v| v["tag_name"].as_str().map(|s| s.to_string()))
    }
}
