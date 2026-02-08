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
	@echo "Building local variant..."
	@$(MAKE) -C local build

# Clean both variants
clean:
	@echo "Cleaning the projects..."
	@$(MAKE) -C iap clean
	@$(MAKE) -C manual clean
	@$(MAKE) -C local clean
	@$(MAKE) -C stdio clean
	@$(MAKE) -C stdiokey clean

# Run tests for both
test:
	@echo "Testing iap variant..."
	@$(MAKE) -C iap test
	@echo "Testing manual variant..."
	@$(MAKE) -C manual test
	@echo "Testing local variant..."
	@$(MAKE) -C local test

# Format the code
fmt:
	@echo "Formatting code..."
	@$(MAKE) -C iap fmt
	@$(MAKE) -C manual fmt
	@$(MAKE) -C local fmt

# Lint the code
clippy:
	@echo "Linting code..."
	@$(MAKE) -C iap clippy
	@$(MAKE) -C manual clippy
	@$(MAKE) -C local clippy

# Check the code
check:
	@echo "Checking the code..."
	@$(MAKE) -C iap check
	@$(MAKE) -C manual check
	@$(MAKE) -C local check

help:
	@echo "Root Makefile for iap-https-rust"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    build        Build all variants (iap, manual, local)"
	@echo "    clean        Clean all variants"
	@echo "    test         Run tests for all variants"
	@echo "    fmt          Check formatting for all"
	@echo "    clippy       Lint all variants"
	@echo "    check        Check all variants"
	@echo ""
	@echo "Note: To run or deploy, navigate to the specific directory (iap/ or manual/)."
