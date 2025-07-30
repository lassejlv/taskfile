# Rust Task Runner

A simple, fast task runner that executes commands defined in TOML files with environment variable support and Node.js/npm script integration.

## Installation

### Option 1: Install Script (Recommended)

```bash
curl -sSL https://raw.githubusercontent.com/lassejlv/taskfile/main/install.sh | bash

# to uninstall use
curl -sSL https://raw.githubusercontent.com/lassejlv/taskfile/main/uninstall.sh | bash
```

### Option 2: Download Binary

Download the latest release for your platform from [GitHub Releases](https://github.com/lassejlv/taskfile/releases).

### Option 3: Build from Source

```bash
git clone https://github.com/lassejlv/taskfile.git
cd taskfile
cargo build --release
cp target/release/task ~/.local/bin/
```

## Quick Start

1. Initialize a new Taskfile or create a `Taskfile.toml`:

```bash
task init  # Creates a default Taskfile.toml
```

Or manually create a `Taskfile.toml`:

```toml
[env]
files = [".env", ".env.local"]

[tasks.hello]
cmd = "echo 'Hello $USER_NAME!'"
desc = "Greet user"

[tasks.build]
cmd = "cargo build"
desc = "Build project"

[tasks.format]
cmd = "prettier . --write"
desc = "Format code using prettier from node_modules"

[tasks.test]
cmd = "jest"
desc = "Run tests using npm script"
```

2. Create `.env` file:

```bash
USER_NAME=Developer
```

3. Run tasks:

```bash
task init        # Initialize new Taskfile.toml
task list        # List all tasks
task hello       # Run hello task
task build       # Run build task
```

## Features

- ✅ Environment variable substitution (`$VAR_NAME`)
- 🔗 Task dependencies with `depends_on`
- 📋 Clean table output for task listing
- 🎨 Colored success/error indicators with real-time spinner
- ⏱️ Task execution timing
- 📁 Multi-file env support with precedence
- 🟢 Node.js/npm script integration
- 📦 Automatic `node_modules/.bin` PATH enhancement
- 🏗️ Modular crate architecture
- 🚫 Circular dependency detection
- 🚀 Auto-initialization of Taskfile.toml

## Node.js Integration

When a `package.json` file is detected, the task runner automatically:

- Detects npm scripts and runs them with the appropriate package manager (npm/yarn/pnpm)
- Adds `node_modules/.bin` to PATH for direct access to installed tools
- Supports commands like `prettier`, `eslint`, `jest` without full paths

Example:
```toml
[tasks.lint]
cmd = "eslint ."
desc = "Lint code (will use node_modules/.bin/eslint)"

[tasks.format]
cmd = "prettier . --write"
desc = "Format code (will use node_modules/.bin/prettier)"

[tasks.test]
cmd = "test"
desc = "Run npm test script"
```

## Supported Platforms

- Linux (x86_64, ARM64)
- macOS (Intel, Apple Silicon)

## Project Structure

- `crates/env-parser` - Environment variable parsing and substitution
- `crates/runner` - Task execution engine with Node.js integration
- `crates/cli` - Command-line interface
