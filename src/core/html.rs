// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/html.rs
// About: HTML escaping for the views and charts.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! HTML escaping for the views and charts. `render` and `svg` pass every value from data through `escape`.
//!
//! The page builds its markup as strings and hands them to `innerHTML` (see `web.rs`), so any text that
//! comes from data (product names, cities, Wikidata classes, the database path) could otherwise inject
//! tags or break out of an attribute. `escape` is the one gate for that text. Callers: `render.rs` (tiles,
//! cards, legends, tables) and `svg.rs` (labels, tooltips, tick text).

/// `s` with `&`, `<`, `>`, `"` and `'` as entities, so it is safe as element text and as a quoted attribute value.
///
/// `&` must become `&amp;` so an existing entity in the data shows literally. `'` uses the numeric form
/// `&#39;`, which every HTML version and SVG parser reads (`&apos;` is missing from HTML 4). Every other character, non-ASCII included, passes
/// through unchanged: the page is UTF-8.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

/// Unit tests for `escape`.
#[cfg(test)]
mod tests {
    use super::*;

    /// All five specials in one string, inside a tag and an attribute, become entities in place.
    #[test]
    fn escape_covers_html_specials() {
        assert_eq!(
            escape(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=3b27006ae9336e345ac2df7fb07e8b2a1520910c4c083c63eb895384666cdb55 ====
