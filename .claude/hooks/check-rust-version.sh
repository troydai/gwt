#!/bin/bash

# Claude Code hook: Ensure correct Rust version and gh CLI are installed (remote environments only)
# This hook checks the rust-toolchain.toml and installs the required Rust version
# and installs the GitHub CLI (gh) for cloud development
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

# Function to install GitHub CLI (gh)
install_gh() {
    echo "gh not found. Installing GitHub CLI..."

    # Detect architecture
    ARCH=$(uname -m)
    case $ARCH in
        x86_64)
            ARCH="amd64"
            ;;
        aarch64)
            ARCH="arm64"
            ;;
        *)
            echo "Unsupported architecture: $ARCH"
            return 1
            ;;
    esac

    # Create bin directory
    mkdir -p "$HOME/.local/bin"

    # Get latest version from GitHub API
    GH_VERSION=$(curl -s https://api.github.com/repos/cli/cli/releases/latest | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [[ -z "$GH_VERSION" ]]; then
        GH_VERSION="2.40.1"  # fallback version
    fi

    # Download and install
    GH_URL="https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_${ARCH}.tar.gz"

    echo "Downloading gh version ${GH_VERSION} for ${ARCH}..."
    curl -fsSL "$GH_URL" | tar -xz -C /tmp
    cp "/tmp/gh_${GH_VERSION}_linux_${ARCH}/bin/gh" "$HOME/.local/bin/"
    rm -rf "/tmp/gh_${GH_VERSION}_linux_${ARCH}"

    # Ensure PATH includes ~/.local/bin
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        export PATH="$HOME/.local/bin:$PATH"
    fi

    echo "GitHub CLI installed successfully."
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

    # Check if gh is installed
    if ! command -v gh &> /dev/null; then
        install_gh
    else
        echo "GitHub CLI (gh) is already installed: $(gh --version | head -n1)"
    fi
}

main
