# manual-go MCP Server

A Go-based Model Context Protocol (MCP) server providing system utilities and disk usage information via Streaming HTTP transport. This server is designed for local development and direct integration with MCP clients.

## Features

- **High-Performance**: Written in Go using modern features (1.26+).
- **Streaming HTTP Transport**: Standard MCP communication over HTTP (supporting SSE).
- **Direct CLI Access**: Run reports directly from the terminal without starting the server.

### Available Tools

- **`local_system_info`**: Provides a comprehensive system report including:
    - OS and Hostname.
    - CPU core count.
    - Memory usage (Total/Used for both Physical and Swap).
    - Network interface statistics (RX/TX bytes and MAC addresses).
- **`disk_usage`**: Provides detailed information for all mounted partitions:
    - Mount point and file system type.
    - Used vs. Total space (in MB).
    - Usage percentage.

## Installation

Ensure you have Go 1.26+ installed.

```bash
make build
```

## Usage

### 1. Running as an MCP Server (Streaming HTTP)

By default, the server starts with Streaming HTTP transport on port 8080.

```bash
make run
```

### 2. Direct CLI Commands

You can execute reports directly for quick inspection:

```bash
# Check system info (performs API Key verification against Cloud)
make info KEY=your_api_key

# Check disk usage (key required)
make disk KEY=your_api_key

# Check API key status directly
make check KEY=your_api_key
```

## Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | Port for the HTTP server | `8080` |
| `MCP_API_KEY` | Manual override for the expected API Key | - |
| `GOOGLE_CLOUD_PROJECT` | Google Cloud Project ID for key fetching | Active `gcloud` project |

## Development

The project includes a comprehensive `Makefile`:

- `make fmt`: Format code using `go fmt`.
- `make lint`: Run `golangci-lint` (if installed).
- `make test`: Execute unit tests.
- `make clean`: Remove the compiled binary.

## Architecture

- **`main.go`**: Contains the MCP server implementation, tool logic, and security middleware.
- **`go-sdk`**: Utilizes the official `github.com/modelcontextprotocol/go-sdk` for standard-compliant MCP communication.
- **`gopsutil`**: Used for cross-platform system and disk metrics.
