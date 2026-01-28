mod parse;
mod analysis;
mod compile;
mod map;
mod runtime;

use anyhow::{anyhow, Result};
use serde_json::{json, Value};

use crate::mcp::{ToolDefinition, ToolResult};
use crate::state::ServerState;

/// Get all available tool definitions
pub fn get_tool_definitions() -> Vec<ToolDefinition> {
    let mut tools = Vec::new();

    // Parsing tools
    tools.push(ToolDefinition {
        name: "dm_parse_environment".to_string(),
        description: "Parse a DreamMaker environment (.dme file) and cache the object tree. This must be called before using other analysis tools.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dme_path": {
                    "type": "string",
                    "description": "Path to the .dme environment file"
                }
            },
            "required": ["dme_path"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_get_type".to_string(),
        description: "Get information about a type including its variables, procs, parent type, and children.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "type_path": {
                    "type": "string",
                    "description": "The type path (e.g., '/obj/item', '/mob/living')"
                }
            },
            "required": ["type_path"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_get_proc".to_string(),
        description: "Get detailed information about a procedure including parameters, body location, and documentation.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "type_path": {
                    "type": "string",
                    "description": "The type path containing the proc"
                },
                "proc_name": {
                    "type": "string",
                    "description": "Name of the procedure"
                }
            },
            "required": ["type_path", "proc_name"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_get_var".to_string(),
        description: "Get detailed information about a variable including its type, initial value, and documentation.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "type_path": {
                    "type": "string",
                    "description": "The type path containing the variable"
                },
                "var_name": {
                    "type": "string",
                    "description": "Name of the variable"
                }
            },
            "required": ["type_path", "var_name"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_list_types".to_string(),
        description: "List all types in the object tree, optionally filtered by a path prefix.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "prefix": {
                    "type": "string",
                    "description": "Optional path prefix to filter types (e.g., '/obj')"
                },
                "max_depth": {
                    "type": "integer",
                    "description": "Maximum depth to traverse (default: unlimited)"
                }
            }
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_search_symbols".to_string(),
        description: "Search for types, procs, or variables by name pattern.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "query": {
                    "type": "string",
                    "description": "Search query (supports partial matches)"
                },
                "kind": {
                    "type": "string",
                    "enum": ["type", "proc", "var", "all"],
                    "description": "Kind of symbol to search for (default: all)"
                },
                "limit": {
                    "type": "integer",
                    "description": "Maximum number of results (default: 50)"
                }
            },
            "required": ["query"]
        }),
    });

    // Analysis tools
    tools.push(ToolDefinition {
        name: "dm_check_errors".to_string(),
        description: "Run the type checker and return all diagnostics (errors and warnings).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "file_path": {
                    "type": "string",
                    "description": "Optional: only show errors from this file"
                }
            }
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_get_definition".to_string(),
        description: "Get the source location where a symbol is defined.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "type_path": {
                    "type": "string",
                    "description": "The type path"
                },
                "member_name": {
                    "type": "string",
                    "description": "Optional: name of var or proc to find definition of"
                }
            },
            "required": ["type_path"]
        }),
    });

    // Compile tool
    tools.push(ToolDefinition {
        name: "dm_compile".to_string(),
        description: "Compile the DreamMaker environment using the DM compiler. Returns compiler output and any errors.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dme_path": {
                    "type": "string",
                    "description": "Path to the .dme file to compile"
                }
            },
            "required": ["dme_path"]
        }),
    });

    // Map tools
    tools.push(ToolDefinition {
        name: "dm_render_map".to_string(),
        description: "Render a map file (.dmm) to a PNG image.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dmm_path": {
                    "type": "string",
                    "description": "Path to the .dmm map file"
                },
                "z_level": {
                    "type": "integer",
                    "description": "Z-level to render (default: 1)"
                },
                "output_path": {
                    "type": "string",
                    "description": "Path to save the PNG (default: same as dmm with .png extension)"
                }
            },
            "required": ["dmm_path"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_map_info".to_string(),
        description: "Get information about a map file including dimensions, z-levels, and area statistics.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dmm_path": {
                    "type": "string",
                    "description": "Path to the .dmm map file"
                }
            },
            "required": ["dmm_path"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_find_on_map".to_string(),
        description: "Find all instances of a type on a map.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dmm_path": {
                    "type": "string",
                    "description": "Path to the .dmm map file"
                },
                "type_path": {
                    "type": "string",
                    "description": "Type path to search for (e.g., '/obj/machinery/door')"
                }
            },
            "required": ["dmm_path", "type_path"]
        }),
    });

    // Runtime tools
    tools.push(ToolDefinition {
        name: "dm_run".to_string(),
        description: "Start DreamDaemon with a compiled .dmb file. Returns the process ID and port.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "dmb_path": {
                    "type": "string",
                    "description": "Path to the compiled .dmb file"
                },
                "port": {
                    "type": "integer",
                    "description": "Port to run the server on (default: 1337)"
                }
            },
            "required": ["dmb_path"]
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_stop".to_string(),
        description: "Stop the currently running DreamDaemon instance.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_status".to_string(),
        description: "Get the status of the running game (port, PID, whether it's running).".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {}
        }),
    });

    tools.push(ToolDefinition {
        name: "dm_topic".to_string(),
        description: "Send a Topic() call to the running game server. Use this to communicate with debug handlers in the game.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "topic": {
                    "type": "string",
                    "description": "The topic string to send (e.g., '?debug_screenshot' or '?debug_click=10,20')"
                },
                "timeout_ms": {
                    "type": "integer",
                    "description": "Timeout in milliseconds (default: 5000)"
                }
            },
            "required": ["topic"]
        }),
    });

    // Client connection test tool
    tools.push(ToolDefinition {
        name: "dm_connect_test".to_string(),
        description: "Test connecting to the running game as a BYOND client and log received packets. Used for protocol debugging.".to_string(),
        input_schema: json!({
            "type": "object",
            "properties": {
                "timeout_secs": {
                    "type": "integer",
                    "description": "How long to wait for packets (default: 5 seconds)"
                }
            }
        }),
    });

    tools
}

/// Call a tool by name with the given arguments
pub async fn call_tool(state: &mut ServerState, name: &str, args: Value) -> Result<ToolResult> {
    match name {
        // Parsing tools
        "dm_parse_environment" => parse::parse_environment(state, args).await,
        "dm_get_type" => parse::get_type(state, args).await,
        "dm_get_proc" => parse::get_proc(state, args).await,
        "dm_get_var" => parse::get_var(state, args).await,
        "dm_list_types" => parse::list_types(state, args).await,
        "dm_search_symbols" => parse::search_symbols(state, args).await,

        // Analysis tools
        "dm_check_errors" => analysis::check_errors(state, args).await,
        "dm_get_definition" => analysis::get_definition(state, args).await,

        // Compile tool
        "dm_compile" => compile::compile(args).await,

        // Map tools
        "dm_render_map" => map::render_map(state, args).await,
        "dm_map_info" => map::map_info(args).await,
        "dm_find_on_map" => map::find_on_map(args).await,

        // Runtime tools
        "dm_run" => runtime::run(state, args).await,
        "dm_stop" => runtime::stop(state, args).await,
        "dm_status" => runtime::status(state, args).await,
        "dm_topic" => runtime::topic(state, args).await,
        "dm_connect_test" => runtime::connect_test(state, args).await,

        _ => Err(anyhow!("Unknown tool: {}", name)),
    }
}
