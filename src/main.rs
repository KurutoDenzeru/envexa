#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

pub fn main() -> anyhow::Result<()> {
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
