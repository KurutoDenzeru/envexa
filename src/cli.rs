use clap::CommandFactory;
use clap::Parser;

use crate::config;
use crate::scanner::{self, Report};
use crate::toolchains;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "envexa", version = VERSION, about = "Dev environment health scanner")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    #[command(about = "Full health scan of all or one toolchain")]
    Scan {
        chain: Option<String>,
        #[arg(long, default_value = "7")]
        ttl: u64,
    },
    #[command(about = "Quick dashboard summary")]
    Status,
    #[command(about = "Check outdated packages only")]
    Outdated { chain: Option<String> },
    #[command(about = "Show the latest cached report")]
    Report,
    #[command(about = "Upgrade a toolchain (pip currently supported)")]
    Upgrade { tool: String },
    #[command(about = "Self-update to latest release")]
    Update,
    #[command(about = "Show version and system info")]
    Info,
    #[command(about = "Remove cache and config")]
    Uninstall,
}

pub async fn run() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();
    match cli.command {
        Some(cmd) => run_cmd(cmd).await,
        None => {
            Cli::command().print_help()?;
            Ok(())
        }
    }
}

async fn run_cmd(cmd: Commands) -> Result<(), anyhow::Error> {
    match cmd {
        Commands::Scan { chain, ttl } => {
            let chain = chain.as_deref().unwrap_or("all");
            let results = if chain == "all" {
                toolchains::scan_all().await
            } else if let Some(res) = toolchains::scan_one(chain).await {
                let mut map = std::collections::HashMap::new();
                map.insert(chain.to_string(), res);
                map
            } else {
                eprintln!("Unknown chain: {chain}");
                std::process::exit(1);
            };
            let report = Report {
                timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                results,
            };
            if let Err(e) = config::write_cache(&report, ttl) {
                eprintln!("Warning: failed to write cache: {e}");
            }
            println!("{}", format_report(&report));
        }
        Commands::Status => {
            let text = if let Some(entry) = config::read_cache() {
                if config::cache_expired(&entry) {
                    let r = fresh_scan().await;
                    println!("{}", scanner::format_status(&r));
                    return Ok(());
                }
                scanner::format_status(&entry.report)
            } else {
                let r = fresh_scan().await;
                println!("{}", scanner::format_status(&r));
                return Ok(());
            };
            println!("{text}");
        }
        Commands::Outdated { chain } => {
            let chain = chain.as_deref().unwrap_or("all");
            let results = if chain == "all" {
                toolchains::scan_all().await
            } else if let Some(res) = toolchains::scan_one(chain).await {
                let mut map = std::collections::HashMap::new();
                map.insert(chain.to_string(), res);
                map
            } else {
                eprintln!("Unknown chain: {chain}");
                std::process::exit(1);
            };
            let report = Report {
                timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                results,
            };
            println!("{}", scanner::format_outdated(&report));
        }
        Commands::Report => match config::read_cache() {
            Some(entry) => {
                if config::cache_expired(&entry) {
                    eprintln!("Cache expired (cached {})", entry.cached_at);
                }
                println!("{}", format_report(&entry.report));
            }
            None => {
                eprintln!("No cached report. Run `envexa scan` first.");
                std::process::exit(1);
            }
        },
        Commands::Upgrade { tool } => {
            upgrade_tool(&tool).await;
        }
        Commands::Update => {
            self_update().await;
        }
        Commands::Info => {
            print_info();
        }
        Commands::Uninstall => {
            uninstall();
        }
    }
    Ok(())
}

async fn fresh_scan() -> Report {
    let results = toolchains::scan_all().await;
    Report {
        timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        results,
    }
}

fn format_report(report: &Report) -> String {
    scanner::format_report(report)
}

async fn upgrade_tool(tool: &str) {
    match tool {
        "pip" => {
            let output = std::process::Command::new("pip3")
                .args(["install", "--upgrade", "pip"])
                .output();
            match output {
                Ok(o) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    println!("pip upgraded successfully.\n{stdout}");
                }
                Ok(o) => {
                    let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                    eprintln!("Upgrade failed:\n{stderr}");
                }
                Err(e) => {
                    eprintln!("Failed to execute pip3: {e}");
                }
            }
        }
        _ => {
            eprintln!("Upgrade not implemented for `{tool}`. Supported: pip");
        }
    }
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

fn print_info() {
    let exe = std::env::current_exe().unwrap_or_default();
    let exe_size = std::fs::metadata(&exe).map(|m| m.len()).unwrap_or(0);
    let exe_str = exe.to_string_lossy().to_string();
    let cache_info = match config::read_cache() {
        Some(entry) => {
            let expired = config::cache_expired(&entry);
            let status = if expired { "expired" } else { "fresh" };
            format!(
                "{} ({}, cached {}, TTL: {}d)",
                cache_path_display(),
                status,
                &entry.cached_at[..10],
                entry.ttl_days
            )
        }
        None => "none".to_string(),
    };

    let config_path = config_path_display();
    let config_exists = std::path::Path::new(&config_path).exists();

    println!("Envexa v{} — Dev Environment Scanner", VERSION);
    println!("Binary:   {} ({})", exe_str, human_size(exe_size));
    println!("Cache:    {}", cache_info);
    println!(
        "Config:   {} ({})",
        config_path,
        if config_exists { "present" } else { "not set" }
    );
    println!("Source:   https://github.com/KurutoDenzeru/envexa");
    println!("Chains:   brew, npm, pnpm, yarn, bun, deno, pip, gem, cargo, docker");
}

fn cache_path_display() -> String {
    let d = config::dir();
    d.join("cache.json").to_string_lossy().to_string()
}

fn config_path_display() -> String {
    let d = config::dir();
    d.join("config.json").to_string_lossy().to_string()
}

fn human_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

fn uninstall() {
    let exe = std::env::current_exe().unwrap_or_default();
    let cache = config::dir();

    println!("This will remove:");
    println!("  Binary: {}", exe.display());
    println!("  Cache:  {}", cache.display());
    print!("Are you sure? [y/N] ");
    use std::io::Write;
    std::io::stdout().flush().ok();
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).ok();
    if input.trim().eq_ignore_ascii_case("y") {
        let _ = config::remove_all();
        if exe.exists() {
            #[cfg(unix)]
            {
                if std::fs::remove_file(&exe).is_err() {
                    eprintln!(
                        "Could not remove binary at {}. Remove it manually.",
                        exe.display()
                    );
                }
            }
            #[cfg(windows)]
            {
                eprintln!("Binary at {}. Remove it manually.", exe.display());
            }
        }
        println!("Envexa has been removed.");
    } else {
        println!("Cancelled.");
    }
}
