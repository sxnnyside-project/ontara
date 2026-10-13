//! The local MCP server: `ontara-mcp <project folder>`, speaking JSON-RPC on standard input and
//! output so an assistant on this computer can read the universe and propose changes.

fn main() -> std::process::ExitCode {
    app_lib::run_mcp(&std::env::args().skip(1).collect::<Vec<_>>())
}
