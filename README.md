# iap-https-rust (v0.2.0)

A multi-language [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) server project that provides system utility tools. This repository features multiple variants implemented in **Rust** and **Python**, supporting both **Streaming HTTP (SSE)** and **Stdio** transports. It is optimized for both local development and deployment on Google Cloud Run with Identity-Aware Proxy (IAP) or API Key security.

## Project Structure

This repository is organized into several variants to suit different deployment and development needs:

### Rust Variants (`rmcp` based)
1.  **`iap/`**: Standard version for Google Cloud Run with IAP. Decodes `x-goog-iap-jwt-assertion`.
2.  **`manual/`**: Enhanced version for Cloud Run with API key check (ADC-fetched) and IAP.
3.  **`local/`**: Tailored for local development over HTTP with `gcloud` API key fetching.
4.  **`stdio/`**: Lightweight Stdio-based version for local use without extra security layers.
5.  **`stdiokey/`**: Stdio-based version with API key validation (fetched via `gcloud` or ADC).

### Python Variants (`FastMCP` based)
6.  **`local-python/`**: Python implementation of the local HTTP (SSE) variant with API key security.
7.  **`manual-python/`**: Python implementation optimized for Cloud Run/Manual use (SSE) with API key security.
8.  **`stdiokey-python/`**: Python implementation using Stdio transport with API key validation.

## Features & Comparison

| Variant | Language | Transport | Security | Key Fetching |
| :--- | :--- | :--- | :--- | :--- |
| **`iap`** | Rust | HTTP | IAP | N/A |
| **`manual`** | Rust | HTTP | IAP + API Key | ADC |
| **`local`** | Rust | HTTP | API Key | gcloud |
| **`stdio`** | Rust | Stdio | None | N/A |
| **`stdiokey`** | Rust | Stdio | API Key | gcloud / ADC |
| **`local-python`** | Python | HTTP (SSE) | API Key | gcloud / ADC |
| **`manual-python`** | Python | HTTP (SSE) | API Key | gcloud / ADC |
| **`stdiokey-python`**| Python | Stdio | API Key | gcloud / ADC |

## Tools Provided

*   **System Information**: Detailed host report (CPU, Memory, OS, Network).
*   **Disk Usage**: Reports usage for all mounted partitions.
*   **Process List**: Lists top 20 processes by memory (available in `manual` variants).

## Getting Started

### Prerequisites
*   **Rust**: Toolchain (Edition 2024)
*   **Python**: Version 3.11+
*   **Make**: For automated tasks

### Quick Start (Manual Rust)
```bash
cd manual
make build
# Ensure ADC is configured or set MCP_API_KEY env var
make run
```

### Quick Start (Python)
```bash
cd local-python
make install
make run KEY=<YOUR_API_KEY>
```

## CLI Usage (Direct Reports)

Most variants support direct CLI execution for quick reports:
*   **Rust**: `cargo run -- info` or `cargo run -- disk`
*   **Python**: `python3 main.py info` or `python3 main.py disk`

## Development & Deployment

Each subdirectory contains its own `Makefile` for formatting (`make fmt`), linting (`make clippy` / `make lint`), and testing (`make test`). Deployment to Cloud Run is supported for `iap/` and `manual/` variants via `make deploy`.

