# Gemini Workspace for `iap-https-rust` (v0.2.0)

You are a developer working on a multi-language MCP server project. This workspace encompasses both **Rust** (Edition 2024) and **Python** (3.11+) implementations.

## Project Overview

`iap-https-rust` (and its Python counterparts) is a suite of Model Context Protocol (MCP) servers providing system utility tools. It supports multiple transports (Streaming HTTP/SSE and Stdio) and various security models (IAP, API Key).

### Project Structure

This repository is divided into several specialized variants:

#### Rust Variants (`rmcp` v0.14+)
*   **`iap/`**: Cloud Run + IAP. HTTP transport.
*   **`manual/`**: Cloud Run + IAP + API Key (ADC). HTTP transport.
*   **`local/`**: Local dev + API Key (gcloud). HTTP transport.
*   **`stdio/`**: Basic Stdio transport. No security.
*   **`stdiokey/`**: Stdio transport + API Key (gcloud/ADC).

#### Python Variants (`FastMCP` based)
*   **`local-python/`**: Local/Cloud SSE + API Key (gcloud/ADC).
*   **`manual-python/`**: Cloud-ready SSE + API Key (gcloud/ADC).
*   **`stdiokey-python/`**: Stdio transport + API Key (gcloud/ADC).

### Key Technologies

*   **Rust Stack**: `rmcp`, `tokio`, `axum`, `sysinfo`, `serde`, `tracing`.
*   **Python Stack**: `mcp`, `starlette` (SSE), `uvicorn`, `psutil`, `pydantic`.
*   **Security**: Google Cloud IAP (JWT), Google Cloud API Keys (ADC or `gcloud` CLI).

## Architecture & Security

### Security Mechanisms
*   **IAP Middleware**: (Rust variants) Extracts identity from `x-goog-iap-jwt-assertion`.
*   **API Key Validation**: 
    *   **Manual/Cloud**: Uses `google-apikeys2` (Rust) or `google-api-python-client` to fetch the expected "MCP API Key" from project `1056842563084` via ADC.
    *   **Local**: Uses `gcloud services api-keys` CLI to fetch the key for developer convenience.
*   **Task-Local Context**: Rust variants use `tokio::task_local` to store request context and headers.

### Transport Details
*   **Streaming HTTP**: Rust uses `transport-streamable-http-server`.
*   **SSE**: Python uses `mcp.run(transport="sse")`.
*   **Stdio**: Standard MCP JSON-RPC over stdin/stdout.

## Development Workflow

### Shared Commands
Each subdirectory has a `Makefile` with standard targets:
*   **Rust**: `make fmt`, `make clippy`, `make test`, `make build`.
*   **Python**: `make fmt` (ruff), `make lint` (ruff), `make test` (unittest).

### Environment Variables
*   `PORT`: Port for HTTP/SSE servers (default: 8080).
*   `MCP_API_KEY`: Manual override for API key validation.
*   `GOOGLE_CLOUD_PROJECT`: Project ID for API key fetching.

## Interacting with Gemini

*   "Add a disk usage tool to the Stdio Rust variant in `stdiokey/src/main.rs`."
*   "Refactor the Python middleware in `local-python/main.py` to support custom error messages."
*   "Explain the API key fetching logic in the `manual` variant vs the `local` variant."
*   "How do I deploy the Python manual variant to Cloud Run?"