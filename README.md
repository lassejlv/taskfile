# Rust Task Runner

A simple, fast task runner that executes commands defined in TOML files with environment variable support.

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
cargo run list        # List all tasks
cargo run hello       # Run hello task
cargo run build       # Run build task
```

## Features

- ✅ Environment variable substitution (`$VAR_NAME`)
- 📋 Clean table output for task listing
- 🎨 Colored success/error indicators
- 📁 Multi-file env support with precedence
- 🏗️ Modular crate architecture

## Project Structure

- `env-parser` - Environment variable parsing and substitution
- `taskfile-runner` - Task execution engine
- `src/main.rs` - Simple CLI interface