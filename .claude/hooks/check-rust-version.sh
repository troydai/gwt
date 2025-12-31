#!/bin/bash

# Claude Code hook: Ensure correct Rust version is installed (remote environments only)
# This hook checks the rust-toolchain.toml and installs the required Rust version
# Only runs when CLAUDE_CODE_REMOTE=true

set -e

# Only run in Claude Code remote environment
if [[ "$CLAUDE_CODE_REMOTE" != "true" ]]; then
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

# Main logic
main() {
    # Check if rustup is installed
    if ! command -v rustup &> /dev/null; then
        install_rustup
    fi

    # If rust-toolchain.toml exists, rustup will automatically use it
    # Just ensure the toolchain is installed
    if [[ -f "$TOOLCHAIN_FILE" ]]; then
        echo "Found rust-toolchain.toml, ensuring toolchain is installed..."
        rustup show active-toolchain || rustup install
        echo "Rust toolchain ready."
    fi
}

main
