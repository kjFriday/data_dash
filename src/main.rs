// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/main.rs
// About: Entry point of generic-company-dash.exe, the dashboard's desktop window.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Entry point of generic-company-dash.exe, the dashboard's desktop window.
//!
//! generic-company-dash: Generic Company Data Dash as a desktop program (build with `--features desktop`).
//!
//! Everything shared with the other Generic Company Rust program (the ticket tracker) lives in
//! `host`: flag parsing, the installer, the database choice and the WebView2 window that serves www/
//! and `/api` over its own `generic-company://` protocol. This file only fills in the `Spec` of what
//! differs: names, window size, the tables the page may read, the icon and the smoke check.
//! `windows_subsystem = "windows"` links the exe as a GUI program, so no console window opens.
#![windows_subsystem = "windows"]

use data_dash::host::{self, api::Tables, Spec};

/// Hands this program's `Spec` to `host::main`, which handles the flags and opens the window or runs
/// the installer.
fn main() {
    host::main(Spec {
        app_id: "GenericCompany.Dash",
        name: "Generic Company Data Dash",
        exe_name: "generic-company-dash.exe",
        version: env!("CARGO_PKG_VERSION"),
        width: 1200.0,
        height: 820.0,
        // the tables and columns the page may read; `api::check_schema` also requires each of them
        // (plus `id`) in the chosen database file, and a file that lacks one falls back to mock mode
        tables: Tables {
            read: &[
                ("orders", &["customer_id", "product_id", "quantity", "total", "ordered_at"]),
                ("products", &["name", "category", "price"]),
                ("customers", &["name", "email", "city"]),
                ("tickets", &["customer_id", "assignee_id", "title", "body", "status", "created_at"]),
            ],
            write: None, // the dashboard only reads
        },
        // opens the database with SQLite's read-only flag, so even a bug cannot write to it
        read_only: true,
        // icon/ is written by the portfolio's pixel icon tool: the .ico goes to the installer; the raw
        // RGBA (width and height as little-endian u32s, then pixels) is the window icon's fallback when
        // the exe's icon resource 1 cannot be loaded
        icon_ico: include_bytes!("../icon/app.ico"),
        icon_rgba: include_bytes!("../icon/app-32.rgba"),
        // --smoke: 0 until the footer status shows a finished load (web.rs writes "Shared database ..."
        // or "... mock snapshot ..."; the match is case-sensitive), then the number of charts on the
        // Business tab
        smoke_count: "(document.getElementById('status').textContent.includes('Shared database') || document.getElementById('status').textContent.includes('mock snapshot')) ? document.querySelectorAll('#business svg').length : 0",
    });
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=754c8fd90e62514ba28c237467827f089cbd793a311c2143c1d19d18f861cf0d ====
