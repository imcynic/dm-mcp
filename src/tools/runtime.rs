use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::io::Write;
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::info;

use crate::client::BYONDClient;
use crate::mcp::ToolResult;
use crate::state::ServerState;

/// Find the DreamDaemon executable
fn find_dreamdaemon() -> Option<PathBuf> {
    let possible_paths = [
        r"C:\Program Files (x86)\BYOND\bin\dreamdaemon.exe",
        r"C:\Program Files\BYOND\bin\dreamdaemon.exe",
        "/usr/local/byond/bin/DreamDaemon",
        "/opt/byond/bin/DreamDaemon",
    ];

    for path in &possible_paths {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    if let Ok(path) = which::which("dreamdaemon") {
        return Some(path);
    }
    if let Ok(path) = which::which("DreamDaemon") {
        return Some(path);
    }

    None
}

/// Start DreamDaemon with a .dmb file
pub async fn run(state: &mut ServerState, args: Value) -> Result<ToolResult> {
    // Check if already running
    if state.is_game_running() {
        return Ok(ToolResult::error("A game instance is already running. Use dm_stop first."));
    }

    let dmb_path = args.get("dmb_path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing dmb_path argument"))?;

    let port = args.get("port")
        .and_then(|v| v.as_u64())
        .unwrap_or(1337) as u16;

    let path = PathBuf::from(dmb_path);
    if !path.exists() {
        return Ok(ToolResult::error(format!("DMB file not found: {}", dmb_path)));
    }

    let dreamdaemon = find_dreamdaemon()
        .ok_or_else(|| anyhow!("DreamDaemon not found. Please install BYOND."))?;

    info!("Starting DreamDaemon with {} on port {}", dmb_path, port);

    // Start DreamDaemon
    // Arguments: dmb_path port -trusted -logself
    // Note: Removed -invisible so the server auto-starts properly
    let child = Command::new(&dreamdaemon)
        .arg(&path)
        .arg(port.to_string())
        .arg("-trusted")
        .arg("-logself")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let pid = child.id();
    state.set_game_process(child, port);

    // Give the server a moment to start
    tokio::time::sleep(Duration::from_millis(1500)).await;

    // Check if it's actually running
    if !state.is_game_running() {
        return Ok(ToolResult::error("DreamDaemon process exited immediately. Check the DMB file."));
    }

    Ok(ToolResult::text(json!({
        "success": true,
        "pid": pid,
        "port": port,
        "dmb_path": dmb_path,
        "message": format!("DreamDaemon started on port {}", port)
    }).to_string()))
}

/// Stop the running DreamDaemon instance
pub async fn stop(state: &mut ServerState, _args: Value) -> Result<ToolResult> {
    if !state.is_game_running() {
        return Ok(ToolResult::error("No game instance is currently running."));
    }

    match state.stop_game_process() {
        Ok(()) => {
            Ok(ToolResult::text(json!({
                "success": true,
                "message": "DreamDaemon stopped"
            }).to_string()))
        }
        Err(e) => {
            Ok(ToolResult::error(format!("Failed to stop DreamDaemon: {}", e)))
        }
    }
}

/// Get status of the running game
pub async fn status(state: &mut ServerState, _args: Value) -> Result<ToolResult> {
    if !state.is_game_running() {
        return Ok(ToolResult::text(json!({
            "running": false
        }).to_string()));
    }

    let port = state.game_port.unwrap_or(0);
    let pid = state.game_process.as_ref().map(|p| p.id());

    Ok(ToolResult::text(json!({
        "running": true,
        "port": port,
        "pid": pid
    }).to_string()))
}

/// Send a Topic() call to the running game server
pub async fn topic(state: &mut ServerState, args: Value) -> Result<ToolResult> {
    if !state.is_game_running() {
        return Ok(ToolResult::error("No game instance is currently running."));
    }

    let topic_string = args.get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing topic argument"))?;

    let port = state.game_port.unwrap_or(1337);
    let timeout_ms = args.get("timeout_ms")
        .and_then(|v| v.as_u64())
        .unwrap_or(5000);

    info!("Sending Topic to port {}: {}", port, topic_string);

    match send_topic(&format!("127.0.0.1:{}", port), topic_string, timeout_ms).await {
        Ok(response) => {
            Ok(ToolResult::text(json!({
                "success": true,
                "response": response
            }).to_string()))
        }
        Err(e) => {
            Ok(ToolResult::error(format!("Topic call failed: {}", e)))
        }
    }
}

/// Send a BYOND Topic packet and get response
async fn send_topic(address: &str, topic: &str, timeout_ms: u64) -> Result<String> {
    // BYOND Topic packet format:
    // Bytes 0-1: 0x00 0x83 (magic header)
    // Bytes 2-3: Length of data (big-endian u16)
    // Bytes 4+: The topic string as null-terminated ASCII

    // Strip leading "?" if present - it's a URL delimiter, not part of the topic
    let topic_clean = topic.strip_prefix('?').unwrap_or(topic);
    let topic_bytes = topic_clean.as_bytes();

    info!("Sending topic packet for: {} (cleaned: {})", topic, topic_clean);
    let packet_len = topic_bytes.len() + 6; // +1 for null terminator, +5 for header overhead

    let mut packet = Vec::with_capacity(packet_len);
    packet.push(0x00); // Magic byte 1
    packet.push(0x83); // Magic byte 2

    // Length (big-endian u16) - includes 5 padding bytes + 1 type byte + string + null terminator
    let data_len = (topic_bytes.len() + 7) as u16;
    packet.push((data_len >> 8) as u8);
    packet.push((data_len & 0xFF) as u8);

    // Type: 0x06 for string topic
    packet.push(0x00);
    packet.push(0x00);
    packet.push(0x00);
    packet.push(0x00);
    packet.push(0x00);
    packet.push(0x06); // String type

    // The topic string
    packet.extend_from_slice(topic_bytes);
    packet.push(0x00); // Null terminator

    // Clone address for the blocking task
    let address_owned = address.to_string();

    // Connect with timeout
    let stream = tokio::time::timeout(
        Duration::from_millis(timeout_ms),
        tokio::task::spawn_blocking(move || {
            TcpStream::connect(&address_owned)
        })
    ).await??;

    let mut stream = stream?;
    stream.set_read_timeout(Some(Duration::from_millis(timeout_ms)))?;
    stream.set_write_timeout(Some(Duration::from_millis(timeout_ms)))?;

    // Send the packet
    stream.write_all(&packet)?;
    stream.flush()?;

    // Read response
    let mut response_header = [0u8; 4];
    std::io::Read::read_exact(&mut stream, &mut response_header)?;

    if response_header[0] != 0x00 || response_header[1] != 0x83 {
        return Err(anyhow!("Invalid response header"));
    }

    let response_len = ((response_header[2] as u16) << 8) | (response_header[3] as u16);

    if response_len == 0 {
        return Ok(String::new());
    }

    let mut response_data = vec![0u8; response_len as usize];
    std::io::Read::read_exact(&mut stream, &mut response_data)?;

    // Response format depends on type byte
    if response_data.is_empty() {
        return Ok(String::new());
    }

    // Skip type indicator and parse as string
    let response_str = if response_data.len() > 1 {
        // Check response type
        match response_data[0] {
            0x06 => {
                // ASCII string - skip type byte, read until null
                let string_data = &response_data[1..];
                let null_pos = string_data.iter().position(|&b| b == 0).unwrap_or(string_data.len());
                String::from_utf8_lossy(&string_data[..null_pos]).to_string()
            }
            0x2a => {
                // Float response
                if response_data.len() >= 5 {
                    let float_bytes: [u8; 4] = [response_data[1], response_data[2], response_data[3], response_data[4]];
                    let value = f32::from_le_bytes(float_bytes);
                    value.to_string()
                } else {
                    String::new()
                }
            }
            _ => {
                // Unknown type, return hex representation
                format!("raw:{:02x?}", response_data)
            }
        }
    } else {
        String::new()
    };

    Ok(response_str)
}

/// Test connecting to the BYOND server as a client and log received packets
pub async fn connect_test(state: &mut ServerState, args: Value) -> Result<ToolResult> {
    if !state.is_game_running() {
        return Ok(ToolResult::error("No game instance is currently running."));
    }

    let port = state.game_port.unwrap_or(1337);
    let timeout_secs = args.get("timeout_secs")
        .and_then(|v| v.as_u64())
        .unwrap_or(5);

    info!("Testing BYOND client connection to port {}", port);

    // Run the connection in a blocking task since it uses sync I/O
    let result = tokio::task::spawn_blocking(move || -> Result<serde_json::Value> {
        // Connect to the server
        let mut client = BYONDClient::connect("127.0.0.1", port)?;

        // Receive initial packets
        let packets = client.receive_initial_packets(timeout_secs)?;

        // Build summary of received packets
        let packet_summary: Vec<serde_json::Value> = packets.iter().map(|p| {
            json!({
                "type": format!("0x{:04X}", u16::from(p.packet_type)),
                "type_name": format!("{:?}", p.packet_type),
                "seq": p.seq,
                "data_len": p.data.len(),
                "data_preview": if p.data.len() <= 64 {
                    format!("{:02X?}", p.data)
                } else {
                    format!("{:02X?}... ({} more bytes)", &p.data[..64], p.data.len() - 64)
                }
            })
        }).collect();

        client.disconnect()?;

        Ok(json!({
            "success": true,
            "packets_received": packets.len(),
            "packets": packet_summary
        }))
    }).await??;

    Ok(ToolResult::text(result.to_string()))
}
