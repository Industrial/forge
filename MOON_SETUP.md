# Moonrepo (moon) Setup

This repository uses [moonrepo](https://moonrepo.dev/docs) as the build system and monorepo management tool.

## Installation

Moon is installed via nix and available in the devenv shell. It's automatically available when you run `devenv shell`.

**Important**: Moon is configured to use devenv's Rust toolchain (not install its own). The `MOON_TOOLCHAIN_FORCE_GLOBALS=rust` environment variable is set in `.envrc` and `devenv.nix` to prevent moon from trying to install Rust via proto/rustup.

## Configuration

### Workspace Configuration

- **`.moon/workspace.yml`**: Workspace-level configuration
  - Defines project locations (`crates/*`)
  - VCS settings (git, main branch)

- **`.moon/toolchain.yml`**: Toolchain configuration
  - Rust toolchain (uses devenv's Rust, not managed by moon)

### Project Configuration

Each crate can have its own `moon.yml` file for project-specific tasks. Currently, `forge-cli` has a `moon.yml` with example tasks.

## Usage

### Basic Commands

```bash
# Sync workspace (detect projects and update configuration)
devenv shell -- moon sync

# List all projects
devenv shell -- moon project list

# Show project details
devenv shell -- moon project forge-cli

# Run a task for a specific project
devenv shell -- moon run forge-cli:build
devenv shell -- moon run forge-cli:check
devenv shell -- moon run forge-cli:test
devenv shell -- moon run forge-cli:lint

# Run tasks for all projects
devenv shell -- moon run :build

# Show task details
devenv shell -- moon task forge-cli:build

# List all tasks
devenv shell -- moon query tasks

# View task graph (interactive web UI)
devenv shell -- moon task-graph

# View project dependency graph (interactive web UI)
devenv shell -- moon project-graph

# Export task graph as DOT format (for Graphviz)
devenv shell -- moon task-graph --dot > task-graph.dot

# Export project graph as JSON
devenv shell -- moon project-graph --json > project-graph.json
```

### Available Tasks (forge-cli example)

- `build`: Build the crate (`cargo build -p <crate>`)
- `check`: Type check (`cargo check -p <crate>`)
- `test`: Run tests (`cargo nextest run -p <crate>`)
- `lint`: Run clippy (`cargo clippy -p <crate>`)

## Adding Tasks to Other Crates

To add moon tasks to other crates, create a `moon.yml` file in the crate directory:

```yaml
# crates/your-crate/moon.yml
language: 'rust'
type: 'library'  # or 'application'

tasks:
  build:
    command: 'cargo'
    args: ['build', '-p', 'your-crate']
    options:
      runFromWorkspaceRoot: true
    inputs:
      - 'Cargo.toml'
      - 'Cargo.lock'
      - '**/*.rs'
    outputs:
      - 'target/**/*'
    local: true
```

## Moon Tasks

### Workspace-Level Tasks

Run these from the repository root:

```bash
# Format entire workspace
moon run :format

# Type check all crates
moon run :check

# Lint all crates
moon run :lint

# Build all crates
moon run :build

# Test all crates
moon run :test

# Audit dependencies
moon run :audit

# Check documentation
moon run :check-docs

# Integration tests
moon run :test-integration

# E2E tests
moon run :test-e2e
```

### Per-Project Tasks

Run tasks for specific crates:

```bash
# Build specific crate
moon run forge-cli:build

# Check specific crate
moon run forge-core:check

# Test specific crate
moon run forge-auth:test

# Lint specific crate
moon run forge-db:lint
```

### Git Hooks

The `bin/pre-push` script uses moon to run pre-push validation:

```bash
bin/pre-push  # Runs: format, check, lint, build, test, audit, check-docs
```

## Moon Benefits

Moon provides:
- **Incremental builds**: Only rebuilds changed crates
- **Dependency graph**: Understands crate dependencies
- **Caching**: Smart caching of build outputs
- **Parallel execution**: Runs independent tasks in parallel
- **Consistent interface**: Same commands for all crates

## Gitignore

Moon cache directories are already in `.gitignore`:
- `.moon/cache`
- `.moon/docker`

## Documentation

- [Moon Documentation](https://moonrepo.dev/docs)
- [Moon Rust Support](https://moonrepo.dev/docs/guides/rust)
