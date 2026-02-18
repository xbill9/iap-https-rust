# Makefile for the Go MCP project

# Variables
BINARY_NAME := bearer-go
GO := go

.PHONY: all build run clean test fmt lint help info disk

# The default target
all: build

# Build the project
build:
	@$(GO) build -o $(BINARY_NAME) main.go

# Build the project and run immediately
release:
	@$(GO) build -o $(BINARY_NAME) main.go
	@./$(BINARY_NAME)

lint:
	go vet

# Run the MCP server (Streaming HTTP)
run: build
	@echo "Running the MCP Streaming HTTP server..."
	@PORT=$(PORT) ./$(BINARY_NAME)

# Get system info directly
info: build
	@./$(BINARY_NAME) info

# Get disk usage directly
disk: build
	@./$(BINARY_NAME) disk

# Check status directly
check: build
	@./$(BINARY_NAME) check

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
	@echo "    run          Run the MCP Streaming HTTP server"
	@echo "    info         Display system information report directly"
	@echo "    disk         Display disk usage report directly"
	@echo "    check        Check status directly"
	@echo "    clean        Clean the project"
	@echo "    test         Run tests"
	@echo "    fmt          Format code"
	@echo "    lint         Lint code"
