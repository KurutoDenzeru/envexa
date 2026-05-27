pub mod core;
mod scanner;
mod toolchains;
mod tui;

use std::io::IsTerminal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] != "--help" && args[1] != "-h" && args[1] != "-V" && args[1] != "--version" {
        crate::core::cli::run().await
    } else if std::io::stdin().is_terminal() {
        tui::app::App::new()
            .run()
            .map_err(|e| anyhow::anyhow!("{e}"))
    } else {
        crate::core::cli::run().await
    }
}
