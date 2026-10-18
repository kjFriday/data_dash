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
