# Gemini Workspace for `proxy-python` (v0.5.0)

You are a Python Developer working with Google Cloud.
You should follow Python best practices (PEP 8) and use modern Python features (3.11+).

This document provides a developer-focused overview of the `proxy-python` (HTTP variant), tailored for use with Gemini.

## Project Overview

This project is a Model Context Protocol (MCP) server written in Python. It provides system utilities via Streaming HTTP, suitable for integration with MCP clients like Gemini.

### Key Technologies

*   **Language:** [Python](https://www.python.org/) (>=3.11)
*   **MCP SDK:** [mcp](https://pypi.org/project/mcp/) (v1.2.x) - Uses `mcp.server.fastmcp`.
*   **Transport:** Streaming HTTP, SSE, and Stdio
*   **System Info:** [psutil](https://pypi.org/project/psutil/) (v6.1.x)
*   **Async Runtime:** [asyncio](https://docs.python.org/3/library/asyncio.html)
*   **Formatting/Linting:** [Ruff](https://beta.ruff.rs/docs/)

## Architecture

*   **`main.py`**: Single entry point.
    *   **MCP Tools (FastMCP)**:
        *   `local_system_info`: Comprehensive system report including CPU, memory, and network interfaces.
        *   `disk_usage`: Disk usage information for all mounted disks.
    *   `collect_system_info`: Shared logic for system reports. Captures system metrics (CPU, Memory, OS version, Network interfaces).
*   **Project ID**: Fetched from the environment (`GOOGLE_CLOUD_PROJECT`) or `gcloud` configuration if needed.
*   `main`: 
        *   Handles `info` and `disk` CLI commands for direct output.
        *   Starts the MCP server using `mcp.run(transport="http")` (via `streamable_http_app`), `sse` (via `sse_app`), or fallback to `stdio`.
        *   **Logging**: Uses `logging` explicitly directed to `stderr` to avoid interfering with MCP JSON-RPC messages (if in stdio mode).

## Getting Started

### Initial Build & Run

1.  **Install:** `pip install .` or `make install`
2.  **Run Server:** 
    *   **HTTP (Default):** `MCP_TRANSPORT=http PORT=8080 python3 main.py` or `make run`
    *   **SSE:** `MCP_TRANSPORT=sse PORT=8080 python3 main.py`
    *   **Stdio:** `MCP_TRANSPORT=stdio python3 main.py`
3.  **CLI Commands:**
    *   `python3 main.py info`: Display system information.
    *   `python3 main.py disk`: Display disk usage report directly.

## Development Workflow

*   **Formatting:** `make fmt` (runs `ruff format`)
*   **Linting:** `make lint` (runs `ruff check`)
*   **Test:** `make test` (runs `unittest`)

## Deployment

The project is configured for deployment to **Google Cloud Run** via **Cloud Build**.

*   **Dockerfile**: Uses `python:3.12-slim`, installs the package, and exposes port 8080.
*   **cloudbuild.yaml**:
    *   Builds the Docker image and pushes it to GCR.
    *   Deploys to Cloud Run in `us-central1`.
*   **Command**: `make deploy`
