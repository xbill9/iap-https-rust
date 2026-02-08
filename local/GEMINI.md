# Gemini Workspace for `sysutils-local-rust` (v0.2.0)

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for Rust is 2024.

This document provides a developer-focused overview of the `sysutils-local-rust` (Local variant), tailored for use with Gemini.

## Project Overview

This variant of `iap-https-rust` is optimized for local development. It supports optional API key validation and captures Google Cloud Identity-Aware Proxy (IAP) context if present.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **MCP SDK:** [rmcp](https://crates.io/crates/rmcp) (v0.14.0) - Uses `transport-streamable-http-server` with `LocalSessionManager`.
*   **System Info:** [sysinfo](https://crates.io/crates/sysinfo) (v0.37.x)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Web Framework:** [Axum](https://github.com/tokio-rs/axum) (v0.8.x)
*   **Logging:** [Tracing](https://crates.io/crates/tracing) (JSON format to stderr)

## Architecture

*   **`src/main.rs`**: Single entry point. 
    *   `SysUtils` struct: Implements `ServerHandler` and `tool_router`.
    *   **MCP Tools**:
        *   `local_system_info`: Comprehensive system report including IAP context and system metrics.
        *   `disk_usage`: Disk usage information for all mounted disks.
    *   `collect_system_info`: Shared logic for system reports. Captures:
        *   IAP JWT Claims (from `x-goog-iap-jwt-assertion`)
        *   HTTP Request Headers
        *   System metrics (CPU, Memory, OS version, Network interfaces)
    *   `iap_middleware`: 
        *   Validates `x-goog-api-key` header against `MCP_API_KEY` environment variable (if set).
        *   Decodes IAP JWT assertions.
        *   Populates `tokio::task_local` storage for `IAP_CONTEXT` and `REQUEST_HEADERS`.
    *   `main`: 
        *   Handles `info` CLI command.
        *   Initializes `StreamableHttpService`.
        *   Sets up Axum with a `/health` route and security middleware.
        *   Listens on `PORT` (default 8080).

## Getting Started

### Environment Setup

*   `PORT`: Port for the HTTP server (default: 8080).
*   `RUST_LOG`: Logging level (default: `info,sysutils_local_rust=debug`).
*   `MCP_API_KEY`: (Optional) Required API key for the `x-goog-api-key` header.

### Initial Build & Run

1.  **Build:** `cargo build`
2.  **Run Server:** `MCP_API_KEY=my-key make run`
3.  **CLI Commands:**
    *   `cargo run -- info`: Display system information report.
    *   `cargo run -- disk`: Display disk usage report.

## Development Workflow

*   **Formatting:** `make fmt`
*   **Linting:** `make clippy`
*   **Testing:** `make test`