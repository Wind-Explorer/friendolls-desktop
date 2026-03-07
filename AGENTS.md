# Friendolls

Passive social app connecting peers through mouse cursor interactions in the form of desktop pets.

# friendolls-desktop

Desktop client app for Friendolls.

## Commands

Check code integrity after every significant change:

- `cd src-tauri && cargo check` for Rust local backend
- `pnpm check` for Svelte frontend

Generate TypeScript bindings with `tauri-specta` when new tauri commands, events or Rust models is added / modified:

- `timeout 30 pnpm tauri dev`

### TypeScript/Svelte

- **Framework**: SvelteKit in SPA mode (SSR disabled for Tauri)
- **Styling**: TailwindCSS + DaisyUI
- **Responsibility**: Minimal logic & data handling, should play as stateless dumb client, communicate with Rust local backend via Tauri events

### Rust

- **Error handling**: `thiserror::Error` derive with descriptive error messages
- **Logging**: `tracing` crate for structured logging (info/warn/error)
- **Async**: `tokio` runtime with `async`/`await`
- **Naming**: snake_case for functions/variables, PascalCase for types/structs
- **State management**: Custom macros (`lock_r!`/`lock_w!`) for thread-safe access
- **Security**: Use secure storage (keyring) for sensitive data
- **Responsibility**: Handles app state & data, business logic, controls UI via events.

## Note

Be sure to gather sufficient context from codebase before proceeding with changes. Observe patterns and follow trends.

Do not run the app without timeout. `cd src-tauri && cargo check` & `pnpm check` to confirm your changes are error-free. Don't perform git actions yourself.
