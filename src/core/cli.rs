use clap::Parser;
use std::io::Write;
use std::time::Instant;

use crate::core::config;
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
        #[arg(long, default_value = "markdown")]
        format: String,
    },
    #[command(about = "Self-update to latest release")]
    Update,
    #[command(about = "Run in background to periodically scan and notify")]
    Daemon {
        #[arg(long, default_value = "14400")]
        interval: u64,
    },
}

pub async fn with_spinner<F, T>(label: &str, future: F) -> T
where
    F: std::future::Future<Output = T>,
{
    let label = label.to_string();
    let (tx, mut rx) = tokio::sync::oneshot::channel::<()>();

    let spinner_task = tokio::spawn(async move {
        let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut idx = 0;
        let start = Instant::now();
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(80));

        loop {
            tokio::select! {
                _ = &mut rx => break,
                _ = interval.tick() => {
                    let elapsed = start.elapsed().as_secs_f64();
                    eprint!("\r\x1b[2K\x1b[36m{}\x1b[0m {} ({:.1}s)", chars[idx], label, elapsed);
                    let _ = std::io::stderr().flush();
                    idx = (idx + 1) % chars.len();
                }
            }
        }
        eprint!("\r\x1b[2K");
        let _ = std::io::stderr().flush();
    });

    let res = future.await;
    let _ = tx.send(());
    let _ = spinner_task.await;
    res
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
        Commands::Scan { ttl, format } => {
            let results = with_spinner("Scanning toolchains...", toolchains::scan_all()).await;
            let report = Report {
                timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                results,
            };
            if let Err(e) = config::write_cache(&report, ttl) {
                eprintln!("Warning: failed to write cache: {e}");
            }
            if format.to_lowercase() == "json" {
                match serde_json::to_string_pretty(&report) {
                    Ok(json) => println!("{}", json),
                    Err(e) => eprintln!("Error formatting JSON: {}", e),
                }
            } else if format.to_lowercase() == "sarif" {
                println!("{}", scanner::format_sarif(&report));
            } else {
                println!("{}", scanner::format_report(&report));
            }
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
        Commands::Daemon { interval } => {
            run_daemon(interval).await;
        }
    }
    Ok(())
}

async fn run_daemon(interval: u64) {
    println!("Starting envexa daemon. Interval: {}s", interval);
    loop {
        let results = toolchains::scan_all().await;

        let mut warnings = 0;
        let mut errors = 0;
        let mut outdated = 0;

        for res in results.values() {
            match res.status.as_str() {
                "warning" => warnings += 1,
                "error" => errors += 1,
                _ => {}
            }
            outdated += scanner::extract_outdated(res).len();
        }

        if warnings > 0 || errors > 0 || outdated > 0 {
            let mut msgs = vec![];
            if errors > 0 {
                msgs.push(format!("{} errors", errors));
            }
            if warnings > 0 {
                msgs.push(format!("{} warnings", warnings));
            }
            if outdated > 0 {
                msgs.push(format!("{} outdated pkgs", outdated));
            }

            let msg = msgs.join(", ");
            let title = "Envexa Health Alert";

            #[cfg(target_os = "macos")]
            {
                let script = format!("display notification \"{}\" with title \"{}\"", msg, title);
                let _ = std::process::Command::new("osascript")
                    .args(["-e", &script])
                    .status();
            }

            #[cfg(target_os = "linux")]
            {
                let _ = std::process::Command::new("notify-send")
                    .args([title, &msg])
                    .status();
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(interval)).await;
    }
}

async fn self_update() {
    let tag = match with_spinner("Checking for updates...", fetch_latest_tag()).await {
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
    match (os, arch) {
        ("macos", "aarch64")
        | ("macos", "x86_64")
        | ("linux", "aarch64")
        | ("linux", "x86_64")
        | ("windows", "x86_64") => {}
        _ => {
            eprintln!("Unsupported platform: {os}-{arch}");
            return;
        }
    };

    let ext = if os == "windows" { ".exe" } else { "" };
    let asset_name = format!("envexa-{arch}-{os}{ext}");
    let download_url =
        format!("https://github.com/KurutoDenzeru/envexa/releases/download/{tag}/{asset_name}");

    let current = std::env::current_exe().unwrap_or_default();
    let tmp = std::env::temp_dir().join(format!("{asset_name}.tmp"));

    let cmd_url = download_url.clone();
    let cmd_tmp = tmp.clone();
    let status = with_spinner(
        "Downloading latest release...",
        tokio::task::spawn_blocking(move || {
            std::process::Command::new(if cfg!(windows) { "powershell" } else { "curl" })
                .args(if cfg!(windows) {
                    vec![
                        "-OutFile".into(),
                        cmd_tmp.to_string_lossy().to_string(),
                        cmd_url,
                    ]
                } else {
                    vec![
                        "-fsLo".into(),
                        cmd_tmp.to_string_lossy().to_string(),
                        cmd_url,
                    ]
                })
                .status()
        }),
    )
    .await;

    let download_success = matches!(status, Ok(Ok(s)) if s.success());
    if !download_success {
        eprintln!("Failed to download binary {asset_name}");
        eprintln!("Download manually: {download_url}");
        return;
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

    #[cfg(unix)]
    {
        let file_out = std::process::Command::new("file")
            .arg(&tmp)
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok()
                } else {
                    None
                }
            });
        match file_out {
            Some(s) if s.contains("Mach-O") || s.contains("ELF") => {}
            _ => {
                let _ = std::fs::remove_file(&tmp);
                eprintln!("Downloaded file is not a valid binary (corrupted or wrong URL)");
                eprintln!("Download manually: {download_url}");
                return;
            }
        }
    }

    if std::fs::rename(&tmp, &current).is_err() && std::fs::copy(&tmp, &current).is_err() {
        eprintln!("Failed to replace binary (try with elevated permissions or sudo)");
        let _ = std::fs::remove_file(&tmp);
        return;
    }
    let _ = std::fs::remove_file(&tmp);

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
