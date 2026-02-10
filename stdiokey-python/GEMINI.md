# Gemini Workspace for `stdiokey-python` (v0.4.0)

You are a Python Developer working with Google Cloud.
You should follow Python best practices (PEP 8) and use modern Python features (3.11+).

This document provides a developer-focused overview of the `stdiokey-python` (Stdio variant), tailored for use with Gemini.

## Project Overview

This project is a Model Context Protocol (MCP) server written in Python. It provides system utilities via standard input/output (Stdio), suitable for local integration with MCP clients like Gemini.

### Key Technologies

*   **Language:** [Python](https://www.python.org/) (>=3.11)
*   **MCP SDK:** [mcp](https://pypi.org/project/mcp/) (v1.2.x) - Uses `mcp.server.stdio`.
*   **System Info:** [psutil](https://pypi.org/project/psutil/) (v6.1.x)
*   **Async Runtime:** [asyncio](https://docs.python.org/3/library/asyncio.html)
*   **Validation:** [Pydantic](https://docs.pydantic.io/) (v2.10.x)
*   **Auth:** `google-api-python-client` & `google-auth`
*   **Formatting/Linting:** [Ruff](https://beta.ruff.rs/docs/)

## Architecture

*   **`main.py`**: Single entry point.
    *   **MCP Tools**:
        *   `local_system_info`: Comprehensive system report including CPU, memory, and network interfaces.
        *   `disk_usage`: Disk usage information for all mounted disks.
    *   `collect_system_info`: Shared logic for system reports. Captures system metrics (CPU, Memory, OS version, Network interfaces).
    *   **Security & Identity**:
        *   **Two-Stage Key Fetching**: 
            1.  Attempts to use `gcloud services api-keys` CLI to fetch the "MCP API Key" (optimized for local dev with User ADC).
            2.  Falls back to `google-api-python-client` Python library using Application Default Credentials (ADC).
        *   Validates provided key (via `--key` or `MCP_API_KEY` env) against the fetched cloud key.
*   **Project ID**: Fetched from the environment (`GOOGLE_CLOUD_PROJECT`) or `gcloud` configuration.
*   `main`: 
        *   Handles `info` and `disk` CLI commands for direct output.
        *   Performs API Key validation (exits if invalid or missing) before starting the server.
        *   Starts the MCP server using `stdio_server`.
        *   **Logging**: Uses `logging` explicitly directed to `stderr` to prevent JSON-RPC interference on `stdout`.

## Getting Started

### Initial Build & Run

1.  **Install:** `pip install .` or `make install`
2.  **Run Server:** 
    *   **Via Python:** `python3 main.py --key <YOUR_API_KEY>` or `MCP_API_KEY=<KEY> python3 main.py`
    *   **Via Make:** `make run KEY=<YOUR_API_KEY>`
3.  **CLI Commands:**
    *   `python3 main.py info`: Display system information and API key verification status.
    *   `python3 main.py disk`: Display disk usage report directly.

## Development Workflow

*   **Formatting:** `make fmt` (runs `ruff format`)
*   **Linting:** `make lint` (runs `ruff check`)
*   **Testing:** `make test` (runs `unittest`)