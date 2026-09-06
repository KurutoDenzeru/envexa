#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub fn main() -> anyhow::Result<()> {
    // Rust-sunset deprecation notice (issue #34): the Go + Bash runtime takes
    // over after the parity gate; cargo installs stop being a release channel.
    // Quiet opt-out for scripts and tests.
    if std::env::var_os("ENVEXA_NO_DEPRECATION").is_none() {
        eprintln!(
            "Notice: the Rust runtime is being sunset. From v2.13, install via:\n  curl -fsSL https://raw.githubusercontent.com/KurutoDenzeru/envexa/main/scripts/install.sh | bash\nSee https://github.com/KurutoDenzeru/envexa/issues/34 (set ENVEXA_NO_DEPRECATION=1 to silence)."
        );
    }
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let args: Vec<String> = std::env::args().collect();

            if args.len() > 1 && args[1] != "--help" && args[1] != "-h" {
                envexa::core::cli::run().await
            } else if std::io::IsTerminal::is_terminal(&std::io::stdin()) {
                envexa::tui::app::App::new()
                    .run()
                    .await
                    .map_err(|e| anyhow::anyhow!("{e}"))
            } else {
                envexa::core::cli::run().await
            }
        })
}
