# stdio-go MCP Server

A Go-based Model Context Protocol (MCP) server providing system utilities and disk usage information via Stdio transport. This server is designed for local development and direct integration with MCP clients.

## Features

- **High-Performance**: Written in Go using modern features (1.26+).
- **Stdio Transport**: Standard MCP communication over stdin/stdout.
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

### 1. Running as an MCP Server (Stdio)

By default, the server starts with Stdio transport.

```bash
make run
```

### 2. Direct CLI Commands

You can execute reports directly for quick inspection:

```bash
# Check system info
make info

# Check disk usage
make disk
```

## Development

The project includes a comprehensive `Makefile`:

- `make fmt`: Format code using `go fmt`.
- `make lint`: Run `golangci-lint` (if installed).
- `make test`: Execute unit tests.
- `make clean`: Remove the compiled binary.

## Architecture

- **`main.go`**: Contains the MCP server implementation, tool logic, and security middleware.
- **`mcp-go` SDK**: Utilizes `mark3labs/mcp-go` for standard-compliant MCP communication.
- **`gopsutil`**: Used for cross-platform system and disk metrics.
