# Makefile for the Go MCP project

# Variables
BINARY_NAME := stdio-go
GO := go

.PHONY: all build run clean test fmt lint help info disk

# The default target
all: build

# Build the project
build:
	@$(GO) build -o $(BINARY_NAME) main.go

release:
	@$(GO) build -o $(BINARY_NAME) main.go

# Run the MCP server (Stdio)
run: build
	@echo "Running the MCP Stdio server..."
	@./$(BINARY_NAME)

# Get system info directly
info: build
	@./$(BINARY_NAME) info

# Get disk usage directly
disk: build
	@./$(BINARY_NAME) disk

# Clean the project
clean:
	@echo "Cleaning the project..."
	@rm -f $(BINARY_NAME)
	@$(GO) clean

# Run tests
test:
	@echo "Running tests..."
	@$(GO) test ./...

# Format the code
fmt:
	@$(GO) fmt ./...

# Lint the code
lint:
	@if command -v golangci-lint > /dev/null; then \
		golangci-lint run; \
	else \
		echo "golangci-lint not found, skipping linting."; \
	fi

help:
	@echo "Makefile for the Go MCP project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all          (default) same as 'build'"
	@echo "    build        Build the Go binary"
	@echo "    run          Run the MCP server (Stdio)"
	@echo "    info         Display system information report directly"
	@echo "    disk         Display disk usage report directly"
	@echo "    clean        Clean the project"
	@echo "    test         Run tests"
	@echo "    fmt          Format code"
	@echo "    lint         Lint code"
