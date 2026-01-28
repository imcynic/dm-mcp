use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::collections::HashMap;
use tracing::info;

use crate::mcp::ToolResult;
use crate::state::ServerState;

/// Render a map to PNG
pub async fn render_map(state: &mut ServerState, args: Value) -> Result<ToolResult> {
    let dmm_path = args.get("dmm_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing dmm_path argument"))?;

    let z_level = args.get("z_level")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as usize;

    let output_path = args.get("output_path")
        .and_then(|v| v.as_str())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let mut p = PathBuf::from(dmm_path);
            p.set_extension("png");
            p
        });

    let path = PathBuf::from(dmm_path);
    if !path.exists() {
        return Ok(ToolResult::error(format!("File not found: {}", dmm_path)));
    }

    // Check if we have an environment loaded (needed for icons)
    let _objtree = state.objtree.as_ref()
        .ok_or_else(|| anyhow!("No environment loaded. Call dm_parse_environment first to load icons."))?;

    info!("Rendering map: {} z-level {} to {:?}", dmm_path, z_level, output_path);

    // Parse the map file
    let _map_content = std::fs::read_to_string(&path)?;

    // Use dmm-tools to parse and render
    // Note: This is a simplified implementation. Full rendering would require
    // loading icons from .dmi files and compositing them.

    // For now, return info about what would be rendered
    let result = json!({
        "dmm_path": dmm_path,
        "z_level": z_level,
        "output_path": output_path.display().to_string(),
        "status": "Map rendering requires icon loading. Use dm_map_info for map details.",
        "note": "Full rendering support requires the complete dmm-tools pipeline with icon files."
    });

    Ok(ToolResult::text(serde_json::to_string_pretty(&result)?))
}

/// Get map information
pub async fn map_info(args: Value) -> Result<ToolResult> {
    let dmm_path = args.get("dmm_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing dmm_path argument"))?;

    let path = PathBuf::from(dmm_path);
    if !path.exists() {
        return Ok(ToolResult::error(format!("File not found: {}", dmm_path)));
    }

    info!("Getting map info: {}", dmm_path);

    let content = std::fs::read_to_string(&path)?;

    // Parse map format (TGM or original)
    let is_tgm = content.contains("//MAP CONVERTED BY dmm2tgm.py");

    // Extract dimensions from map header
    // DMM format has lines like: (1,1,1) = {"
    let mut max_x = 0u32;
    let mut max_y = 0u32;
    let mut max_z = 0u32;

    // Count unique keys (tile definitions)
    let mut keys: HashMap<String, u32> = HashMap::new();

    for line in content.lines() {
        // Match coordinate lines like (1,1,1) = {" or "aaa" = (
        if let Some(start) = line.find('(') {
            if let Some(end) = line.find(')') {
                let coords = &line[start + 1..end];
                let parts: Vec<&str> = coords.split(',').collect();
                if parts.len() == 3 {
                    if let (Ok(x), Ok(y), Ok(z)) = (
                        parts[0].trim().parse::<u32>(),
                        parts[1].trim().parse::<u32>(),
                        parts[2].trim().parse::<u32>()
                    ) {
                        max_x = max_x.max(x);
                        max_y = max_y.max(y);
                        max_z = max_z.max(z);
                    }
                }
            }
        }

        // Count tile definition keys (lines starting with ")
        if line.starts_with('"') {
            if let Some(end) = line[1..].find('"') {
                let key = &line[1..end + 1];
                *keys.entry(key.to_string()).or_insert(0) += 1;
            }
        }
    }

    // Extract type references from the map
    let mut type_counts: HashMap<String, u32> = HashMap::new();
    for line in content.lines() {
        // Look for type paths like /turf/open/floor
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '/' {
                let mut type_path = String::from("/");
                while let Some(&next) = chars.peek() {
                    if next.is_alphanumeric() || next == '/' || next == '_' {
                        type_path.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if type_path.len() > 1 {
                    // Get just the base type (first two segments)
                    let segments: Vec<&str> = type_path.split('/').collect();
                    if segments.len() >= 2 {
                        let base = format!("/{}", segments[1]);
                        *type_counts.entry(base).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    // Sort types by count
    let mut sorted_types: Vec<_> = type_counts.into_iter().collect();
    sorted_types.sort_by(|a, b| b.1.cmp(&a.1));

    let result = json!({
        "file": dmm_path,
        "format": if is_tgm { "TGM" } else { "DMM" },
        "dimensions": {
            "x": max_x,
            "y": max_y,
            "z": max_z
        },
        "unique_tiles": keys.len(),
        "file_size_bytes": std::fs::metadata(&path)?.len(),
        "top_types": sorted_types.into_iter().take(20).collect::<Vec<_>>()
    });

    Ok(ToolResult::text(serde_json::to_string_pretty(&result)?))
}

/// Find instances of a type on a map
pub async fn find_on_map(args: Value) -> Result<ToolResult> {
    let dmm_path = args.get("dmm_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing dmm_path argument"))?;

    let type_path = args.get("type_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing type_path argument"))?;

    let path = PathBuf::from(dmm_path);
    if !path.exists() {
        return Ok(ToolResult::error(format!("File not found: {}", dmm_path)));
    }

    info!("Finding {} on map {}", type_path, dmm_path);

    let content = std::fs::read_to_string(&path)?;

    // Find all keys that contain this type
    let mut matching_keys: Vec<String> = Vec::new();
    let mut in_definition = false;
    let mut current_key = String::new();
    let mut current_def = String::new();

    for line in content.lines() {
        if line.starts_with('"') && line.contains("= (") {
            // Start of a tile definition
            if let Some(end) = line[1..].find('"') {
                in_definition = true;
                current_key = line[1..end + 1].to_string();
                current_def = line.to_string();
            }
        } else if in_definition {
            current_def.push_str(line);
            if line.contains(')') {
                // End of definition
                in_definition = false;
                if current_def.contains(type_path) {
                    matching_keys.push(current_key.clone());
                }
                current_def.clear();
            }
        }
    }

    // Now find coordinates where these keys are used
    // This is simplified - full implementation would parse the grid properly
    let result = json!({
        "type_path": type_path,
        "dmm_path": dmm_path,
        "matching_tile_keys": matching_keys.len(),
        "keys": matching_keys,
        "note": "Use these keys to find the type in the map grid section"
    });

    Ok(ToolResult::text(serde_json::to_string_pretty(&result)?))
}
