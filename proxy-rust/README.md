# Proxy Rust MCP Server

A Model Context Protocol (MCP) server implemented in Rust that provides system utility tools via streaming HTTP. This variant includes manual API key validation and automated key fetching from Google Cloud.

## Features

- **MCP Tools**:
  - `sysutils_proxy_rust`: Comprehensive system report (Kernel, CPU, Memory, Network).
  - `disk_usage`: Usage stats for all mounted disks.
  - `list_processes`: Top 20 processes by memory usage.
- **Security**:
  - Validates `x-goog-api-key` header against a required API key.
  - Captures and logs Google Cloud IAP JWT assertions.
- **Automated API Key Management**:
  - Automatically fetches the API key named "MCP API Key" from Google Cloud API Keys service using Application Default Credentials (ADC).
- **Flexible Execution**:
  - Runs as a streaming HTTP server (compatible with MCP clients like Gemini).
  - Supports CLI mode for quick local reports.

## Prerequisites

- Rust (2024 edition)
- Google Cloud Project with API Keys API enabled (if using automated fetching).
- Application Default Credentials (ADC) configured.

## Getting Started

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Port for the HTTP server | `8080` |
| `RUST_LOG` | Logging level | `info,proxy_rust=debug` |

### Installation

```bash
cargo build --release
```

### Running the Server

```bash
# Using Makefile
make run

# Using Cargo
cargo run
```

### CLI Mode

Generate reports directly in your terminal:

```bash
cargo run -- info
cargo run -- disk
cargo run -- processes
```

## Development

### Useful Commands

- **Format code**: `make fmt`
- **Lint code**: `make clippy`
- **Run tests**: `make test`
- **Check types**: `make check`

### Testing

The project includes tests for schema generation and tool functionality.

```bash
cargo test
```

## Deployment

This service is designed to run on Google Cloud Run. Deployment is managed via Google Cloud Build.

```bash
make deploy
```

## Architecture

The server uses the `rmcp` SDK with `transport-streamable-http-server`. It leverages `axum` for the web layer and `sysinfo` for gathering system metrics. Security is implemented via a custom middleware that checks for the `x-goog-api-key` header.

License: MIT
