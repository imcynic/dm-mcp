# BYOND Forums Post

**Title:** dm-mcp: MCP Server for AI-Assisted DreamMaker Development

---

Hi everyone,

I've released **dm-mcp**, an MCP (Model Context Protocol) server that bridges DreamMaker development with modern AI coding assistants like Claude, Cursor, and Windsurf.

## What is MCP?

Model Context Protocol is a standard that lets AI assistants connect to external tools. Instead of the AI just seeing your code as text, it can use specialized tools to actually understand your project structure.

## What dm-mcp Provides

**17 tools for DM development:**

| Category | Tools |
|----------|-------|
| Code Intelligence | `dm_parse_environment`, `dm_get_type`, `dm_get_proc`, `dm_get_var`, `dm_list_types`, `dm_search_symbols`, `dm_check_errors`, `dm_get_definition` |
| Compilation | `dm_compile` |
| Runtime | `dm_run`, `dm_stop`, `dm_status`, `dm_topic` |
| Map Analysis | `dm_map_info`, `dm_find_on_map`, `dm_render_map` |
| Testing | `dm_connect_test` |

## Technical Details

- Written in Rust using tokio for async I/O
- Uses SpacemanDMM for parsing and type checking
- Includes full BYOND client protocol implementation (with RUNSUB encryption)
- Supports both Windows and Linux
- 3.6MB optimized binary

## Use Cases

1. **Code exploration**: AI can navigate your type hierarchy, understand inheritance, find related procs
2. **Debugging help**: Real-time diagnostics from dreamchecker fed to the AI
3. **Map work**: Query map contents, find object instances, generate previews
4. **Server management**: Start/stop DreamDaemon, send Topic() calls, monitor status

## Installation

```bash
git clone https://github.com/imcynic/dm-mcp.git
cd dm-mcp
cargo build --release
```

Then configure your MCP client to use the binary.

**GitHub:** https://github.com/imcynic/dm-mcp

I'd welcome any feedback, feature requests, or bug reports. This is the first MCP server for DreamMaker that I'm aware of - hoping it helps the community leverage modern AI tools for BYOND development.
