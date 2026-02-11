# Gemini Workspace for `httpkey-python` (v0.5.0)

You are a Python Developer working with Google Cloud.
You should follow Python best practices (PEP 8) and use modern Python features (3.11+).

This document provides a developer-focused overview of the `httpkey-python` (HTTP variant), tailored for use with Gemini.

## Project Overview

This project is a Model Context Protocol (MCP) server written in Python. It provides system utilities via HTTP (SSE), suitable for integration with MCP clients like Gemini.

### Key Technologies

*   **Language:** [Python](https://www.python.org/) (>=3.11)
*   **MCP SDK:** [mcp](https://pypi.org/project/mcp/) (v1.2.x) - Uses `mcp.server.fastmcp`.
*   **Transport:** SSE (via `starlette` and `uvicorn`)
*   **System Info:** [psutil](https://pypi.org/project/psutil/) (v6.1.x)
*   **Async Runtime:** [asyncio](https://docs.python.org/3/library/asyncio.html)
*   **Validation:** [Pydantic](https://docs.pydantic.io/) (v2.10.x)
*   **Auth:** `google-api-python-client` & `google-auth`
*   **Formatting/Linting:** [Ruff](https://beta.ruff.rs/docs/)

## Architecture

*   **`main.py`**: Single entry point.
    *   **MCP Tools (FastMCP)**:
        *   `local_system_info`: Comprehensive system report including CPU, memory, and network interfaces.
        *   `disk_usage`: Disk usage information for all mounted disks.
    *   `collect_system_info`: Shared logic for system reports. Captures system metrics (CPU, Memory, OS version, Network interfaces).
    *   **Security & Identity**:
        *   **Two-Stage Key Fetching**: 
            1.  Attempts to use `gcloud services api-keys` CLI to fetch the "MCP API Key" (optimized for local dev with User ADC).
            2.  Falls back to `google-api-python-client` Python library using Application Default Credentials (ADC).
        *   **Middleware**: When using SSE transport, an HTTP middleware validates the `X-Goog-Api-Key` header against the expected key.
        *   Validates provided key (via `--key` or `MCP_API_KEY` env) against the fetched cloud key.
*   **Project ID**: Fetched from the environment (`GOOGLE_CLOUD_PROJECT`) or `gcloud` configuration.
*   `main`: 
        *   Handles `info` and `disk` CLI commands for direct output. `info` requires a valid API key (provided via `--key` or env) to proceed, performing a live verification against Google Cloud.
        *   Performs API Key validation (exits if invalid or missing) before starting the server if a key is provided or found in the cloud.
        *   Starts the MCP server using `mcp.run(transport="sse")` or fallback to `stdio`.
        *   **Logging**: Uses `logging` explicitly directed to `stderr` to avoid interfering with MCP JSON-RPC messages (if in stdio mode).

## Getting Started

### Initial Build & Run

1.  **Install:** `pip install .` or `make install`
2.  **Run Server:** 
    *   **Via Python:** `MCP_TRANSPORT=sse PORT=8080 python3 main.py --key <YOUR_API_KEY>`
    *   **Via Make:** `make run KEY=<YOUR_API_KEY>`
3.  **CLI Commands:**
    *   `python3 main.py info`: Display system information and API key verification status.
    *   `python3 main.py disk`: Display disk usage report directly.

## Development Workflow

*   **Formatting:** `make fmt` (runs `ruff format`)
*   **Linting:** `make lint` (runs `ruff check`)
*   **Testing:** `make test` (runs `unittest`)

## Deployment

The project is configured for deployment to **Google Cloud Run** via **Cloud Build**.

*   **Dockerfile**: Uses `python:3.12-slim`, installs the package, and exposes port 8080.
*   **cloudbuild.yaml**:
    *   Builds the Docker image and pushes it to GCR.
    *   Deploys to Cloud Run in `us-central1`.
    *   Sets environment variables for Vertex AI and Project ID.
*   **Command**: `make deploy`
