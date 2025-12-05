# Friendolls (Desktop)

Run the following command in project root on first run & after changes to models on Rust side to generate TypeScript type bindings from Rust models

```sh
# unix
TS_RS_EXPORT_DIR="../src/types/bindings" cargo test export_bindings --manifest-path=./src-tauri/Cargo.toml
```

```sh
# powershell
$Env:TS_RS_EXPORT_DIR = "../src/types/bindings"; cargo test export_bindings --manifest-path=./src-tauri/Cargo.toml
```
