// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/lib.rs
// About: ==== Generic Company Inc. portfolio ===============================================...
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

// ==== Generic Company Inc. portfolio ===============================================
// Project: data_dash | File: src/lib.rs
// About: Generic Company Inc. data dashboard.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Generic Company Inc. data dashboard. `core` is plain Rust (tested on the host); `web` binds it to the page;
//! `host` (feature `desktop`) is the native window that shows that page over the shared database.

pub mod core;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(all(feature = "desktop", windows))]
pub mod host;

// ==== stamp v1 | author=kjFriday | hmac-sha256=c20f1c5a62abc8a82a75a8b720c11dcfb593cb5e65e0a9dfc0b3270b5bd9baec ====
