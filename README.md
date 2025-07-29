# Rust Task Runner

A simple, fast task runner that executes commands defined in TOML files with environment variable support.

## Installation

### Option 1: Install Script (Recommended)
```bash
curl -sSL https://raw.githubusercontent.com/your-username/rust-hello-world/main/install.sh | bash
```

### Option 2: Download Binary
Download the latest release for your platform from [GitHub Releases](https://github.com/your-username/rust-hello-world/releases).

### Option 3: Build from Source
```bash
git clone https://github.com/your-username/rust-hello-world.git
cd rust-hello-world
cargo build --release
cp target/release/task ~/.local/bin/
```

## Quick Start

1. Create a `Taskfile.toml`:
```toml
[env]
files = [".env", ".env.local"]

[tasks.hello]
cmd = "echo 'Hello $USER_NAME!'"
desc = "Greet user"

[tasks.build]
cmd = "cargo build"
desc = "Build project"
```

2. Create `.env` file:
```bash
USER_NAME=Developer
```

3. Run tasks:
```bash
task list        # List all tasks
task hello       # Run hello task
task build       # Run build task
```

## Features

- ✅ Environment variable substitution (`$VAR_NAME`)
- 🔗 Task dependencies with `depends_on`
- 📋 Clean table output for task listing
- 🎨 Colored success/error indicators
- 📁 Multi-file env support with precedence
- 🏗️ Modular crate architecture
- 🚫 Circular dependency detection

## Supported Platforms

- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)

## Project Structure

- `crates/env-parser` - Environment variable parsing and substitution
- `crates/taskfile-runner` - Task execution engine
- `src/main.rs` - Simple CLI interface