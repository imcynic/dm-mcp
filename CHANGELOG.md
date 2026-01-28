# Changelog

All notable changes to dm-mcp will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-01-28

### Added

- Initial release
- **17 MCP Tools** for DreamMaker development:
  - `dm_parse_environment` - Parse .dme files and cache object tree
  - `dm_get_type` - Get type definition with vars and procs
  - `dm_get_proc` - Get proc source code and location
  - `dm_get_var` - Get variable definition and value
  - `dm_list_types` - List types matching a pattern
  - `dm_search_symbols` - Search for symbols by name
  - `dm_check_errors` - Run dreamchecker type analysis
  - `dm_get_definition` - Go to definition for any symbol
  - `dm_compile` - Compile project with DreamMaker
  - `dm_render_map` - Render .dmm map to PNG
  - `dm_map_info` - Get map dimensions and statistics
  - `dm_find_on_map` - Find object instances on maps
  - `dm_run` - Start DreamDaemon server
  - `dm_stop` - Stop DreamDaemon server
  - `dm_status` - Check DreamDaemon status
  - `dm_topic` - Send Topic() calls to running server
  - `dm_connect_test` - Test BYOND client connection

- **BYOND Client Protocol** implementation
  - Full packet framing and types
  - RUNSUB encryption support
  - Connection testing capabilities

- Cross-platform support (Windows primary, Linux compatible)
- Integration with SpacemanDMM for AST parsing and type checking

### Technical Notes

- Built with Rust + Tokio for async I/O
- Uses tree-sitter style parsing via SpacemanDMM
- Stateful server design (caches parsed environments)

[0.1.0]: https://github.com/imcynic/dm-mcp/releases/tag/v0.1.0
