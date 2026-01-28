# Default recipe - shows available commands
default:
    @just --list

# Format code with cargo fmt
fmt:
    cargo fmt

# Check formatting without changes
fmt-check:
    cargo fmt -- --check

# Run clippy lints
lint:
    cargo clippy -- -D clippy::all

# Run all tests
test:
    cargo test

# Check if code compiles (faster than build)
check:
    cargo check

# Build in debug mode
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Install in user mode
install:
    cargo install --path .

# Run all CI checks (format, lint, test)
ci: fmt-check lint test

# Clean build artifacts
clean:
    cargo clean

# Generate and print config schema
schema:
    cargo run -- --print-config-schema

# Generate and print layout config schema
layout-schema:
    cargo run -- --print-layout-config-schema

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and check compilation
watch-check:
    cargo watch -x check
