mod app;
mod cli;
mod config;
mod scanner;
mod toolchains;
mod ui;

use std::io::IsTerminal;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        cli::run().await
    } else if std::io::stdin().is_terminal() {
        app::App::new().run().map_err(|e| anyhow::anyhow!("{e}"))
    } else {
        cli::run().await
    }
}
