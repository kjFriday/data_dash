//! ACME Inc. data dashboard. `core` is plain Rust (tested on the host); `web` binds it to the page.

pub mod core;

#[cfg(target_arch = "wasm32")]
mod web;
