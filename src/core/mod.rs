// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/mod.rs
// About: The dashboard's pure core: parsing, metrics and HTML/SVG rendering, with no I/O.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! The dashboard's pure core: parsing, metrics and HTML/SVG rendering, with no I/O.
//!
//! `lib.rs` compiles this module on every target, so `cargo test` on the host covers all of it. The
//! wasm page (`web.rs`) is its only caller at run time: it feeds rows in and sets the returned HTML
//! strings as `innerHTML`. Data flows one way:
//!
//! 1. `metrics` (database or mock JSON rows), `numbers` (data/numbers.dat) and `tracks`
//!    (data/tracks.dat) parse and aggregate.
//! 2. `svg` draws each chart as a string.
//! 3. `render` lays tiles, cards, charts and tables out as the two views.
//! 4. `html::escape` guards every value from data on the way into markup.
//!
//! Start reading at `render::business` and `render::fixtures`.

/// HTML escaping shared by `render` and `svg`.
pub mod html;
/// Order, product, customer and ticket rows and the business metrics over them (money in cents).
pub mod metrics;
/// The data/numbers.dat parser and its class and sitelink statistics.
pub mod numbers;
/// The two views (Business, Numbers & tracks) and the number table, as HTML strings.
pub mod render;
/// Hand-built SVG chart primitives: bars, columns, donut, line and x/y paths.
pub mod svg;
/// The data/tracks.dat parser and per-track grouping.
pub mod tracks;

// ==== stamp v1 | author=kjFriday | hmac-sha256=5626ad43161b08aae4f4ff916d8738e564b11ffff02f11b85ed02d76ae803b93 ====
