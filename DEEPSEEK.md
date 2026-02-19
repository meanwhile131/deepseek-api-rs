# DeepSeek API Client Project

## Overview
This Rust crate provides an asynchronous client for the DeepSeek chat API, including automatic Proof of Work (PoW) solving via a WebAssembly module. It supports both streaming and non-streaming completions.

## Key Modules
- `src/lib.rs` – Main client (`DeepSeekAPI`) with methods to create chats, get info, and complete messages (streaming/non‑streaming).
- `src/models.rs` – Data structures: `Message`, `ChatSession`, `StreamingUpdate`, `StreamingMessageBuilder`.
- `src/pow_solver.rs` – PoW solver using wasmtime; loads a WASM module to compute challenge answers.
- `src/wasm_download.rs` – Downloads and caches the required WASM file.

## Conventions & Important Notes
- All public functions returning `Result` have `# Errors` doc sections.
- Functions that may panic have `# Panics` doc sections (where known).
- Casting between `usize` and `i32` is handled safely with `try_from` and context; where truncation is intentional, `#[allow(clippy::cast_possible_truncation)]` is used.
- Debug formatting of paths uses `display()` instead of `:?` to avoid unnecessary quotes/escaping.
- The `Challenge` struct has `#[allow(clippy::struct_field_names)]` because the field `challenge` naturally repeats the struct name.
- The `complete` and `complete_stream` methods are lengthy (exceeding 100 lines) – refactoring is deferred but noted.

## Recent Changes
- Added missing `# Errors` and `# Panics` doc comments.
- Replaced `:?` formatting with `display()` for paths.
- Handled `usize`/`i32` casts with `try_from` and proper error handling.
- Added necessary `#[allow]` attributes for intentional casts and struct naming.
- All clippy pedantic warnings have been addressed (verified with `cargo clippy -- -W clippy::pedantic`).

## Running Clippy
To check for lint issues:
```bash
cargo clippy -- -W clippy::pedantic
```