# stdiokey-python (v0.4.0)

`stdiokey-python` is a Model Context Protocol (MCP) server written in Python. It provides system utility tools to MCP clients (like Gemini) using the **Stdio transport** (Standard Input/Output).

This project allows LLMs to safely query local system information such as CPU usage, memory statistics, and disk space.

## Features

*   **MCP Stdio Transport:** Communicates via JSON-RPC messages over stdin/stdout.
*   **Modern Python:** Built with Python 3.11+ and the official `mcp` SDK.
*   **Direct CLI Mode:** Can be run directly from the command line for quick reports.

## Tools

The server exposes the following MCP tools:

1.  **`local_system_info`**: Generates a comprehensive system report.
    *   System Name, OS Name, Hostname
    *   CPU Core Count
    *   Memory Usage (Total, Used, Swap)
    *   Network Interface Statistics (RX/TX bytes and MAC addresses)

2.  **`disk_usage`**: Lists usage statistics for all mounted disks.
    *   Mount Point, File System
    *   Used/Total Space
    *   Percentage Used

## Prerequisites

*   Python 3.11+
*   `make` (optional, for convenience)
*   `gcloud` CLI (for authentication)

## Authentication

This server requires an **MCP API Key** for security, even when running locally via Stdio.
The server will verify the provided key against the "MCP API Key" stored in Google Cloud API Keys. The project ID is automatically detected from the `GOOGLE_CLOUD_PROJECT` environment variable or your `gcloud` configuration.

You can provide the key in two ways:
1.  Using the `--key` argument when starting the server.
2.  Setting the `MCP_API_KEY` environment variable.

The server uses a two-stage verification process:
*   It attempts to fetch the valid key using the `gcloud` CLI (ideal for local development).
*   It falls back to the Google Cloud API Keys library using Application Default Credentials (ADC).

## Installation

Clone the repository and install dependencies:

```bash
make install
```

## Usage

### 1. As an MCP Server (with Gemini)

This project is configured for use with the Gemini CLI. The configuration is located in `.gemini/settings.json`.
**Note:** You must provide your API key either via `args` or `env`:

```json
{
  "mcpServers": {
    "stdiokey-python": {
      "command": "python3",
      "args": ["main.py", "--key", "YOUR_SECRET_KEY"],
      "env": {
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
python3 main.py info --key YOUR_KEY
```

**Disk Usage:**
```bash
make disk
# OR
python3 main.py disk
```

### 3. Manual MCP Server Run

To start the server manually (it will wait for JSON-RPC input on stdin):

```bash
make run KEY=your-secret-key
# OR
export MCP_API_KEY=your-secret-key
python3 main.py
```

## Development

*   **Lint:** `make lint`
*   **Format:** `make fmt`
*   **Test:** `make test`
*   **Clean:** `make clean`
