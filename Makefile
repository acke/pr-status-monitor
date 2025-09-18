.PHONY: build test clean lint run help

# Default target
all: build

# Build the project
build:
	cargo build

# Build release version
release:
	cargo build --release

# Run tests
test:
	cargo test

# Run linter
lint:
	cargo clippy -- -D warnings

# Clean build artifacts
clean:
	cargo clean

# Run the CLI with help
run:
	cargo run -- --help

# Show help
help:
	@echo "Available targets:"
	@echo "  build    - Build the project (debug)"
	@echo "  release  - Build the project (release)"
	@echo "  test     - Run tests"
	@echo "  lint     - Run clippy linter"
	@echo "  clean    - Clean build artifacts"
	@echo "  run      - Run CLI with help"
	@echo "  help     - Show this help"
