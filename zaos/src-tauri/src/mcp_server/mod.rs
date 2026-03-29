/// MCP Server Module — zaos-ide MCP server
///
/// Phase 2: Implements a local MCP server (127.0.0.1:random port) to expose
/// tools to the Claude Code CLI:
/// - show_diff(path, before, after) — visual diff in dashboard
/// - notify(title, message) — system notification
/// - get_ui_state() — current UI state for CLI awareness
/// - capture_screenshot() — trigger GoPeak screenshot
///
/// This allows bidirectional communication between ZAOS frontend and the CLI:
/// CLI can request UI actions (diffs, notifications)
/// ZAOS can tell CLI about UI state (visible files, current section)
///
/// TODO: Phase 2 implementation
/// - [ ] Setup local TCP server on 127.0.0.1:random
/// - [ ] Generate ephemeral auth token (~/.claude/ide/)
/// - [ ] Implement MCP protocol handshake
/// - [ ] Define and expose tools
/// - [ ] Register in .mcp.json for CLI discovery
/// - [ ] Handle tool invocations from CLI
/// - [ ] Emit Tauri events for UI updates

pub mod server {
    pub fn start_mcp_server() {
        unimplemented!("Phase 2: MCP server implementation")
    }
}

pub use server::*;
