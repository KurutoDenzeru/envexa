mod scanner;
mod server;
mod toolchains;
mod transport;

use scanner::ReportCache;
use server::McpServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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
