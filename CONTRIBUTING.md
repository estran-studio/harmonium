# Contributing to Harmonium

## AI Coding Guidelines

To ensure high code quality, stability, and performance, all AI-generated code must adhere to the following strict rules:

### 1. Error Handling
- **NEVER** use `.unwrap()`. It is strictly forbidden in production code.
- Always use `?` for error propagation where possible.
- If you must panic (e.g., in a test or unrecoverable startup state), use `.expect("detailed reason")` with a clear context message.
- Handle `Option` and `Result` types explicitly.

### 2. Performance & Memory
- **Pre-allocation**: Always pre-allocate vectors when the size is known or predictable (e.g., `Vec::with_capacity(n)`).
- **Allocations in Audio Thread**: strictly FORBIDDEN. No `Vec::new()`, `Box::new()`, or string manipulation in the audio processing hot path (`process_buffer`). Use pre-allocated buffers or lock-free queues.
- **Cloning**: Avoid unnecessary cloning (`.clone()`). Pass by reference where possible.

### 3. Code Style & Linting
- All code must pass `cargo clippy --workspace -- -D warnings`.
- All code must be formatted with `cargo fmt`.
- Respect the `[lints]` configuration in `Cargo.toml`.

## Workflow
1.  Run `make quality` before submitting any changes.
2.  Ensure no new warnings are introduced.
