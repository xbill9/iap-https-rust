# iap-https-rust (v0.2.0)

A Rust-based [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server that provides system utility tools. This application runs over **streaming HTTP** and is optimized for deployment on Google Cloud Run.

## Project Structure

This repository contains three variants of the MCP server:

1.  **`iap/`**: The standard version designed for use with Google Cloud Identity-Aware Proxy (IAP). It relies on IAP to handle authentication and decodes the `x-goog-iap-jwt-assertion` header to provide identity context.
2.  **`manual/`**: An enhanced version that adds a manual API key check and advanced tools. It automatically fetches an API key named "MCP API Key" from your Google Cloud project using Application Default Credentials (ADC). Optimized for Cloud Run deployment.
3.  **`local/`**: Tailored for local development. It uses `gcloud` commands to fetch the API key and supports a subset of the advanced tools.

## Features & Tools

| Feature / Tool | iap/ | manual/ | local/ |
| :--- | :---: | :---: | :---: |
| **IAP JWT Context** | ✅ | ✅ | ✅ |
| **API Key Security** | ❌ | ✅ | ✅ |
| **Auto Key Fetching** | ❌ | ✅ (ADC) | ✅ (gcloud) |
| **`iap_system_info`** | ✅ | ❌ | ❌ |
| **`sysutils_manual_rust`** | ❌ | ✅ | ❌ |
| **`local_system_info`** | ❌ | ❌ | ✅ |
| **`disk_usage`** | ❌ | ✅ | ✅ |
| **`list_processes`** | ❌ | ✅ | ❌ |

### Tool Descriptions

*   **System Information**: Provides a detailed report of the host system.
    *   **Data Collected**: IAP Context, Request Headers, System (Name, Kernel, OS), CPU (Cores), Memory (RAM, Swap), and Network Interfaces (manual/local only).
*   **Disk Usage**: Reports disk usage for all mounted partitions (manual/local only).
*   **Process List**: Lists the top 20 running processes by memory usage (manual only).

## Logging

*   **Structured JSON**: All variants log in JSON format.
*   **Destination**: `iap/` and `manual/` log to `stdout` (standard for Cloud Run), while `local/` logs to `stderr`.

## Getting Started

### Prerequisites

*   [Rust Toolchain](https://www.rust-lang.org/tools/install) (Edition 2024)
*   [Make](https://www.gnu.org/software/make/)

### Build & Run

Each variant has its own directory and `Makefile`.

#### Manual Version (Recommended for Features)
```bash
cd manual
make build
# Ensure ADC is configured or set MCP_API_KEY env var
make run
```

### CLI Usage (Direct Reports)

You can run tools directly from the command line:

```bash
cargo run -- info       # System Info
cargo run -- disk       # Disk Usage (manual/local)
cargo run -- processes  # Process List (manual)
```

## Development

The root directory contains a `Makefile` that can be used to clean all sub-projects. Individual development tasks should be performed within the subdirectories.

*   **Format Code**: `make fmt`
*   **Lint Code**: `make clippy`
*   **Run Tests**: `make test`

## Deployment

`iap/` and `manual/` are containerized and ready for deployment to Google Cloud Run via Google Cloud Build.

```bash
cd iap # or cd manual
make deploy
```

