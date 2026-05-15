//! Phase 1 stub for the `x2p-agent-server` binary.
//!
//! The full agent server (MCP tool surface plus a loopback JSON-RPC fallback)
//! is a Phase 2 deliverable. To keep the Phase 1
//! release coherent, the binary exists in the workspace but immediately
//! reports `not_implemented` and exits with status `1`. The CLI's `serve`
//! subcommand delegates to this binary and surfaces the same error to users
//! (see task 15.1).

fn main() -> ! {
    // Use `println!` per the task spec: the message goes to stdout so callers
    // (including the CLI's `serve` subcommand) can pipe and grep for it
    // unambiguously, while the non-zero exit code still signals failure.
    println!("not_implemented");
    std::process::exit(1);
}
