mod cli;
mod config;
mod scanner;
mod toolchains;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    cli::run().await
}
