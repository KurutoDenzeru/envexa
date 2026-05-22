mod cli;
mod config;
mod scanner;
mod server;
mod toolchains;
mod transport;

use scanner::ReportCache;
use server::McpServer;
use std::io::IsTerminal;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args: Vec<String> = std::env::args().collect();
    let stdin_terminal = std::io::stdin().is_terminal();

    if args.len() > 1 || stdin_terminal {
        cli::run().await
    } else {
        run_mcp().await
    }
}

async fn run_mcp() -> Result<(), anyhow::Error> {
    let cache = ReportCache::new();
    let server = Arc::new(McpServer::new(cache));

    let tools = server.tools();
    let prompts = server.prompts();
    let resources = server.resources();

    let server_clone = server.clone();
    transport::read_loop(
        tools,
        prompts,
        resources,
        |name, args| server.handle_tool(name, args),
        |name| server_clone.handle_prompt(name),
        |uri| server_clone.handle_resource(uri),
    )
    .await
}
