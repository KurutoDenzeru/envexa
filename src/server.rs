use serde_json::Value;
use std::collections::HashMap;

use crate::cli;
use crate::scanner::{self, ReportCache};
use crate::toolchains;
use crate::transport::*;

fn str_val(v: Option<Value>, key: &str) -> String {
    v.as_ref()
        .and_then(|p| p.get(key))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string()
}

pub struct McpServer {
    pub cache: ReportCache,
}

impl McpServer {
    pub fn new(cache: ReportCache) -> Self {
        Self { cache }
    }

    pub fn tools(&self) -> Vec<ToolDescription> {
        vec![
            ToolDescription {
                name: "envexa_scan".into(),
                description: "Envexa — scan dev environment toolchains. chain: all|brew|npm|pnpm|yarn|bun|deno|pip|gem|cargo|docker".into(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "chain": {
                            "type": "string",
                            "default": "all",
                            "description": "Toolchain to scan"
                        }
                    }
                })),
            },
            ToolDescription {
                name: "envexa_check_outdated".into(),
                description: "Envexa — check for outdated packages. chain: all|brew|npm|pnpm|yarn|bun|deno|pip|gem|cargo|docker".into(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "chain": {
                            "type": "string",
                            "default": "all"
                        }
                    }
                })),
            },
            ToolDescription {
                name: "envexa_check_mismatches".into(),
                description: "Envexa — detect version mismatches of the same package across different projects".into(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "projects": {
                            "type": "array",
                            "items": { "type": "string" }
                        }
                    }
                })),
            },
            ToolDescription {
                name: "envexa_find_unused".into(),
                description: "Envexa — find unused dependencies in a project directory".into(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "project": {
                            "type": "string"
                        }
                    },
                    "required": ["project"]
                })),
            },
            ToolDescription {
                name: "envexa_get_report".into(),
                description: "Envexa — get the latest dev environment health report".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_brew_status".into(),
                description: "Envexa — scan only Homebrew (formulae + casks)".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_npm_status".into(),
                description: "Envexa — scan only npm/Node.js".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_pnpm_status".into(),
                description: "Envexa — scan only pnpm".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_yarn_status".into(),
                description: "Envexa — scan only Yarn".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_bun_status".into(),
                description: "Envexa — scan only Bun".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_deno_status".into(),
                description: "Envexa — scan only Deno".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_pip_status".into(),
                description: "Envexa — scan only Python/pip".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_pip_upgrade".into(),
                description: "Envexa — upgrade pip to the latest version".into(),
                input_schema: Some(serde_json::json!({ "type": "object", "properties": {} })),
            },
            ToolDescription {
                name: "envexa_cmd".into(),
                description: "Envexa — execute a preset slash command. Use when the user types /scan, /outdated, /status, /upgrade, /report, or /help in chat.".into(),
                input_schema: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "Slash command to execute"
                        }
                    },
                    "required": ["command"]
                })),
            },
        ]
    }

    pub fn prompts(&self) -> Vec<PromptDescription> {
        vec![
            PromptDescription {
                name: "envexa:scan".into(),
                description: "Envexa — full health scan of dev environment toolchains".into(),
            },
            PromptDescription {
                name: "envexa:status".into(),
                description: "Envexa — quick dashboard overview of all toolchains".into(),
            },
            PromptDescription {
                name: "envexa:outdated".into(),
                description: "Envexa — list outdated packages across all toolchains".into(),
            },
        ]
    }

    pub fn resources(&self) -> Vec<ResourceDescription> {
        vec![
            ResourceDescription {
                uri: "envexa://report".into(),
                name: "Envexa Health Report".into(),
                description: "Latest dev environment health report as markdown".into(),
                mime_type: "text/markdown".into(),
            },
            ResourceDescription {
                uri: "envexa://report/raw".into(),
                name: "Envexa Health Report (Raw)".into(),
                description: "Latest dev environment health report as raw JSON".into(),
                mime_type: "application/json".into(),
            },
        ]
    }

    pub fn handle_tool(&self, name: &str, args: Option<Value>) -> Result<Value, String> {
        match name {
            "envexa_scan" => {
                let chain = str_val(args, "chain");
                let chain = if chain.is_empty() { "all" } else { &chain };
                Ok(Value::String(tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current()
                        .block_on(async { scanner::scan_and_cache(&self.cache, chain).await })
                })))
            }
            "envexa_check_outdated" => {
                let chain = str_val(args, "chain");
                let chain = if chain.is_empty() { "all" } else { &chain };
                Ok(Value::String(tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        let results = if chain == "all" {
                            toolchains::scan_all().await
                        } else if let Some(res) = toolchains::scan_one(chain).await {
                            let mut map = HashMap::new();
                            map.insert(chain.to_string(), res);
                            map
                        } else {
                            return format!("Unknown chain: {chain}");
                        };
                        let report = scanner::Report {
                            timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                            results,
                        };
                        scanner::format_outdated(&report)
                    })
                })))
            }
            "envexa_check_mismatches" => Ok(Value::String(
                "Version mismatch detection not yet implemented in Rust".into(),
            )),
            "envexa_find_unused" => {
                let project = str_val(args, "project");
                Ok(Value::String(format!("Unused dependency analysis not yet implemented in Rust. Check the Python version for project: {project}")))
            }
            "envexa_get_report" => {
                let text = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        match self.cache.get().await {
                            Some(report) => scanner::format_report(&report),
                            None => "No report available. Run `scan` first.".into(),
                        }
                    })
                });
                Ok(Value::String(text))
            }
            "envexa_brew_status" => self.scan_single("brew"),
            "envexa_npm_status" => self.scan_single("npm"),
            "envexa_pnpm_status" => self.scan_single("pnpm"),
            "envexa_yarn_status" => self.scan_single("yarn"),
            "envexa_bun_status" => self.scan_single("bun"),
            "envexa_deno_status" => self.scan_single("deno"),
            "envexa_pip_status" => self.scan_single("pip"),
            "envexa_pip_upgrade" => Ok(Value::String(cli::execute("/upgrade pip"))),
            "envexa_cmd" => {
                let cmd = str_val(args, "command");
                Ok(Value::String(cli::execute(&cmd)))
            }
            _ => Err(format!("Unknown tool: {name}")),
        }
    }

    fn scan_single(&self, chain: &str) -> Result<Value, String> {
        Ok(Value::String(tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { scanner::scan_and_cache(&self.cache, chain).await })
        })))
    }

    pub fn handle_prompt(&self, name: &str) -> Result<Value, String> {
        let text = match name {
            "envexa:scan" => tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current()
                    .block_on(async { scanner::scan_and_cache(&self.cache, "all").await })
            }),
            "envexa:status" => tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let results = toolchains::scan_all().await;
                    let report = scanner::Report {
                        timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                        results,
                    };
                    scanner::format_status(&report)
                })
            }),
            "envexa:outdated" => tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(async {
                    let results = toolchains::scan_all().await;
                    let report = scanner::Report {
                        timestamp: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
                        results,
                    };
                    scanner::format_outdated(&report)
                })
            }),
            _ => return Err(format!("Unknown prompt: {name}")),
        };
        Ok(Value::String(text))
    }

    pub fn handle_resource(&self, uri: &str) -> Result<Value, String> {
        match uri {
            "envexa://report" | "envexa://report/raw" => {
                let text = tokio::task::block_in_place(|| {
                    tokio::runtime::Handle::current().block_on(async {
                        match self.cache.get().await {
                            Some(report) => {
                                if uri.ends_with("/raw") {
                                    serde_json::to_string_pretty(&report)
                                        .unwrap_or_else(|_| "{}".into())
                                } else {
                                    scanner::format_report(&report)
                                }
                            }
                            None => "No report available. Run `scan` first.".into(),
                        }
                    })
                });
                Ok(Value::String(text))
            }
            _ => Err(format!("Unknown resource: {uri}")),
        }
    }
}
