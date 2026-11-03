// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/main.rs
// About: generic-company-dash: Generic Company Data Dash as a desktop program (build with...
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! generic-company-dash: Generic Company Data Dash as a desktop program (build with `--features desktop`).
#![windows_subsystem = "windows"]

use data_dash::host::{self, api::Tables, Spec};

fn main() {
    host::main(Spec {
        app_id: "GenericCompany.Dash",
        name: "Generic Company Data Dash",
        exe_name: "generic-company-dash.exe",
        version: env!("CARGO_PKG_VERSION"),
        width: 1200.0,
        height: 820.0,
        tables: Tables {
            read: &[
                ("orders", &["customer_id", "product_id", "quantity", "total", "ordered_at"]),
                ("products", &["name", "category", "price"]),
                ("customers", &["name", "email", "city"]),
                ("tickets", &["customer_id", "assignee_id", "title", "body", "status", "created_at"]),
            ],
            write: None, // the dashboard only reads
        },
        read_only: true,
        icon_ico: include_bytes!("../icon/app.ico"),
        icon_rgba: include_bytes!("../icon/app-32.rgba"),
        smoke_count: "(document.getElementById('status').textContent.includes('Shared database') || document.getElementById('status').textContent.includes('mock snapshot')) ? document.querySelectorAll('#business svg').length : 0",
    });
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=a1b9c90108b1057258276df23edacacd7939b28a4c317c3096a7ddfe421aea56 ====
