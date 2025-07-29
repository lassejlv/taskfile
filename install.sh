#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

INSTALL_DIR="$HOME/.local/bin"

REPO="rust-hello-world"
OWNER="your-username"

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}


detect_platform() {
    local os arch

    case "$(uname -s)" in
        Linux*)
            os="linux"
            ;;
        Darwin*)
            os="macos"
            ;;
        *)
            print_error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac

    case "$(uname -m)" in
        x86_64)
            arch="x86_64"
            ;;
        arm64|aarch64)
            arch="aarch64"
            ;;
        *)
            print_error "Unsupported architecture: $(uname -m)"
            exit 1
            ;;
    esac

    echo "${os}-${arch}"
}


get_latest_version() {
    curl -s "https://api.github.com/repos/${OWNER}/${REPO}/releases/latest" | \
        grep '"tag_name":' | \
        sed -E 's/.*"([^"]+)".*/\1/'
}


install_task_runner() {
    local platform version download_url filename

    platform=$(detect_platform)
    print_info "Detected platform: $platform"


    print_info "Fetching latest version..."
    version=$(get_latest_version)

    if [ -z "$version" ]; then
        print_error "Failed to get latest version"
        exit 1
    fi

    print_info "Latest version: $version"


    filename="task-${platform}.tar.gz"
    download_url="https://github.com/${OWNER}/${REPO}/releases/download/${version}/${filename}"

    print_info "Downloading from: $download_url"


    tmp_dir=$(mktemp -d)
    cd "$tmp_dir"


    if ! curl -L -o "$filename" "$download_url"; then
        print_error "Failed to download $filename"
        exit 1
    fi


    print_info "Extracting archive..."
    tar -xzf "$filename"


    mkdir -p "$INSTALL_DIR"


    print_info "Installing to $INSTALL_DIR/task"
    cp "task-${platform}" "$INSTALL_DIR/task"
    chmod +x "$INSTALL_DIR/task"


    cd /
    rm -rf "$tmp_dir"

    print_info "✓ Task runner installed successfully!"


    if [[ ":$PATH:" != *":$INSTALL_DIR:"* ]]; then
        print_warn "Warning: $INSTALL_DIR is not in your PATH"
        print_warn "Add the following line to your shell profile (~/.bashrc, ~/.zshrc, etc.):"
        echo "export PATH=\"$INSTALL_DIR:\$PATH\""
    else
        print_info "You can now run 'task --help' to get started!"
    fi
}


while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help|-h)
            echo "Task Runner Installation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir DIR    Install directory (default: $HOME/.local/bin)"
            echo "  --help, -h          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done


if ! command -v curl >/dev/null 2>&1; then
    print_error "curl is required but not installed"
    exit 1
fi

if ! command -v tar >/dev/null 2>&1; then
    print_error "tar is required but not installed"
    exit 1
fi


print_info "Starting Task Runner installation..."
install_task_runner
