// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: www/main.js
// About: Loads the Rust/WebAssembly build; everything else happens in src/web.rs.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

// Loads the Rust/WebAssembly build; everything else happens in src/web.rs.
//
// index.html loads this file as an ES module. ./pkg/ticket_tracker.js is the glue wasm-pack writes
// (wasm-pack build --target web --out-dir www/pkg); its default export fetches and instantiates
// ticket_tracker_bg.wasm beside it, then runs the Rust function marked #[wasm_bindgen(start)]
// (web.rs `start`), which wires up the whole page.
import init from "./pkg/data_dash.js";

// a failed fetch of the wasm, an error returned by start, or a panic during start rejects the promise;
// the status bar is the one place the user will see it
init().catch((err) => {
  document.getElementById("status").textContent = "Could not start: " + err;
});

// ==== stamp v1 | author=kjFriday | hmac-sha256=92a005daed4eba217a6b32012d3fb8a63766ea4a52bcb1bedf7301cf5ad47f7f ====
