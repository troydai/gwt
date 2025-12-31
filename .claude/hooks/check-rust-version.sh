#!/bin/bash

# Claude Code hook: Ensure correct Rust version is installed (cloud environments only)
# This hook checks the rust-toolchain.toml and installs the required Rust version
# Skips execution on macOS (local dev environment)

set -e

# Skip on macOS - this hook is only for cloud environments
if [[ "$(uname -s)" == "Darwin" ]]; then
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TOOLCHAIN_FILE="$PROJECT_ROOT/rust-toolchain.toml"

# Function to install rustup
install_rustup() {
    echo "rustup not found. Installing rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path
    source "$HOME/.cargo/env"
    echo "rustup installed successfully."
}

# Function to get required Rust version from rust-toolchain.toml
get_required_version() {
    if [[ -f "$TOOLCHAIN_FILE" ]]; then
        # Parse the channel from rust-toolchain.toml
        grep -E '^channel\s*=' "$TOOLCHAIN_FILE" | sed 's/^channel[[:space:]]*=[[:space:]]*"\{0,1\}\([^"]*\)"\{0,1\}/\1/' | tr -d ' '
    else
        echo ""
    fi
}

# Function to get current Rust version
get_current_version() {
    if command -v rustc &> /dev/null; then
        rustc --version | awk '{print $2}'
    else
        echo ""
    fi
}

# Main logic
main() {
    # Check if rustup is installed
    if ! command -v rustup &> /dev/null; then
        install_rustup
    fi

    # Get required version from rust-toolchain.toml
    REQUIRED_VERSION=$(get_required_version)

    if [[ -z "$REQUIRED_VERSION" ]]; then
        echo "No rust-toolchain.toml found or no channel specified. Skipping version check."
        exit 0
    fi

    # Get current version
    CURRENT_VERSION=$(get_current_version)

    # Compare versions (strip any leading 'v' if present)
    REQUIRED_CLEAN=$(echo "$REQUIRED_VERSION" | sed 's/^v//')
    CURRENT_CLEAN=$(echo "$CURRENT_VERSION" | sed 's/^v//')

    if [[ "$CURRENT_CLEAN" != "$REQUIRED_CLEAN"* ]]; then
        echo "Rust version mismatch: current=$CURRENT_VERSION, required=$REQUIRED_VERSION"
        echo "Installing Rust $REQUIRED_VERSION..."
        rustup install "$REQUIRED_VERSION"
        rustup default "$REQUIRED_VERSION"
        echo "Rust $REQUIRED_VERSION installed and set as default."
    else
        echo "Rust version OK: $CURRENT_VERSION (required: $REQUIRED_VERSION)"
    fi
}

main
