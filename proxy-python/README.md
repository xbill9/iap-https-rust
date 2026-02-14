# proxy-python (v0.5.0)

`proxy-python` is a Model Context Protocol (MCP) server written in Python. It provides system utility tools to MCP clients (like Gemini) using the **Streaming HTTP transport**.

This project allows LLMs to safely query local system information such as CPU usage, memory statistics, and disk space.

## Features

*   **MCP Transports:** Supports **Streaming HTTP** (default), **SSE**, and **Stdio** transports.
*   **Modern Python:** Built with Python 3.11+ and the official `mcp` SDK using `FastMCP`.
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

## Installation

Clone the repository and install dependencies:

```bash
make install
```

## Usage

### 1. As an MCP Server (with Gemini)

This project is configured for use with the Gemini CLI. The configuration is located in `.gemini/settings.json`.

```json
{
  "mcpServers": {
    "proxy-python": {
      "url": "http://localhost:8080/mcp"
    }
  }
}
```

### 2. Direct CLI Usage

You can run the tools directly without an MCP client:

**System Info:**
```bash
make info
# OR
python3 main.py info
```

**Disk Usage:**
```bash
make disk
# OR
python3 main.py disk
```

### 3. Manual MCP Server Run

To start the server manually with the default (HTTP) transport:

```bash
make run
# OR
python3 main.py
```

To use **SSE** transport:
```bash
MCP_TRANSPORT=sse PORT=8080 python3 main.py
```

To use **Stdio** transport:
```bash
MCP_TRANSPORT=stdio python3 main.py
```

## Deployment

This project includes a `Dockerfile` and `cloudbuild.yaml` for easy deployment to **Google Cloud Run**.

1.  **Build and Deploy:**
    ```bash
    make deploy
    ```
    This command uses Google Cloud Build to build the container image and deploy it to Cloud Run.

2.  **Service Configuration:**
    The default service name is `sysutils-proxy-python` (as defined in `cloudbuild.yaml`). It deploys to `us-central1` by default.

## Development

*   **Lint:** `make lint`
*   **Format:** `make fmt`
*   **Test:** `make test`
*   **Clean:** `make clean`
