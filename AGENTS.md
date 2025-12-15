# AGENTS.md

## Build/Lint/Test Commands

### Full App (Standard)

- **Dev**: `pnpm dev` (runs Tauri dev mode)

### Frontend (SvelteKit + TypeScript)

- **Build**: `pnpm build`
- **Dev server**: `pnpm dev`
- **Type check**: `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json`
- **Watch type check**: `svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch`

### Backend (Rust + Tauri)

- **Build**: `cargo build`
- **Check**: `cargo check`
- **Lint**: `cargo clippy`
- **Test**: `cargo test`
- **Run single test**: `cargo test <test_name>`
- **Generate TypeScript bindings**: `TS_RS_EXPORT_DIR="../src/types/bindings" cargo test export_bindings --manifest-path="./src-tauri/Cargo.toml"`

## Code Style Guidelines

### TypeScript/Svelte

- **Strict TypeScript**: `"strict": true` enabled
- **Imports**: At top of file, before exports
- **Naming**: camelCase for variables/functions, PascalCase for types/interfaces
- **Modules**: ES modules only
- **Styling**: TailwindCSS + DaisyUI
- **Framework**: SvelteKit in SPA mode (SSR disabled for Tauri)
- **Error handling**: Standard try/catch with console.error logging
- **Responsibility**: Minimal logic & data handling, should play as stateless dumb client

### Rust

- **Error handling**: `thiserror::Error` derive with descriptive error messages
- **Logging**: `tracing` crate for structured logging (info/warn/error)
- **Async**: `tokio` runtime with `async`/`await`
- **Serialization**: `serde` with `Serialize`/`Deserialize`
- **Naming**: snake_case for functions/variables, PascalCase for types/structs
- **Documentation**: Comprehensive doc comments with examples for public APIs
- **State management**: Custom macros (`lock_r!`/`lock_w!`) for thread-safe access
- **Security**: Use secure storage (keyring) for sensitive data, proper PKCE flow for OAuth
- **Imports**: Group by standard library, then external crates, then local modules
- **Responsibility**: Handles app state & data, business logic, controls UI via events.

## Note

Do not run the app yourself. Detect errors via LSPs if available, `cd src-tauri && cargo check` & `pnpm check` else.
