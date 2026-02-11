# httpkey-python (v0.5.0)

`httpkey-python` is a Model Context Protocol (MCP) server written in Python. It provides system utility tools to MCP clients (like Gemini) using the **HTTP (SSE) transport**.

This project allows LLMs to safely query local system information such as CPU usage, memory statistics, and disk space.

## Features

*   **MCP HTTP (SSE) Transport:** Communicates via JSON-RPC messages over Server-Sent Events.
*   **Modern Python:** Built with Python 3.11+ and the official `mcp` SDK using `FastMCP`.
*   **Direct CLI Mode:** Can be run directly from the command line for quick reports.

## Tools

The server exposes the following MCP tools:

1.  **`local_system_info`**: Generates a comprehensive system report.
    *   System Name, OS Name, Hostname
    *   CPU Core Count
    *   Memory Usage (Total, Used, Swap)
    *   Network Interface Statistics (RX/TX bytes and MAC addresses)
    *   Includes an authentication status header when run via MCP.

2.  **`disk_usage`**: Lists usage statistics for all mounted disks.
    *   Mount Point, File System
    *   Used/Total Space
    *   Percentage Used

## Prerequisites

*   Python 3.11+
*   `make` (optional, for convenience)
*   `gcloud` CLI (for authentication)

## Authentication

This server requires an **MCP API Key** for security.
The server will verify the provided key against the "MCP API Key" stored in Google Cloud API Keys. The project ID is automatically detected from the `GOOGLE_CLOUD_PROJECT` environment variable or your `gcloud` configuration.

When using HTTP (SSE) transport, the client must provide the API key in the `X-Goog-Api-Key` header.

You can provide the key to the server in two ways:
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
    "httpkey-python": {
      "url": "http://localhost:8080/sse",
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
make info KEY=YOUR_KEY
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

To start the server manually:

```bash
make run KEY=your-secret-key
# OR
export MCP_API_KEY=your-secret-key
python3 main.py
```

For production-like runs where the key is expected to be fetched automatically from the environment or Google Cloud:
```bash
make release
```

## Deployment

This project includes a `Dockerfile` and `cloudbuild.yaml` for easy deployment to **Google Cloud Run**.

1.  **Build and Deploy:**
    ```bash
    make deploy
    ```
    This command uses Google Cloud Build to build the container image and deploy it to Cloud Run.

2.  **Service Configuration:**
    The default service name is `sysutils-manual-python` (as defined in `cloudbuild.yaml`).

## Development

*   **Lint:** `make lint`
*   **Format:** `make fmt`
*   **Test:** `make test`
*   **Clean:** `make clean`