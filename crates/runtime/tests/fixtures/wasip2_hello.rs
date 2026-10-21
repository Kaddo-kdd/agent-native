//! Source pinned to the AN-050 real WASI Preview 2 fixture.
//!
//! Compile with Rust 1.85.0 and `--target wasm32-wasip2`; the resulting component
//! is checked by `wasi_real_binaries.rs` instead of being rebuilt in ordinary CI.

fn main() {
    println!("hello agent-native");
}
