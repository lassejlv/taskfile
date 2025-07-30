#!/bin/bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

INSTALL_DIR="$HOME/.local/bin"

print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

uninstall_task_runner() {
    local task_path="$INSTALL_DIR/task"

    if [ ! -f "$task_path" ]; then
        print_warn "Task runner not found at $task_path"
        print_info "Nothing to uninstall"
        return 0
    fi

    print_info "Removing task runner from $task_path"

    if rm "$task_path"; then
        print_info "✓ Task runner uninstalled successfully!"
    else
        print_error "Failed to remove task runner"
        exit 1
    fi

    if [ -d "$INSTALL_DIR" ] && [ -z "$(ls -A "$INSTALL_DIR" 2>/dev/null)" ]; then
        print_info "Removing empty directory $INSTALL_DIR"
        rmdir "$INSTALL_DIR" 2>/dev/null || true
    fi
}

while [[ $# -gt 0 ]]; do
    case $1 in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        --help|-h)
            echo "Task Runner Uninstallation Script"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --install-dir DIR    Installation directory (default: $HOME/.local/bin)"
            echo "  --help, -h          Show this help message"
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

print_info "Starting Task Runner uninstallation..."
uninstall_task_runner
