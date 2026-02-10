# sysutils-local-rust

A Model Context Protocol (MCP) server written in Rust, optimized for local development and system monitoring. It provides system utility tools via streaming HTTP and includes built-in CLI commands for quick reports.

## Overview

This project implements an MCP server using the `rmcp` SDK with a `transport-streamable-http-server`. It is designed to be easily run locally while maintaining compatibility with Google Cloud Identity-Aware Proxy (IAP) patterns.

### Key Features

*   **MCP Tools**:
    *   `local_system_info`: Comprehensive system report (CPU, Memory, OS, Network, IAP context).
    *   `disk_usage`: Detailed usage statistics for all mounted disks.
*   **CLI Mode**: Run reports directly from the terminal without starting the HTTP server.
*   **Security**:
    *   Optional API key validation via `x-goog-api-key` header or `key` query parameter in the URL.
    *   Dual-stage API key fetching: Attempts to use `gcloud` CLI (optimized for local dev) with a library-based fallback using Application Default Credentials (ADC) to find an "MCP API Key" in Google Cloud.
    *   IAP JWT decoding for identity context.
*   **Monitoring**: 
    *   Integrated health check endpoint at `/health`.
    *   Automatic inclusion of local IAP configuration files (`iap_settings.yaml`, etc.) in system reports.

## Getting Started

### Prerequisites

*   [Rust](https://www.rust-lang.org/) (Edition 2024)
*   [gcloud CLI](https://cloud.google.com/sdk/gcloud) (Optional, for automatic API key fetching)

### Environment Variables

*   `PORT`: Port for the HTTP server (default: `8080`).
*   `RUST_LOG`: Logging level (default: `info,sysutils_local_rust=debug`).
*   `MCP_API_KEY`: (Optional) Required API key for the `x-goog-api-key` header. If not set, the server attempts to fetch it from Google Cloud API keys named "MCP API Key".

### Installation & Running

1.  **Build**:
    ```bash
    cargo build
    ```

2.  **Run Server**:
    ```bash
    make run
    # OR
    cargo run --release
    ```

3.  **CLI Commands**:
    ```bash
    cargo run -- info   # Display system report + verify API key status
    cargo run -- disk   # Display disk usage report
    ```

## Development

### Useful Commands

*   **Format code**: `make fmt`
*   **Lint code**: `make clippy`
*   **Run tests**: `make test`
*   **Check code**: `make check`

### Testing

The project includes unit tests for:
*   Schema generation verification.
*   System info and disk usage tool logic.
*   IAP JWT decoding.
*   Context propagation via task-local storage.

## License

This project is licensed under the MIT License.
