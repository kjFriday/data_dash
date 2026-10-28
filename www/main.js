// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: www/main.js
// About: ==== Generic Company Inc. portfolio ===============================================...
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

// ==== Generic Company Inc. portfolio ===============================================
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

// ==== stamp v1 | author=kjFriday | hmac-sha256=5d3ac8a54e7ccfdd6de287d18f95c278a0581fb4d7655b17393d8fbbf196859b ====
