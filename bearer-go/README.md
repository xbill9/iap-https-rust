# bearer-go MCP Server

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

The server exposes:
- `/`: The MCP Streaming HTTP endpoint.
- `/healthz`: A health check endpoint returning `OK`.

### 2. Direct CLI Commands

You can execute reports directly for quick inspection:

```bash
# Check system info
make info

# Check disk usage
make disk

# Check status directly
make check
```

## Security

This variant of the server supports **Bearer Token Authentication**. 

### Configuring Authentication

To enable authentication, set the `MCP_BEARER_TOKEN` environment variable before starting the server:

```bash
export MCP_BEARER_TOKEN="your-secure-token"
make run
```

When enabled, all MCP requests (except for health checks) must include the following HTTP header:

`Authorization: Bearer your-secure-token`

If `MCP_BEARER_TOKEN` is not set, the server operates without authentication (open access).

## Deployment

You can deploy this server to Google Cloud Run using the provided `Makefile` target:

```bash
make deploy
```

For secure deployments, it is recommended to store the `MCP_BEARER_TOKEN` in Secret Manager and reference it in the Cloud Run service configuration.

## Environment Variables

| Variable | Description | Default |
| :--- | :--- | :--- |
| `PORT` | Port for the HTTP server | `8080` |
| `MCP_BEARER_TOKEN` | Optional bearer token for authentication | (None) |

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
