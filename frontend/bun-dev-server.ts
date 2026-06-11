import { spawn } from "bun";
import { createServer } from "vite";

async function startDevServer() {
  console.log("🚀 Starting Envexa Unified Dev Server...");

  // 1. Start the Rust backend on port 8080 in the background
  console.log("🦀 Starting Rust Backend (cargo run -- serve) on port 8080...");
  const rustProcess = spawn({
    cmd: ["cargo", "run", "--", "serve"],
    cwd: "..", // Run from the envexa root directory
    stdout: "inherit",
    stderr: "inherit",
  });

  // Handle cleanup on exit
  process.on("SIGINT", () => {
    console.log("🛑 Stopping Dev Server...");
    rustProcess.kill();
    process.exit(0);
  });

  // 2. Start Vite Dev Server on port 3000
  console.log("⚡ Starting Vite Frontend on port 3000...");
  const viteServer = await createServer({
    server: {
      port: 3000,
      proxy: {
        // Proxy API requests to the Rust backend
        '/api': {
          target: 'http://localhost:8080',
          changeOrigin: true,
        },
      },
    },
  });

  await viteServer.listen();
  viteServer.printUrls();
}

startDevServer().catch((err) => {
  console.error("Failed to start dev server:", err);
  process.exit(1);
});
