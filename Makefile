# Makefile for iap-https-rust workspace

RUST_VARIANTS := iap manual local stdio stdiokey
PYTHON_VARIANTS := local-python manual-python stdiokey-python
ALL_VARIANTS := $(RUST_VARIANTS) $(PYTHON_VARIANTS)

.PHONY: all build clean test fmt clippy check help $(ALL_VARIANTS)

# The default target
all: build

# Build all variants
build:
	@for dir in $(RUST_VARIANTS); do \
		echo "Building $$dir variant..."; \
		$(MAKE) -C $$dir build; \
	done
	@for dir in $(PYTHON_VARIANTS); do \
		echo "Installing dependencies for $$dir variant..."; \
		$(MAKE) -C $$dir install; \
	done

# Clean all variants
clean:
	@echo "Cleaning the projects..."
	@for dir in $(ALL_VARIANTS); do \
		$(MAKE) -C $$dir clean; \
	done

# Run tests for all
test:
	@for dir in $(ALL_VARIANTS); do \
		echo "Testing $$dir variant..."; \
		$(MAKE) -C $$dir test; \
	done

# Format the code
fmt:
	@echo "Formatting code..."
	@for dir in $(ALL_VARIANTS); do \
		$(MAKE) -C $$dir fmt; \
	done

# Lint the code
clippy:
	@echo "Linting code..."
	@for dir in $(RUST_VARIANTS); do \
		$(MAKE) -C $$dir clippy; \
	done
	@for dir in $(PYTHON_VARIANTS); do \
		echo "Linting $$dir variant..."; \
		$(MAKE) -C $$dir lint; \
	done

# Check the code
check:
	@echo "Checking the code..."
	@for dir in $(RUST_VARIANTS); do \
		$(MAKE) -C $$dir check; \
	done
	@for dir in $(PYTHON_VARIANTS); do \
		echo "Linting (checking) $$dir variant..."; \
		$(MAKE) -C $$dir lint; \
	done

help:
	@echo "Root Makefile for iap-https-rust"
	@echo ""
	@echo "Usage:"
	@echo "    make <target>"
	@echo ""
	@echo "Targets:"
	@echo "    build        Build Rust variants and install Python dependencies"
	@echo "    clean        Clean all variants"
	@echo "    test         Run tests for all variants"
	@echo "    fmt          Format code for all variants"
	@echo "    clippy       Lint all variants (clippy for Rust, lint for Python)"
	@echo "    check        Check all variants"
	@echo ""
	@echo "Note: To run or deploy, navigate to the specific directory."