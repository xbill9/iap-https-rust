# Makefile for the Python MCP project

# Variables
PROJECT_ID := $(shell gcloud config get-value project)
SERVICE_NAME := httpkey-python
PYTHON := python3
PIP := pip3

.PHONY: all install run clean test fmt lint help info disk

# The default target
all: install

# Install dependencies
install:
	@$(PIP) install .

# Run the MCP server (HTTP SSE)
run:
	@echo "Running the MCP HTTP (SSE) server..."
	@MCP_TRANSPORT=sse PORT=8080 MCP_API_KEY=$(KEY) $(PYTHON) main.py

# Run the MCP server (HTTP SSE)
release:
	@echo "Running the MCP HTTP (SSE) server..."
	@MCP_TRANSPORT=sse PORT=8080 $(PYTHON) main.py

# Get system info directly
info:
	@MCP_API_KEY=$(KEY) $(PYTHON) main.py info

# Get disk usage directly
disk:
	@$(PYTHON) main.py disk

# Clean the project
clean:
	@echo "Cleaning the project..."
	@rm -rf __pycache__
	@rm -rf *.egg-info
	@rm -rf build
	@rm -rf dist

# Run tests (placeholder)
test:
	@echo "Running tests..."
	@$(PYTHON) -m unittest discover

# Format the code (using ruff if available)
fmt:
	@$(PIP) install ruff
	@ruff format .

# Lint the code
lint:
	@$(PIP) install ruff
	@ruff check .

# Deploy to Google Cloud Run via Cloud Build
deploy:
	@echo "Deploying to Google Cloud Run..."
	@gcloud builds submit --config cloudbuild.yaml .

help:
	@echo "Makefile for the Python MCP project"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    all          (default) same as 'install'"
	@echo "    install      Install dependencies"
	@echo "    run          Run the MCP HTTP (SSE) server (requires KEY=<your_api_key>)"
	@echo "    info         Display system information report directly (requires KEY=<your_api_key>)"
	@echo "    disk         Display disk usage report directly"
	@echo "    clean        Clean the project"
	@echo "    test         Run tests"
	@echo "    fmt          Format code"
	@echo "    lint         Lint code"
