# Makefile for WASM Test Project

.PHONY: build copy serve clean all stop

# Default target
all: build copy serve

# Build the WASM module
build:
	@echo "Building WASM module..."
	cargo build --target wasm32-unknown-unknown

# Copy the WASM file to current directory
copy:
	@echo "Copying WASM file..."
	cp ../../target/wasm32-unknown-unknown/debug/test_wasm.wasm .

# Start HTTP server
serve:
	@echo "Starting HTTP server on http://localhost:8000"
	@echo "Press Ctrl+C to stop the server"
	python3 -m http.server 8000

# Stop any running HTTP servers
stop:
	@echo "Stopping HTTP servers..."
	@pkill -f "python3 -m http.server" || true

# Clean build artifacts
clean:
	@echo "Cleaning..."
	cargo clean
	rm -f test_wasm.wasm

# Build, copy, stop any existing server, and serve
dev: stop build copy serve

# Just serve (useful if already built)
serve-only: stop serve

# Help
help:
	@echo "Available targets:"
	@echo "  all       - Build, copy, and serve (default)"
	@echo "  build     - Build the WASM module"
	@echo "  copy      - Copy WASM file to current directory"
	@echo "  serve     - Start HTTP server"
	@echo "  serve-only- Stop existing server and start new one"
	@echo "  dev       - Stop server, build, copy, and serve"
	@echo "  stop      - Stop HTTP server"
	@echo "  clean     - Clean build artifacts"
	@echo "  help      - Show this help"
