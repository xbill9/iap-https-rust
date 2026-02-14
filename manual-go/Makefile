# Makefile for the Go MCP project

# Variables
PROJECT_ID := $(shell gcloud config get-value project)
BINARY_NAME := manual-go
GO := go

.PHONY: all build run clean test fmt lint help info disk

# The default target
all: build

# Build the project
build:
	@$(GO) build -o $(BINARY_NAME) main.go

# Build the project
release:
	@$(GO) build -o $(BINARY_NAME) main.go
	@./$(BINARY_NAME)

# Run the MCP server (Streaming HTTP)
run: build
	@echo "Running the MCP Streaming HTTP server..."
	@MCP_API_KEY=$(KEY) PORT=$(PORT) ./$(BINARY_NAME)

# Get system info directly
info: build
	@MCP_API_KEY=$(KEY) ./$(BINARY_NAME) info

# Get disk usage directly
disk: build
	@MCP_API_KEY=$(KEY) ./$(BINARY_NAME) disk

# Check API key status directly
check: build
	@MCP_API_KEY=$(KEY) ./$(BINARY_NAME) check

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

# Submit the build to Google Cloud Build
deploy:
	@echo "Submitting build to Google Cloud Build..."
	@make clean
	@gcloud builds submit . --config cloudbuild.yaml


help:
	@echo "Makefile for the Go MCP project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all          (default) same as 'build'"
	@echo "    build        Build the Go binary"
	@echo "    run          Run the MCP Streaming HTTP server (requires KEY=<your_api_key> if not in env)"
	@echo "    info         Display system information report directly (requires KEY=<your_api_key>)"
	@echo "    disk         Display disk usage report directly"
	@echo "    check        Check API key status directly"
	@echo "    clean        Clean the project"
	@echo "    test         Run tests"
	@echo "    fmt          Format code"
	@echo "    lint         Lint code"
