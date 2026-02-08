# sysutils-stdio-rust

`sysutils-stdio-rust` is a Model Context Protocol (MCP) server written in Rust. It provides system utility tools to MCP clients (like Gemini) using the **Stdio transport** (Standard Input/Output).

This project allows LLMs to safely query local system information such as CPU usage, memory statistics, and disk space.

## Features

*   **MCP Stdio Transport:** Communicates via JSON-RPC messages over stdin/stdout.
*   **Performance:** Built with Rust and Tokio for efficiency.
*   **Direct CLI Mode:** Can be run directly from the command line for quick reports.

## Tools

The server exposes the following MCP tools:

1.  **`local_system_info`**: Generates a comprehensive system report.
    *   System Name, Kernel, OS Version, Hostname
    *   CPU Core Count
    *   Memory Usage (Total, Used, Swap)
    *   Network Interface Statistics (RX/TX bytes)

2.  **`disk_usage`**: Lists usage statistics for all mounted disks.
    *   Mount Point, File System
    *   Used/Total Space
    *   Percentage Used

## Prerequisites

*   [Rust](https://www.rust-lang.org/tools/install) (Edition 2024 compatible)
*   `make` (optional, for convenience)

## Installation & Build

Clone the repository and build the project:

```bash
cargo build --release
```

## Usage

### 1. As an MCP Server (with Gemini)

This project is configured for use with the Gemini CLI. The configuration is located in `.gemini/settings.json`:

```json
{
  "mcpServers": {
    "sysutils-stdio-rust": {
      "command": "cargo",
      "args": ["run", "--quiet", "--release"],
      "env": {
        "RUST_LOG": "info,sysutils_stdio_rust=debug"
      }
    }
  }
}
```

When you start a session with Gemini, this server will automatically start, and the tools will be available to the model.

### 2. Direct CLI Usage

You can run the tools directly without an MCP client for debugging or quick checks:

**System Info:**
```bash
make info
# OR
cargo run --quiet -- info
```

**Disk Usage:**
```bash
make disk
# OR
cargo run --quiet -- disk
```

### 3. Manual MCP Server Run

To start the server manually (it will wait for JSON-RPC input on stdin):

```bash
make run
# OR
cargo run --release
```

## Development

*   **Check:** `make check`
*   **Test:** `make test`
*   **Format:** `make fmt`
*   **Lint:** `make clippy`