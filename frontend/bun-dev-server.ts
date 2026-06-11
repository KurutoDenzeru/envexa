import { spawn } from "bun";
// import { createServer } from "vite"; // Not needed, running via CLI

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

  // 2. Start Vite in Watch Mode
  console.log("⚡ Compiling Vite Frontend (watch mode)...");
  const viteProcess = spawn({
    cmd: ["bun", "run", "vite", "build", "--watch"],
    cwd: import.meta.dir,
    stdout: "inherit",
    stderr: "inherit",
  });

  // Handle cleanup on exit
  process.on("SIGINT", () => {
    console.log("🛑 Stopping Dev Server...");
    rustProcess.kill();
    viteProcess.kill();
    process.exit(0);
  });
}

startDevServer().catch((err) => {
  console.error("Failed to start dev server:", err);
  process.exit(1);
});
