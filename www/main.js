// ==== ACME Inc. portfolio ===============================================
// Project: data_dash | File: www/main.js
// About: Loads the Rust/WebAssembly build; everything else happens in src/web.rs.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

// Loads the Rust/WebAssembly build; everything else happens in src/web.rs.
import init from "./pkg/data_dash.js";

init().catch((err) => {
  document.getElementById("status").textContent = "Could not start: " + err;
});

// ==== stamp v1 | author=kjFriday | hmac-sha256=8d6647287167baaf1704a97798eca651274f6c22a446670d70c1021f33f2f09e ====
