# Bearer Rust MCP Server

A Model Context Protocol (MCP) server implemented in Rust that provides system utility tools via streaming HTTP. This variant is designed for deployment on Google Cloud Run and utilizes identity-based authentication via Google Cloud IAP.

## Features

- **MCP Tools**:
  - `sysutils_bearer_rust`: Comprehensive system report (Kernel, CPU, Memory, Network).
  - `disk_usage`: Usage stats for all mounted disks.
  - `list_processes`: Top 20 processes by memory usage.
- **Security**:
  - Leverages Google Cloud IAP (Identity-Aware Proxy) for authentication.
  - Captures and decodes Google Cloud IAP JWT assertions (`x-goog-iap-jwt-assertion`) to provide identity context to tools.
- **Flexible Execution**:
  - Runs as a streaming HTTP server (compatible with MCP clients like Gemini).
  - Supports CLI mode for quick local reports.

## Prerequisites

- Rust (2024 edition)
- Google Cloud Project with Cloud Run and IAP enabled.

## Getting Started

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PORT` | Port for the HTTP server | `8080` |
| `RUST_LOG` | Logging level | `info,bearer_rust=debug` |

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

The project includes tests for schema generation, tool functionality, and JWT decoding.

```bash
cargo test
```

## Deployment

This service is designed to run on Google Cloud Run. Deployment is managed via Google Cloud Build.

```bash
make deploy
```

## Architecture

The server uses the `rmcp` SDK with `transport-streamable-http-server`. It leverages `axum` for the web layer and `sysinfo` for gathering system metrics. Security is implemented via a custom middleware that extracts identity context from IAP headers.

License: MIT
