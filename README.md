# iap-https-rust (v0.2.0)

A Rust-based [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that provides system utility tools. This application runs over **streaming HTTP** and is optimized for deployment on Google Cloud Run.

## Project Structure

This repository contains two variants of the MCP server:

1.  **`iap/`**: The standard version designed for use with Google Cloud Identity-Aware Proxy (IAP). It relies on IAP to handle authentication and decodes the `x-goog-iap-jwt-assertion` header to provide identity context.
2.  **`manual/`**: An enhanced version that adds a manual API key check. It looks for the `MCP_API_KEY` environment variable; if set, it validates the `x-goog-api-key` header on incoming requests.

Both variants provide the same set of system utility tools.

## Features

*   **MCP Protocol Support**: Implements the Model Context Protocol over streaming HTTP using the `rmcp` library.
*   **System Information**: Provides a detailed report of the host system.
    *   **Tool**: `iap_system_info`
    *   **Collected Data**: 
        *   **IAP Context**: Decodes identity from IAP JWT.
        *   **System**: Name, Kernel version, OS version, Host name.
        *   **CPU**: Number of cores.
        *   **Memory**: Total/Used RAM and Total/Used Swap.
*   **Logging**: Structured JSON logging to `stdout` using `tracing-subscriber`.
*   **CLI Mode**: Direct execution to print system information without starting the MCP server.
*   **Health Check**: Includes a `/health` endpoint for monitoring.

## Getting Started

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install) (Edition 2024)
*   [Make](https://www.gnu.org/software/make/)

### Build & Run

Each variant has its own directory and `Makefile`. To build or run a specific version, navigate to its directory:

#### IAP Version
```bash
cd iap
make build
make run
```

#### Manual Version
```bash
cd manual
make build
MCP_API_KEY=your-secret-key make run
```

### CLI Usage (Direct Info)

You can run the system information tool directly from the command line in either directory:

```bash
cargo run -- info
```

## Development

The root directory contains a `Makefile` that can be used to clean all sub-projects. Individual development tasks should be performed within the `iap/` or `manual/` directories.

*   **Format Code**: `make fmt`
*   **Lint Code**: `make clippy`
*   **Run Tests**: `make test`

## Deployment

Both variants are containerized and ready for deployment to Google Cloud Run via Google Cloud Build.

```bash
cd iap # or cd manual
make deploy
```

The `Dockerfile` in each directory uses a multi-stage build and targets a distroless runtime for security.
