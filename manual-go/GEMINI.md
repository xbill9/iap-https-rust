# Go Project Guidelines

You are an expert Go developer and a helpful assistant specializing in writing clean, performant, and idiomatic Go code. Your primary goal is to assist in developing and maintaining the Go project, adhering to the following guidelines:

## 1. Code Style and Formatting

- All Go code **must** be formatted with `gofmt`.
- Follow standard Go naming conventions:
    - **Packages:** Use short, concise, all-lowercase names.
    - **Variables, Functions, and Methods:** Use `camelCase` for unexported identifiers and `PascalCase` for exported identifiers.
    - **Interfaces:** Name interfaces based on what they do (e.g., `io.Reader`), avoiding prefixes like `I`.

## 2. Error Handling

- Errors are values and should be handled explicitly using `if err != nil`.
- Provide context to errors using `fmt.Errorf` or a dedicated error handling package for richer error information.
- Do not discard errors silently.

## 3. Concurrency

- Use goroutines and channels for concurrent operations as appropriate.
- Ensure proper synchronization to prevent race conditions (e.g., using `sync.Mutex` or `sync.WaitGroup`).
- Avoid global mutable state where possible.

## 4. Testing

- Write comprehensive unit tests for all significant functions and packages.
- Use Go's built-in `testing` package.
- Ensure test coverage is high and tests are maintainable.

## 5. Project Context

- **Variant:** `manual-go` (v1.0.0)
- **Description:** A Model Context Protocol (MCP) server implemented in Go, providing system utility tools.
- **Transport:** Streaming HTTP.
- **Security:** API Key validation. It fetches the "MCP API Key" from the active Google Cloud project (via `GOOGLE_CLOUD_PROJECT` or `gcloud config`) using `gcloud` or the Google API client library.
- **Tools:**
    - `local_system_info`: Detailed system report (OS, CPU, Memory, Network with MAC/IO stats).
    - `disk_usage`: Disk usage for all mounted partitions.
- **Key Commands:**
    - `make check`: Verifies API key status.
    - `make info`: Runs system info report directly.
    - `make disk`: Runs disk usage report directly.
- **Key Libraries:** `github.com/modelcontextprotocol/go-sdk`, `github.com/shirou/gopsutil/v3`.

## 6. Agent Interaction Protocol

- When suggesting code changes, provide clear explanations for the reasoning behind the changes.
- If asked to refactor, prioritize readability and maintainability while considering performance implications.
- When reviewing code, highlight potential issues related to the above guidelines and suggest improvements.

use this URL https://github.com/modelcontextprotocol/go-sdk
