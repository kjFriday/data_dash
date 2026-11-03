// ==== Generic Company Inc. portfolio ====================================
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

// ==== stamp v1 | author=kjFriday | hmac-sha256=9fdd0ee57ca8407b1c66d7a41508566f7b0e870484cf5a87dfe86ec3eca2e619 ====
