// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/html.rs
// About: HTML escaping for the views and charts.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! HTML escaping for the views and charts. `render` and `svg` pass every value from data through `escape`.

/// `s` with `&`, `<`, `>`, `"` and `'` as entities, so it is safe as element text and as a quoted attribute value.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_covers_html_specials() {
        assert_eq!(
            escape(r#"<a href="x">'&'</a>"#),
            "&lt;a href=&quot;x&quot;&gt;&#39;&amp;&#39;&lt;/a&gt;"
        );
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=80020056da9d47d34210b0b0cac1fa2e2658f8fb5a8f1ce5dedf947e133359b1 ====
