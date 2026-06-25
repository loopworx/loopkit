# loopkit

The Loop Language compiler — validates skill repositories against the Loop Language spec.

## Verification Commands

**Always use `+stable` toolchain** to match CI (which uses `dtolnay/rust-toolchain@stable`).

```bash
cargo +stable clippy --workspace --all-targets -- -D warnings
cargo +stable fmt --all -- --check
cargo +stable test --workspace
```

All three must pass before committing. Local nightly toolchain may not catch all lints that stable clippy enforces.
