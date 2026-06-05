// Library crate re-exports for integration testing
// The main application logic lives in main.rs; this lib.rs exists to allow
// `tests/` integration tests to import types from `mcl-rs`.

pub mod core;
pub mod config;
pub mod utils;
