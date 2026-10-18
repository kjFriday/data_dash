// ==== ACME Inc. portfolio ===============================================
// Project: data_dash | File: src/host/textsafe.rs
// About: The CRUD hub's textsafe rules (copied from the ticket tracker's `core::validate::clean`)...
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! The CRUD hub's textsafe rules (copied from the ticket tracker's `core::validate::clean`), used by the
//! shared API code before anything is stored. Data Dash itself only reads.

/// textsafe.LIMIT in the hub: string columns are capped at this many characters.
pub const LIMIT: usize = 240;

/// Drop the characters textsafe refuses (C0 except tab/newline, DEL, C1, U+2028/9, bidi controls), then cap.
pub fn clean(text: &str, limit: usize) -> String {
    text.chars().filter(|&c| !refused(c)).take(limit).collect()
}

fn refused(c: char) -> bool {
    let u = c as u32;
    (u < 0x20 && c != '\t' && c != '\n')
        || (0x7F..=0x9F).contains(&u)
        || u == 0x2028
        || u == 0x2029
        || (0x202A..=0x202E).contains(&u)
        || (0x2066..=0x2069).contains(&u)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn drops_controls_and_caps() {
        assert_eq!(clean("a\u{0007}b\tc\u{202e}d", 240), "ab\tcd");
        assert_eq!(clean(&"x".repeat(300), LIMIT).len(), 240);
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=1c04e8ad2026ffe4af39af9909c4dcc643bb090b7a809bfd1aa4cc02c7d0ae28 ====
