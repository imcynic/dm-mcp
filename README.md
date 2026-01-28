# dm-mcp

**MCP Server for DreamMaker/BYOND Development**

An [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server that provides AI coding assistants with full access to DreamMaker language tooling. Works with Claude, Cursor, Windsurf, Continue, and any MCP-compatible client.

## Features

### Code Intelligence
- **Parse & Navigate** - Load `.dme` projects, explore type hierarchies, search symbols
- **Type Checking** - Real-time diagnostics via SpacemanDMM's dreamchecker
- **Go to Definition** - Find where any type, proc, or variable is defined

### Compilation & Runtime  
- **Compile** - Build projects with the DM compiler
- **Run/Stop** - Control DreamDaemon instances
- **Topic Calls** - Send `Topic()` messages to running servers

### Map Tools
- **Map Info** - Get dimensions, z-levels, area statistics
- **Find on Map** - Search for object instances across maps
- **Render Maps** - Generate PNG previews of `.dmm` files

### Protocol Support
- Full BYOND client protocol implementation (for testing)
- RUNSUB encryption for secure packet handling

## Installation

### Prerequisites
- [Rust](https://rustup.rs/) 1.70+
- [BYOND](http://www.byond.com/download/) (for compilation and runtime features)

### Build from Source

```bash
git clone https://github.com/YOUR_USERNAME/dm-mcp.git
cd dm-mcp
cargo build --release
```

The binary will be at `target/release/dm-mcp`.

### Configure Your MCP Client

#### Claude Code / Cursor / Windsurf

Add to your MCP settings (usually `~/.config/claude/mcp.json` or IDE settings):

```json
{
  "mcpServers": {
    "dm-mcp": {
      "command": "/path/to/dm-mcp"
    }
  }
}
```

#### Continue

Add to `~/.continue/config.json`:

```json
{
  "experimental": {
    "modelContextProtocolServers": [
      {
        "name": "dm-mcp",
        "transport": {
          "type": "stdio",
          "command": "/path/to/dm-mcp"
        }
      }
    ]
  }
}
```

## Available Tools

| Tool | Description |
|------|-------------|
| `dm_parse_environment` | Parse a `.dme` file and cache the object tree |
| `dm_get_type` | Get type info (vars, procs, parent, children) |
| `dm_get_proc` | Get proc details (params, location, docs) |
| `dm_get_var` | Get variable info (type, initial value) |
| `dm_list_types` | List types with optional path prefix filter |
| `dm_search_symbols` | Search for types, procs, or vars by pattern |
| `dm_check_errors` | Run type checker, get diagnostics |
| `dm_get_definition` | Find source location of any symbol |
| `dm_compile` | Compile the project |
| `dm_render_map` | Render a map to PNG |
| `dm_map_info` | Get map dimensions and statistics |
| `dm_find_on_map` | Find instances of a type on a map |
| `dm_run` | Start DreamDaemon with a `.dmb` file |
| `dm_stop` | Stop the running game |
| `dm_status` | Get game server status |
| `dm_topic` | Send a Topic() call to the server |
| `dm_connect_test` | Test BYOND client protocol |

## Example Usage

Once configured, you can ask your AI assistant things like:

> "Parse /code/myss13/myss13.dme and show me the type hierarchy for /mob/living"

> "Find all instances of /obj/machinery/door on the station map"

> "Check for type errors in the codebase"

> "Compile the project and run it on port 1337"

## How It Works

dm-mcp uses [SpacemanDMM](https://github.com/SpaceManiac/SpacemanDMM) for parsing and type checking. This is the same tooling used by the SS13 community for linting and IDE support.

The server communicates over stdio using JSON-RPC, following the MCP specification.

## Platform Support

- **Windows** - Full support (BYOND native)
- **Linux** - Full support (requires BYOND Linux build)
- **macOS** - Untested but should work with BYOND Wine wrapper

## BYOND Paths

The server looks for BYOND in these locations:

**Windows:**
- `C:\Program Files (x86)\BYOND\bin\`
- `C:\Program Files\BYOND\bin\`

**Linux:**
- `/usr/local/byond/bin/`
- `/opt/byond/bin/`
- System PATH (`dm`, `DreamMaker`, `dreamdaemon`, `DreamDaemon`)

## Development

```bash
# Run with debug logging
RUST_LOG=dm_mcp=debug cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## License

MIT

## Contributing

Issues and PRs welcome! This project aims to bring modern AI tooling to the BYOND/DreamMaker ecosystem.

---

*Built with ❤️ for the SS13 community*
