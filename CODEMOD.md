# Codemod: Replace `log::` with `tracing::`

This codemod uses [ast-grep](https://github.com/ast-grep/ast-grep) to replace all occurrences of `log::` with `tracing::` in the Rust codebase.

## Installation

First, install ast-grep if you haven't already:

```bash
# Using cargo (recommended for Rust projects)
cargo install ast-grep --locked

# Or using homebrew (macOS)
brew install ast-grep

# Or using npm
npm install --global @ast-grep/cli
```

## Usage

### Option 1: Using the justfile command

```bash
just codemod-log-to-tracing
```

### Option 2: Using ast-grep directly

#### Preview changes (dry-run):
```bash
ast-grep scan --rule log-to-tracing.yml src/
```

#### Apply changes:
```bash
ast-grep scan --rule log-to-tracing.yml --rewrite src/
```

#### Interactive mode (review each change):
```bash
ast-grep scan --rule log-to-tracing.yml --rewrite src/ --interactive
```

### Option 3: Command-line pattern (one-off)

If you prefer a quick command-line approach:

```bash
ast-grep --pattern 'log::$M' --rewrite 'tracing::$M' --lang rust src/
```

## What it does

The codemod replaces:
- `log::debug!(...)` → `tracing::debug!(...)`
- `log::info!(...)` → `tracing::info!(...)`
- `log::warn!(...)` → `tracing::warn!(...)`
- `log::error!(...)` → `tracing::error!(...)`
- Any other `log::` path → `tracing::` path

## Configuration

The codemod configuration is in `log-to-tracing.yml`:

```yaml
id: log-to-tracing
language: rust
rule:
  pattern: log::$M
fix: tracing::$M
```

This pattern matches any path starting with `log::` and captures the rest (`$M`) to preserve it in the replacement.

## After running the codemod

1. Review the changes with `git diff`
2. Run `cargo fmt` to format the code
3. Run `cargo check` to ensure everything compiles
4. Update `Cargo.toml` to remove the `log` dependency if it's no longer needed
5. Test your application to ensure logging still works correctly

