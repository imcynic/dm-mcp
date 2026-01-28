use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};
use tracing::{debug, error, info};

use crate::state::ServerState;
use crate::tools;

/// JSON-RPC 2.0 Request
#[derive(Debug, Deserialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    #[serde(default)]
    params: Value,
}

/// JSON-RPC 2.0 Response
#[derive(Debug, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

/// MCP Server Info
#[derive(Debug, Serialize)]
struct ServerInfo {
    name: String,
    version: String,
}

/// MCP Initialize Result
#[derive(Debug, Serialize)]
struct InitializeResult {
    #[serde(rename = "protocolVersion")]
    protocol_version: String,
    capabilities: ServerCapabilities,
    #[serde(rename = "serverInfo")]
    server_info: ServerInfo,
}

#[derive(Debug, Serialize)]
struct ServerCapabilities {
    tools: ToolsCapability,
}

#[derive(Debug, Serialize)]
struct ToolsCapability {
    #[serde(rename = "listChanged")]
    list_changed: bool,
}

/// Tool definition for MCP
#[derive(Debug, Serialize, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Tool call result
#[derive(Debug, Serialize)]
pub struct ToolResult {
    pub content: Vec<ToolContent>,
    #[serde(rename = "isError", skip_serializing_if = "Option::is_none")]
    pub is_error: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum ToolContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { data: String, #[serde(rename = "mimeType")] mime_type: String },
}

impl ToolResult {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: text.into() }],
            is_error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Text { text: message.into() }],
            is_error: Some(true),
        }
    }

    pub fn image(data: String, mime_type: impl Into<String>) -> Self {
        Self {
            content: vec![ToolContent::Image {
                data,
                mime_type: mime_type.into()
            }],
            is_error: None,
        }
    }
}

/// Run the MCP server (stdio JSON-RPC)
pub async fn run_server() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut state = ServerState::new();

    info!("MCP Server ready, waiting for requests...");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        debug!("Received: {}", line);

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(req) => req,
            Err(e) => {
                error!("Failed to parse request: {}", e);
                continue;
            }
        };

        let response = handle_request(&mut state, request).await;

        if let Some(response) = response {
            let response_str = serde_json::to_string(&response)?;
            debug!("Sending: {}", response_str);
            writeln!(stdout, "{}", response_str)?;
            stdout.flush()?;
        }
    }

    Ok(())
}

async fn handle_request(state: &mut ServerState, request: JsonRpcRequest) -> Option<JsonRpcResponse> {
    let id = request.id.clone()?; // Notifications have no id

    let result = match request.method.as_str() {
        "initialize" => handle_initialize(&request.params),
        "initialized" => return None, // Notification, no response
        "tools/list" => handle_tools_list(),
        "tools/call" => handle_tools_call(state, &request.params).await,
        "ping" => Ok(json!({})),
        _ => Err(anyhow!("Unknown method: {}", request.method)),
    };

    Some(match result {
        Ok(result) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: e.to_string(),
                data: None,
            }),
        },
    })
}

fn handle_initialize(_params: &Value) -> Result<Value> {
    info!("Initializing MCP server");

    Ok(serde_json::to_value(InitializeResult {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ServerCapabilities {
            tools: ToolsCapability {
                list_changed: false,
            },
        },
        server_info: ServerInfo {
            name: "dm-mcp".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        },
    })?)
}

fn handle_tools_list() -> Result<Value> {
    let tools = tools::get_tool_definitions();
    Ok(json!({ "tools": tools }))
}

async fn handle_tools_call(state: &mut ServerState, params: &Value) -> Result<Value> {
    let name = params.get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing tool name"))?;

    let arguments = params.get("arguments")
        .cloned()
        .unwrap_or(json!({}));

    info!("Calling tool: {} with args: {}", name, arguments);

    let result = tools::call_tool(state, name, arguments).await?;

    Ok(serde_json::to_value(result)?)
}
