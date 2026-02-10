# Makefile for iap-https-rust workspace

.PHONY: all build run clean test fmt clippy check help

# The default target
all: build

# Build all variants
build:
	@echo "Building iap variant..."
	@$(MAKE) -C iap build
	@echo "Building manual variant..."
	@$(MAKE) -C manual build
	@echo "Building local variant..."
	@$(MAKE) -C local build
	@echo "Building stdio variant..."
	@$(MAKE) -C stdio build
	@echo "Building stdiokey variant..."
	@$(MAKE) -C stdiokey build

# Clean all variants
clean:
	@echo "Cleaning the projects..."
	@$(MAKE) -C iap clean
	@$(MAKE) -C manual clean
	@$(MAKE) -C local clean
	@$(MAKE) -C stdio clean
	@$(MAKE) -C stdiokey clean
	@$(MAKE) -C stdiokey-python clean

# Run tests for all
test:
	@echo "Testing iap variant..."
	@$(MAKE) -C iap test
	@echo "Testing manual variant..."
	@$(MAKE) -C manual test
	@echo "Testing local variant..."
	@$(MAKE) -C local test
	@echo "Testing stdio variant..."
	@$(MAKE) -C stdio test
	@echo "Testing stdiokey variant..."
	@$(MAKE) -C stdiokey test
	@echo "Testing stdiokey-python variant..."
	@$(MAKE) -C stdiokey-python test

# Format the code
fmt:
	@echo "Formatting code..."
	@$(MAKE) -C iap fmt
	@$(MAKE) -C manual fmt
	@$(MAKE) -C local fmt
	@$(MAKE) -C stdio fmt
	@$(MAKE) -C stdiokey fmt
	@$(MAKE) -C stdiokey-python fmt

# Lint the code
clippy:
	@echo "Linting code..."
	@$(MAKE) -C iap clippy
	@$(MAKE) -C manual clippy
	@$(MAKE) -C local clippy
	@$(MAKE) -C stdio clippy
	@$(MAKE) -C stdiokey clippy
	@echo "Linting stdiokey-python variant..."
	@$(MAKE) -C stdiokey-python lint

# Check the code
check:
	@echo "Checking the code..."
	@$(MAKE) -C iap check
	@$(MAKE) -C manual check
	@$(MAKE) -C local check
	@$(MAKE) -C stdio check
	@$(MAKE) -C stdiokey check

help:
	@echo "Root Makefile for iap-https-rust"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    build        Build all variants (iap, manual, local, stdio, stdiokey)"
	@echo "    clean        Clean all variants (including stdiokey-python)"
	@echo "    test         Run tests for all variants"
	@echo "    fmt          Check formatting for all"
	@echo "    clippy       Lint all variants"
	@echo "    check        Check all variants"
	@echo ""
	@echo "Note: To run or deploy, navigate to the specific directory."
