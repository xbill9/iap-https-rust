# Makefile for iap-https-rust workspace

.PHONY: all build run clean test fmt clippy check help

# The default target
all: build

# Build both variants
build:
	@echo "Building iap variant..."
	@$(MAKE) -C iap build
	@echo "Building manual variant..."
	@$(MAKE) -C manual build

# Clean both variants
clean:
	@echo "Cleaning the projects..."
	@$(MAKE) -C iap clean
	@$(MAKE) -C manual clean

# Run tests for both
test:
	@echo "Testing iap variant..."
	@$(MAKE) -C iap test
	@echo "Testing manual variant..."
	@$(MAKE) -C manual test

# Format the code
fmt:
	@echo "Formatting code..."
	@$(MAKE) -C iap fmt
	@$(MAKE) -C manual fmt

# Lint the code
clippy:
	@echo "Linting code..."
	@$(MAKE) -C iap clippy
	@$(MAKE) -C manual clippy

# Check the code
check:
	@echo "Checking the code..."
	@$(MAKE) -C iap check
	@$(MAKE) -C manual check

help:
	@echo "Root Makefile for iap-https-rust"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    build        Build both iap and manual variants"
	@echo "    clean        Clean both variants"
	@echo "    test         Run tests for both variants"
	@echo "    fmt          Check formatting for both"
	@echo "    clippy       Lint both variants"
	@echo "    check        Check both variants"
	@echo ""
	@echo "Note: To run or deploy, navigate to the specific directory (iap/ or manual/)."
