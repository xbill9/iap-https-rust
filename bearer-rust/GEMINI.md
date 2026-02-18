# Gemini Workspace for `proxy-rust` (v0.3.0)

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for Rust is 2024.

This document provides a developer-focused overview of the `proxy-rust` (Manual variant), tailored for use with Gemini.

## Project Overview

This variant of `iap-https-rust` adds an explicit API key check to the MCP server. It is ideal for environments where you want an additional layer of security beyond IAP or for testing purposes. It features automated API key fetching from Google Cloud API Keys service using Application Default Credentials (ADC).

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **MCP SDK:** [rmcp](https://crates.io/crates/rmcp) (v0.14.0) - Uses `transport-streamable-http-server`.
*   **System Info:** [sysinfo](https://crates.io/crates/sysinfo) (v0.37.2)
*   **Async Runtime:** [Tokio](https://tokio.rs/) (v1.x)
*   **Web Framework:** [Axum](https://github.com/tokio-rs/axum) (v0.8.x)
*   **Security:** `google-apikeys2`, `yup-oauth2` (for ADC-based API key fetching)
*   **Serialization:** [Serde](https://serde.rs/) & [Schemars](https://crates.io/crates/schemars)
*   **Logging:** [Tracing](https://crates.io/crates/tracing) (JSON format to stderr, sent to Cloud Logging)

## Architecture

*   **`src/main.rs`**: Single entry point. 
    *   `SysUtils` struct: Implements `ServerHandler` and `tool_router`.
    *   **MCP Tools**:
        *   `sysutils_proxy_rust`: Detailed system info (kernel, CPU, memory, network).
        *   `disk_usage`: Disk usage information for all mounted disks.
        *   `list_processes`: Top 20 running processes by memory usage.
    *   `fetch_mcp_api_key`: Automatically fetches the API key named "MCP API Key" from the Google Cloud project `1056842563084` using ADC.
    *   `iap_middleware`: Captures IAP JWT *and* validates the `x-goog-api-key` header against the `MCP_API_KEY` environment variable (or the automatically fetched key).
    *   `main`: Initializes the `StreamableHttpService` with `LocalSessionManager`, sets up Axum with a `/health` route, and applies the security middleware.

## Getting Started

### Environment Setup

*   `PORT`: Port for the HTTP server (default: 8080).
*   `RUST_LOG`: Logging level (default: `info,proxy_rust=debug`).

### Initial Build & Run

1.  **Build:** `cargo build`
2.  **Run:** `make run`
3.  **CLI Commands:**
    *   `cargo run -- info`: Display system information report.
    *   `cargo run -- disk`: Display disk usage report.
    *   `cargo run -- processes`: Display process list report.

## Development Workflow

*   **Formatting:** `make fmt`
*   **Linting:** `make clippy`
*   **Testing:** `make test`

## Deployment

Deployment configuration is managed via `cloudbuild.yaml`.

```bash
make deploy
```