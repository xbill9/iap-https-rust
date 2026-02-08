# stdiokey

`stdiokey` is a Model Context Protocol (MCP) server written in Rust. It provides system utility tools to MCP clients (like Gemini) using the **Stdio transport** (Standard Input/Output).

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

## Authentication

This server requires an **MCP API Key** for security, even when running locally via Stdio.
The server will verify the provided key against the "MCP API Key" stored in Google Cloud API Keys (Project ID: `1056842563084`).

You can provide the key in two ways:
1.  Using the `--key` argument when starting the server.
2.  Setting the `MCP_API_KEY` environment variable.

The server uses a two-stage verification process:
*   It attempts to fetch the valid key using the `gcloud` CLI (ideal for local development).
*   It falls back to the Google Cloud API Keys library using Application Default Credentials (ADC).

## Installation & Build

Clone the repository and build the project:

```bash
cargo build --release
```

## Usage

### 1. As an MCP Server (with Gemini)

This project is configured for use with the Gemini CLI. The configuration is located in `.gemini/settings.json`.
**Note:** You must provide your API key either via `args` or `env`:

```json
{
  "mcpServers": {
    "stdiokey": {
      "command": "cargo",
      "args": ["run", "--quiet", "--release", "--", "--key", "YOUR_SECRET_KEY"],
      "env": {
        "RUST_LOG": "info,stdiokey=debug",
        "MCP_API_KEY": "YOUR_SECRET_KEY"
      }
    }
  }
}
```

### 2. Direct CLI Usage

You can run the tools directly without an MCP client. These commands will also display the verification status of your API key if provided:

**System Info:**
```bash
make info
# OR
cargo run --quiet -- info --key YOUR_KEY
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
make run KEY=your-secret-key
# OR
export MCP_API_KEY=your-secret-key
cargo run --release
```

## Development

*   **Check:** `make check`
*   **Test:** `make test`
*   **Format:** `make fmt`
*   **Lint:** `make clippy`