// ==== ACME Inc. portfolio ===============================================
// Project: data_dash | File: src/lib.rs
// About: ACME Inc. data dashboard.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! ACME Inc. data dashboard. `core` is plain Rust (tested on the host); `web` binds it to the page;
//! `host` (feature `desktop`) is the native window that shows that page over the shared database.

pub mod core;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(all(feature = "desktop", windows))]
pub mod host;

// ==== stamp v1 | author=kjFriday | hmac-sha256=2c1b5885440bb3e476743ef58739130d5c5eea1062f21c0c9189f01d8fa5790a ====
