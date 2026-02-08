# Gemini Workspace for `sysutils-stdio-rust` (v0.2.0)

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for Rust is 2024.

This document provides a developer-focused overview of the `sysutils-stdio-rust` (Stdio variant), tailored for use with Gemini.

## Project Overview

This variant of `iap-https-rust` is refactored to use the **Stdio MCP transport**. It provides system utilities via standard input/output, suitable for local integration with MCP clients.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **MCP SDK:** [rmcp](https://crates.io/crates/rmcp) (v0.14.0) - Uses `transport-io` (Stdio).
*   **System Info:** [sysinfo](https://crates.io/crates/sysinfo) (v0.37.x)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Logging:** [Tracing](https://crates.io/crates/tracing) (JSON format to stderr)

## Architecture

*   **`src/main.rs`**: Single entry point. 
    *   `SysUtils` struct: Implements `ServerHandler` and `tool_router`.
    *   **MCP Tools**:
        *   `local_system_info`: Comprehensive system report including system metrics.
        *   `disk_usage`: Disk usage information for all mounted disks.
    *   `collect_system_info`: Shared logic for system reports. Captures system metrics (CPU, Memory, OS version, Network interfaces, MAC addresses).
    *   `main`: 
        *   Handles `info` and `disk` CLI commands for direct output.
        *   Initializes `SysUtils` service.
        *   Starts the MCP server using `transport::stdio`.
        *   Logs to stderr to avoid interfering with stdout JSON-RPC.

## Getting Started

### Initial Build & Run

1.  **Build:** `cargo build`
2.  **Run Server:** `cargo run` (Starts MCP server on Stdio)
3.  **CLI Commands:**
    *   `cargo run -- info`: Display system information report directly.
    *   `cargo run -- disk`: Display disk usage report directly.

## Development Workflow

*   **Formatting:** `make fmt`
*   **Linting:** `make clippy`
*   **Testing:** `make test`
