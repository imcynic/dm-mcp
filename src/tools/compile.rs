use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

use crate::mcp::ToolResult;

/// Find the DreamMaker compiler
fn find_dm_compiler() -> Option<PathBuf> {
    // Try common locations
    let possible_paths = [
        // Windows BYOND installation
        r"C:\Program Files (x86)\BYOND\bin\dm.exe",
        r"C:\Program Files\BYOND\bin\dm.exe",
        // Linux/WSL
        "/usr/local/byond/bin/DreamMaker",
        "/opt/byond/bin/DreamMaker",
    ];

    for path in &possible_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Try PATH
    if let Ok(path) = which::which("dm") {
        return Some(path);
    }
    if let Ok(path) = which::which("DreamMaker") {
        return Some(path);
    }

    None
}

/// Compile a DreamMaker environment
pub async fn compile(args: Value) -> Result<ToolResult> {
    let dme_path = args.get("dme_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing dme_path argument"))?;

    let path = PathBuf::from(dme_path);
    if !path.exists() {
        return Ok(ToolResult::error(format!("File not found: {}", dme_path)));
    }

    let compiler = find_dm_compiler()
        .ok_or_else(|| anyhow!("DreamMaker compiler not found. Please install BYOND."))?;

    info!("Compiling {} with {:?}", dme_path, compiler);

    let output = Command::new(&compiler)
        .arg(&path)
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Parse output for errors/warnings
    let mut errors: Vec<Value> = Vec::new();
    let mut warnings: Vec<Value> = Vec::new();

    for line in stdout.lines().chain(stderr.lines()) {
        // BYOND error format: filename:line:error: message
        // or: filename:line:warning: message
        if let Some(colon_pos) = line.find(':') {
            let rest = &line[colon_pos + 1..];
            if let Some(second_colon) = rest.find(':') {
                let line_num = &rest[..second_colon];
                let rest2 = &rest[second_colon + 1..];

                if rest2.contains("error") {
                    errors.push(json!({
                        "message": line,
                        "line": line_num.parse::<u32>().unwrap_or(0)
                    }));
                } else if rest2.contains("warning") {
                    warnings.push(json!({
                        "message": line,
                        "line": line_num.parse::<u32>().unwrap_or(0)
                    }));
                }
            }
        }
    }

    let success = output.status.success();
    let dmb_path = path.with_extension("dmb");
    let dmb_exists = dmb_path.exists();

    let result = json!({
        "success": success,
        "exit_code": output.status.code(),
        "dmb_created": dmb_exists,
        "dmb_path": if dmb_exists { Some(dmb_path.display().to_string()) } else { None },
        "errors": errors,
        "warnings": warnings,
        "stdout": stdout.to_string(),
        "stderr": stderr.to_string()
    });

    if success {
        Ok(ToolResult::text(serde_json::to_string_pretty(&result)?))
    } else {
        Ok(ToolResult::error(serde_json::to_string_pretty(&result)?))
    }
}
