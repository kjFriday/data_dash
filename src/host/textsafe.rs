// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/host/textsafe.rs
// About: Text cleaning for stored strings, matching the CRUD hub's textsafe rules.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Text cleaning for stored strings, matching the CRUD hub's textsafe rules.
//!
//! The CRUD hub's textsafe rules (copied from the ticket tracker's `core::validate::clean`), used by the
//! shared API code before anything is stored. Data Dash itself only reads.
//!
//! `host/api.rs` (shared with the ticket tracker) runs every string a write sends through `clean`.
//! This program's `Spec` makes no table writable, so the code compiles but never runs here; it is kept
//! so `api.rs` stays the same file in both programs. Keep the rules in step with the hub and with
//! the tracker's `core/validate.rs`.

/// textsafe.LIMIT in the hub: string columns are capped at this many characters.
pub const LIMIT: usize = 240;

/// Drop the characters textsafe refuses (C0 except tab/newline, DEL, C1, U+2028/9, bidi controls), then cap.
///
/// The cap counts characters (Unicode scalar values), not bytes, so a multibyte character is never
/// split; it applies after the drop, so refused characters do not use up the limit.
pub fn clean(text: &str, limit: usize) -> String {
    text.chars().filter(|&c| !refused(c)).take(limit).collect()
}

/// Whether textsafe drops `c`.
///
/// The refused ranges, and why:
/// - C0 controls U+0000-001F except tab and newline: invisible, and NUL or ESC can upset other tools.
/// - DEL U+007F and the C1 controls U+0080-009F: invisible legacy controls (0x9B is a terminal escape).
/// - U+2028 LINE SEPARATOR and U+2029 PARAGRAPH SEPARATOR: line breaks many tools do not expect.
/// - U+202A-202E (embeddings and overrides) and U+2066-2069 (isolates): bidirectional controls that
///   can make text display in a different order than it is stored.
fn refused(c: char) -> bool {
    let u = c as u32;
    (u < 0x20 && c != '\t' && c != '\n')
        || (0x7F..=0x9F).contains(&u)
        || u == 0x2028
        || u == 0x2029
        || (0x202A..=0x202E).contains(&u)
        || (0x2066..=0x2069).contains(&u)
}

/// Unit test for `clean`.
#[cfg(test)]
mod tests {
    use super::*;

    /// BEL and a right-to-left override go, a tab stays, and 300 characters cap at `LIMIT`.
    #[test]
    fn drops_controls_and_caps() {
        assert_eq!(clean("a\u{0007}b\tc\u{202e}d", 240), "ab\tcd");
        assert_eq!(clean(&"x".repeat(300), LIMIT).len(), 240);
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=7f533e47e5a58a9c5e30510b929ab9d1456133f27b54cdba8f4e32d90f432121 ====
