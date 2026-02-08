# Gemini Workspace for `stdiokey` (v0.2.0)

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for Rust is 2024.

This document provides a developer-focused overview of the `stdiokey` (Stdio variant), tailored for use with Gemini.

## Project Overview

This variant of `iap-https-rust` is refactored to use the **Stdio MCP transport**. It provides system utilities via standard input/output, suitable for local integration with MCP clients.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **MCP SDK:** [rmcp](https://crates.io/crates/rmcp) (v0.14.0) - Uses `transport-io` (Stdio).
*   **System Info:** [sysinfo](https://crates.io/crates/sysinfo) (v0.37.x)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Web Framework:** [Hyper](https://hyper.rs/) & [Hyper-util](https://crates.io/crates/hyper-util) (for Auth)
*   **Auth:** [google-apikeys2](https://crates.io/crates/google-apikeys2) & [yup-oauth2](https://crates.io/crates/yup-oauth2)
*   **Logging:** [Tracing](https://crates.io/crates/tracing) (JSON format to stderr)

## Architecture

*   **`src/main.rs`**: Single entry point. 
    *   `SysUtils` struct: Implements `ServerHandler` and `tool_router`.
    *   **MCP Tools**:
        *   `local_system_info`: Comprehensive system report including system metrics.
        *   `disk_usage`: Disk usage information for all mounted disks.
    *   `collect_system_info`: Shared logic for system reports. Captures system metrics (CPU, Memory, OS version, Network interfaces).
    *   **Security & Identity**:
        *   **Two-Stage Key Fetching**: 
            1.  Attempts to use `gcloud services api-keys` CLI to fetch the "MCP API Key" (optimized for local dev with User ADC).
            2.  Falls back to `google-apikeys2` Rust library using Application Default Credentials (ADC).
        *   Validates provided key (via `--key` or `MCP_API_KEY` env) against the fetched cloud key.
        *   Project ID: `1056842563084`.
    *   `main`: 
        *   Handles `info` and `disk` CLI commands for direct output.
        *   Performs API Key validation (exits if invalid or missing).
        *   Initializes `SysUtils` service with `LazyLock` schema generation.
        *   Starts the MCP server using `transport::stdio`.
        *   **Logging**: Uses `tracing` with JSON format, explicitly directed to `stderr` to prevent JSON-RPC interference on `stdout`.

## Getting Started

### Initial Build & Run

1.  **Build:** `cargo build`
2.  **Run Server:** 
    *   **Via Cargo:** `cargo run -- --key <YOUR_API_KEY>` or `MCP_API_KEY=<KEY> cargo run`
    *   **Via Make:** `make run KEY=<YOUR_API_KEY>`
3.  **CLI Commands:**
    *   `cargo run -- info`: Display system information and API key verification status.
    *   `cargo run -- disk`: Display disk usage report directly.

## Development Workflow

*   **Formatting:** `make fmt`
*   **Linting:** `make clippy`
*   **Testing:** `make test`
