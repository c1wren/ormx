# Ormx AGENTS.md

## Project Overview
Ormx is a Rust procedural macro library that provides database entity derive macros. It generates CRUD operations (Get, Insert, Update, Patch, Delete) for structs annotated with `#[derive(Entity)]`, primarily for use with SQLx and PostgreSQL.

## Build Commands

### Compile
```bash
cargo check            # Fast compilation check
cargo build            # Build all packages
cargo build --release  # Release build
```

### Test
```bash
cargo test                     # Run all tests
cargo test --package ormx      # Test specific package
cargo test --package example   # Run example tests
cargo test -- test_name        # Run single test by name
cargo test --no-run            # Compile tests without running
```

### Lint
```bash
cargo clippy              # Run Clippy linter
cargo clippy -- --allow clippy::needless_lifetimes
```

### Format
```bash
cargo fmt           # Format all code
rustfmt --check     # Check formatting without modifying
```

### Check Only
```bash
cargo check --lib            # Check library only
cargo check --bin example    # Check example binary
```

## Code Style Guidelines

### General
- **Language**: Rust 2021 edition
- **Target**: PostgreSQL with SQLx
- **Error handling**: Use `anyhow::Result`, `thiserror` for custom errors
- **Async**: Tokio runtime

### Naming Conventions
- **Types**: PascalCase (e.g., `Club`, `InsertClub`, `PatchClub`)
- **Functions/Methods**: snake_case (e.g., `get_by_id`, `update_name`)
- **Macros**: PascalCase with `derive` prefix (e.g., `#[derive(Entity)]`)
- **Attributes**: snake_case in `#[ormx(...)]` (e.g., `update = "my_update"`)

### Imports
- Standard library imports first
- External crates (serde, sqlx, anyhow) next
- Local module imports last
- Use explicit imports, avoid `use crate::*`

### Formatting
- 4-space indentation
- Max line length 100 characters
- Single blank line between top-level items
- No trailing whitespace
- Group imports with blank lines between categories

### Attributes
- **Entity attributes**: `table`, `update`, `insertable`, `patchable`, `deletable`, `get_all`, `context_type`, `error_type`
- **Field attributes**: `key`, `rename`, `get_one`, `get_optional`, `get_many`, `set`, `delete`, `default`, `custom_type`, `patchable`, `updatable`, `convert`, `convert_as`
- **Lifecycle hooks**: `before_patch`, `after_patch`, `before_update`, `after_update`, `before_insert`, `after_insert`, `before_delete`, `after_delete`

### Types
- Use `sqlx::Type` derive for custom enum types
- Use `#[ormx(custom_type)]` for SQLx type inference
- Use `convert` or `convert_as` for complex type transformations
- Use `Option<Vec<T>>` with explicit convert function for array types

### Error Handling
- Return `Result<T, TestError>` for lifecycle hooks
- Use `anyhow::Result` for main functions
- Define custom error types with `thiserror` or similar
- Pass context via `Option<&ContextType>` parameter

### Documentation
- Document public structs and enums
- Document lifecycle hook parameters
- Include examples where complex configuration is used

### Testing
- Tests are integrated in example crate
- Run with `cargo build --package example`
- Database required: `postgres://postgres@127.0.0.1/ormx`
- Use `.env` file for configuration (ignored in git)
