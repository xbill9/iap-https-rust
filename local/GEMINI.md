# Gemini Workspace for `sysutils-local-rust` (v0.2.0)

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for Rust is 2024.

This document provides a developer-focused overview of the `sysutils-local-rust` (Manual variant), tailored for use with Gemini.

## Project Overview

This variant of `iap-https-rust` adds an explicit API key check to the MCP server. It is ideal for environments where you want an additional layer of security beyond IAP or for testing purposes.

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **MCP SDK:** [rmcp](https://crates.io/crates/rmcp) (v0.14.0)
*   **System Info:** [sysinfo](https://crates.io/crates/sysinfo) (v0.37.x)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
*   **Logging:** [Tracing](https://crates.io/crates/tracing) (JSON format to stdout)

## Architecture

*   **`src/main.rs`**: Single entry point. 
    *   `SysUtils` struct: Implements `ServerHandler` and `tool_router`.
    *   `iap_system_info`: The primary MCP tool.
    *   `collect_system_info`: Shared logic for both MCP tool and CLI `info` command.
    *   `iap_middleware`: Captures IAP JWT *and* validates the `x-goog-api-key` header against the `MCP_API_KEY` environment variable.
    *   `main`: Initializes the `StreamableHttpService`, sets up Axum with a `/health` route and security middleware.

## Getting Started

### Environment Setup

*   `PORT`: Port for the HTTP server (default: 8080).
*   `RUST_LOG`: Logging level (default: `info,sysutils_local_rust=debug`).
*   `MCP_API_KEY`: **Required** if you want to enable API key validation.

### Initial Build & Run

1.  **Build:** `cargo build`
2.  **Run:** `MCP_API_KEY=my-key make run`
3.  **CLI Info:** `cargo run -- info`

## Development Workflow

*   **Formatting:** `make fmt`
*   **Linting:** `make clippy`
*   **Testing:** `make test`