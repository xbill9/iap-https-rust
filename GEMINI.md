# Gemini Workspace for `iap-https-rust` (v0.2.0)

You are a Rust Developer working with Google Cloud.
You should follow Rust Best practices.
The recommended language level for Rust is 2024.

This document provides a developer-focused overview of the `iap-https-rust` project, tailored for use with Gemini.

## Project Overview

`iap-https-rust` is a Model Context Protocol (MCP) server written in Rust. It interacts via **streaming HTTP** to provide system utility tools to MCP clients. It is optimized for serverless environments like Google Cloud Run and supports Google Cloud Identity-Aware Proxy (IAP).

### Project Structure

This repository is split into three variants:
*   **`iap/`**: Standard implementation for IAP-protected Cloud Run services.
*   **`manual/`**: Adds an optional `MCP_API_KEY` check (via `x-goog-api-key` header) for additional security. Optimized for Cloud Run.
*   **`local/`**: API key support tailored for local development (no containerization).

### Key Technologies

*   **Language:** [Rust](https://www.rust-lang.org/) (Edition 2024)
*   **MCP SDK:** [rmcp](https://crates.io/crates/rmcp) (v0.14.0) - Uses `transport-streamable-http-server`.
*   **System Info:** [sysinfo](https://crates.io/crates/sysinfo) (v0.37.x)
*   **Async Runtime:** [Tokio](https://tokio.rs/)
*   **Web Framework:** [Axum](https://github.com/tokio-rs/axum)
*   **Serialization:** [Serde](https://serde.rs/) & [Schemars](https://crates.io/crates/schemars)
*   **Logging:** [Tracing](https://crates.io/crates/tracing) (JSON format to stdout)

## Architecture

Each variant (`iap/`, `manual/`, and `local/`) has its own `src/main.rs`:
*   `SysUtils` struct: Implements `ServerHandler` and `tool_router`.
*   **MCP Tools**:
    *   `iap/`: `iap_system_info`
    *   `manual/`: `sysutils_manual_rust`, `disk_usage`, `list_processes`
    *   `local/`: `local_system_info`, `disk_usage`
*   `collect_system_info`: Shared logic for system reports. Includes network interface info in `manual` and `local`.
*   **Security & Identity**:
    *   `iap_middleware`: Captures IAP JWT headers and validates API keys (if applicable).
    *   **API Key Fetching**: 
        *   `manual/`: Uses `google-apikeys2` and ADC to fetch "MCP API Key" programmatically.
        *   `local/`: Uses `gcloud` CLI to fetch "MCP API Key" for development.
*   `main`: Initializes the `StreamableHttpService`, sets up Axum with a `/health` route, and listens on `PORT`.

## Getting Started

Each subdirectory has its own `Makefile`.

### Environment Setup

*   `PORT`: Port for the HTTP server (default: 8080).
*   `RUST_LOG`: Logging level (default: `info,iap_https_rust=debug`, `info,manual_https_rust=debug`, or `info,sysutils_local_rust=debug`).
*   `MCP_API_KEY`: (Manual/Local variants only) Required API key for the `x-goog-api-key` header.

### Initial Build & Run

```bash
cd iap # or cd manual or cd local
make build
make run
```

### CLI Info command
```bash
cargo run -- info
```

## Development Workflow

### Code Quality
*   **Formatting:** `make fmt`
*   **Linting:** `make clippy`
*   **Checking:** `make check`

### Testing
```bash
make test
```
Tests include schema generation verification and basic tool functionality checks.

## Deployment

Deployment is handled via `cloudbuild.yaml` in the `iap/` and `manual/` directories. Note that the `local/` variant is not intended for Cloud Run deployment.

```bash
cd iap # or cd manual
make deploy
```

## Interacting with Gemini

You can use Gemini to help you with various tasks in this project. Relevant examples:

*   "Add a new tool to `SysUtils` in `iap/src/main.rs` that checks disk usage."
*   "Explain the difference between the `iap`, `manual`, and `local` variants."
*   "How does the `iap_middleware` in `local/src/main.rs` handle the API key check?"
*   "Modify `collect_system_info` in all variants to include network interface information."