# Friendolls (Desktop)

This repository contins source for Friendolls desktop app. Will add more info when the app scales.

Run the following command in project root after changes to models on Rust side to generate TypeScript type bindings from Rust models

```sh
# average unix shells
TS_RS_EXPORT_DIR="../src/types/bindings" cargo test export_bindings --manifest-path="./src-tauri/Cargo.toml"
```

```sh
# powershell
$Env:TS_RS_EXPORT_DIR = "../src/types/bindings"; cargo test export_bindings --manifest-path="./src-tauri/Cargo.toml"
```

> _To the gods of programming, please grant me the perseverance to push through and get this app into production_ 🙏
