#!/bin/bash

# Claude Code hook: Ensure correct Rust version and gh CLI are installed (remote environments only)
# This hook checks the rust-toolchain.toml and installs the required Rust version
# and installs the GitHub CLI (gh) for cloud development
# Only runs when CLAUDE_CODE_REMOTE=true

# NOTE: Using resilient error handling instead of 'set -e' to prevent silent failures
# SessionStart hooks can fail silently - we want to attempt all operations even if one fails

# Only run in Claude Code remote environment
if [[ "$CLAUDE_CODE_REMOTE" != "true" ]]; then
    exit 0
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
TOOLCHAIN_FILE="$PROJECT_ROOT/rust-toolchain.toml"

# Function to install rustup
install_rustup() {
    echo "rustup not found. Installing rustup..." >&2
    if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --no-modify-path; then
        # shellcheck disable=SC1091
        source "$HOME/.cargo/env" 2>/dev/null || true
        echo "rustup installed successfully." >&2
        return 0
    else
        echo "Warning: Failed to install rustup" >&2
        return 1
    fi
}

# Function to install GitHub CLI (gh)
install_gh() {
    echo "gh not found. Installing GitHub CLI..." >&2

    # Try apt-get first (works in most Debian/Ubuntu environments)
    if command -v apt-get &> /dev/null; then
        echo "Installing gh via apt-get..." >&2
        if apt-get update -qq 2>&1 && apt-get install -y gh 2>&1; then
            echo "GitHub CLI installed successfully via apt-get." >&2
            return 0
        else
            echo "Warning: apt-get installation failed, trying fallback method..." >&2
        fi
    fi

    # Fallback: Try downloading from GitHub releases (may fail in restricted environments)
    echo "apt-get not available or failed, trying GitHub releases..." >&2

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
            echo "Warning: Unsupported architecture: $ARCH" >&2
            return 1
            ;;
    esac

    # Create bin directory
    mkdir -p "$HOME/.local/bin" || true

    # Get latest version from GitHub API
    GH_VERSION=$(curl -s https://api.github.com/repos/cli/cli/releases/latest 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"v([^"]+)".*/\1/')

    if [[ -z "$GH_VERSION" ]]; then
        GH_VERSION="2.40.1"  # fallback version
        echo "Warning: Could not fetch latest version, using fallback: ${GH_VERSION}" >&2
    fi

    # Download and install
    GH_URL="https://github.com/cli/cli/releases/download/v${GH_VERSION}/gh_${GH_VERSION}_linux_${ARCH}.tar.gz"

    echo "Downloading gh version ${GH_VERSION} for ${ARCH}..." >&2
    if curl -fsSL "$GH_URL" 2>&1 | tar -xz -C /tmp 2>&1; then
        if cp "/tmp/gh_${GH_VERSION}_linux_${ARCH}/bin/gh" "$HOME/.local/bin/" 2>&1; then
            rm -rf "/tmp/gh_${GH_VERSION}_linux_${ARCH}" 2>/dev/null || true

            # Ensure PATH includes ~/.local/bin
            if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
                export PATH="$HOME/.local/bin:$PATH"
            fi

            echo "GitHub CLI installed successfully from GitHub releases." >&2
            return 0
        fi
    fi

    echo "Warning: Failed to install GitHub CLI" >&2
    echo "Please install gh manually: https://github.com/cli/cli#installation" >&2
    return 1
}

# Main logic
main() {
    local rustup_ok=true
    local gh_ok=true

    # Check if rustup is installed
    if ! command -v rustup &> /dev/null; then
        install_rustup || rustup_ok=false
    fi

    # If rust-toolchain.toml exists, rustup will automatically use it
    # Just ensure the toolchain is installed
    if [[ "$rustup_ok" == "true" ]] && [[ -f "$TOOLCHAIN_FILE" ]]; then
        echo "Found rust-toolchain.toml, ensuring toolchain is installed..." >&2
        if rustup show active-toolchain 2>&1 || rustup install 2>&1; then
            echo "Rust toolchain ready." >&2
        else
            echo "Warning: Failed to install Rust toolchain" >&2
            rustup_ok=false
        fi
    fi

    # Check if gh is installed
    if ! command -v gh &> /dev/null; then
        install_gh || gh_ok=false
    else
        echo "GitHub CLI (gh) is already installed: $(gh --version 2>&1 | head -n1)" >&2
    fi

    # Always exit 0 to allow partial success (non-blocking hook)
    # This prevents SessionStart hook from blocking if one component fails
    if [[ "$rustup_ok" == "true" && "$gh_ok" == "true" ]]; then
        echo "SessionStart hook completed successfully" >&2
    else
        echo "SessionStart hook completed with warnings (some components may not be installed)" >&2
    fi
    exit 0
}

main
