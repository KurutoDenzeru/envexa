#![allow(dead_code)]
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcMessage {
    Request {
        jsonrpc: String,
        id: Value,
        method: String,
        #[serde(default)]
        params: Option<Value>,
    },
    Notification {
        jsonrpc: String,
        method: String,
        #[serde(default)]
        params: Option<Value>,
    },
    Response {
        jsonrpc: String,
        id: Value,
        #[serde(default)]
        result: Option<Value>,
        #[serde(default)]
        error: Option<JsonRpcError>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcError {
    pub fn new(code: i32, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn ok(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn err(id: Value, code: i32, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError::new(code, message)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializeParams {
    #[serde(default)]
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: Value,
    #[serde(default)]
    pub client_info: Value,
}

#[derive(Debug, Serialize)]
pub struct InitializeResult {
    pub protocol_version: String,
    pub capabilities: ServerCapabilities,
    pub server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
pub struct ServerCapabilities {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompts: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ToolDescription {
    pub name: String,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_schema: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct CallToolResult {
    pub content: Vec<ContentItem>,
}

#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub content_type: String,
    pub text: String,
}

impl ContentItem {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content_type: "text".into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PromptDescription {
    pub name: String,
    pub description: String,
}

#[derive(Debug, Serialize)]
pub struct GetPromptResult {
    pub messages: Vec<PromptMessage>,
}

#[derive(Debug, Serialize)]
pub struct PromptMessage {
    pub role: String,
    pub content: ContentItem,
}

#[derive(Debug, Serialize)]
pub struct ResourceDescription {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub mime_type: String,
}

#[derive(Debug, Serialize)]
pub struct ReadResourceResult {
    pub contents: Vec<ResourceContent>,
}

#[derive(Debug, Serialize)]
pub struct ResourceContent {
    pub uri: String,
    pub mime_type: String,
    pub text: String,
}

pub async fn read_loop(
    tools: Vec<ToolDescription>,
    prompts: Vec<PromptDescription>,
    resources: Vec<ResourceDescription>,
    tool_handler: impl Fn(&str, Option<Value>) -> Result<Value, String>,
    prompt_handler: impl Fn(&str) -> Result<Value, String>,
    resource_handler: impl Fn(&str) -> Result<Value, String>,
) -> Result<(), anyhow::Error> {
    let stdin = tokio::io::stdin();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();
    let mut stdout = tokio::io::stdout();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }

        let msg: JsonRpcMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                // Can't respond without an id, skip
                eprintln!("JSON-RPC parse error: {e}");
                continue;
            }
        };

        match msg {
            JsonRpcMessage::Request {
                id, method, params, ..
            } => {
                let response = handle_request(
                    &method,
                    params,
                    &tools,
                    &prompts,
                    &resources,
                    &tool_handler,
                    &prompt_handler,
                    &resource_handler,
                );
                let resp = match response {
                    Ok(val) => JsonRpcResponse::ok(id, val),
                    Err(e) => JsonRpcResponse::err(id, -32601, e),
                };
                let mut json = serde_json::to_string(&resp)?;
                json.push('\n');
                stdout.write_all(json.as_bytes()).await?;
                stdout.flush().await?;
            }
            JsonRpcMessage::Notification {
                method, params: _, ..
            } => {
                if method == "notifications/initialized" {
                    // Handshake complete, nothing to do
                } else if method == "$/cancelRequest" {
                    // No-op for now
                }
                // Other notifications are silently ignored
            }
            JsonRpcMessage::Response { .. } => {
                // We don't send requests to the client, so responses are unexpected
            }
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn handle_request(
    method: &str,
    params: Option<Value>,
    tools: &[ToolDescription],
    prompts: &[PromptDescription],
    resources: &[ResourceDescription],
    tool_handler: &impl Fn(&str, Option<Value>) -> Result<Value, String>,
    prompt_handler: &impl Fn(&str) -> Result<Value, String>,
    resource_handler: &impl Fn(&str) -> Result<Value, String>,
) -> Result<Value, String> {
    match method {
        "initialize" => {
            let result = InitializeResult {
                protocol_version: "2024-11-05".into(),
                capabilities: ServerCapabilities {
                    tools: Some(Value::Object(serde_json::Map::new())),
                    prompts: Some(Value::Object(serde_json::Map::new())),
                    resources: Some(Value::Object(serde_json::Map::new())),
                    logging: None,
                },
                server_info: ServerInfo {
                    name: "envexa".into(),
                    version: "0.1.0".into(),
                },
            };
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "ping" => Ok(Value::Object(serde_json::Map::new())),
        "tools/list" => serde_json::to_value(tools).map_err(|e| e.to_string()),
        "tools/call" => {
            let params = params.ok_or_else(|| "Missing params".to_string())?;
            let name = params["name"]
                .as_str()
                .ok_or_else(|| "Missing tool name".to_string())?;
            let args = params.get("arguments").cloned();
            let result_val = tool_handler(name, args)?;
            let text = result_val.as_str().unwrap_or("").to_string();
            let result = CallToolResult {
                content: vec![ContentItem::text(text)],
            };
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "prompts/list" => serde_json::to_value(prompts).map_err(|e| e.to_string()),
        "prompts/get" => {
            let params = params.ok_or_else(|| "Missing params".to_string())?;
            let name = params["name"]
                .as_str()
                .ok_or_else(|| "Missing prompt name".to_string())?;
            let result_val = prompt_handler(name)?;
            let text = result_val.as_str().unwrap_or("").to_string();
            let result = GetPromptResult {
                messages: vec![PromptMessage {
                    role: "assistant".into(),
                    content: ContentItem::text(text),
                }],
            };
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "resources/list" => serde_json::to_value(resources).map_err(|e| e.to_string()),
        "resources/read" => {
            let params = params.ok_or_else(|| "Missing params".to_string())?;
            let uri = params["uri"]
                .as_str()
                .ok_or_else(|| "Missing resource URI".to_string())?;
            let result_val = resource_handler(uri)?;
            let text = result_val.as_str().unwrap_or("").to_string();
            let result = ReadResourceResult {
                contents: vec![ResourceContent {
                    uri: uri.to_string(),
                    mime_type: "text/markdown".into(),
                    text,
                }],
            };
            serde_json::to_value(result).map_err(|e| e.to_string())
        }
        "sampling/createMessage" | "logging/setLevel" => {
            // Not supported yet
            Ok(Value::Object(serde_json::Map::new()))
        }
        _ => Err(format!("Method not found: {method}")),
    }
}
