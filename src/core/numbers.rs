// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/numbers.rs
// About: Parser and statistics for data/numbers.dat, the integers 1 to 100 as Wikidata tags them.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Parser and statistics for data/numbers.dat, the integers 1 to 100 as Wikidata tags them.
//!
//! data/numbers.dat: n TAB item TAB sitelinks TAB classes (| separated); `#` lines are comments.
//! data/README.md describes the columns and the intended charts. A sitelink is a link from a Wikidata
//! item to a page about it on one Wikimedia site (a Wikipedia language edition, Wikiquote ...), so
//! the count is a rough measure of how widely the number is written about. The classes are the item's
//! "instance of" (P31) values, which reflect what editors have tagged, not mathematical truth.
//!
//! `web.rs` parses the compiled-in file once at start; `render::fixtures` and `render::number_table`
//! call the statistics: `class_counts`, `prime_split`, `median_sitelinks_x2` and `most_linked`.

use std::collections::BTreeMap;

/// One line of numbers.dat.
#[derive(Debug, Clone, PartialEq)]
pub struct NumberRow {
    /// The number itself, 1 to 100.
    pub n: u32,
    /// Its Wikidata item id ("Q199" for 1).
    pub item: String,
    /// How many Wikimedia sites link the item.
    pub sitelinks: u32,
    /// Class labels in file order; empty pieces between `|` are dropped.
    pub classes: Vec<String>,
}

/// Parses numbers.dat -> one `NumberRow` per data line, in file order.
///
/// Blank lines and lines starting with `#` are skipped; a trailing `\r` is dropped, since the file has
/// CRLF line endings. Errors: "line N: ..." for a line without exactly 4 tab-separated fields or with
/// an `n` or `sitelinks` that is not a `u32`. N is the line number in the file, comments included, because lines are
/// numbered before the filter.
pub fn parse(text: &str) -> Result<Vec<NumberRow>, String> {
    text.lines()
        .map(|l| l.trim_end_matches('\r'))
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|(i, l)| {
            let f: Vec<&str> = l.split('\t').collect();
            if f.len() != 4 {
                return Err(format!("line {}: expected 4 fields, got {}", i + 1, f.len()));
            }
            let num = |s: &str| s.trim().parse::<u32>().map_err(|e| format!("line {}: {e}", i + 1));
            Ok(NumberRow {
                n: num(f[0])?,
                item: f[1].to_string(),
                sitelinks: num(f[2])?,
                classes: f[3].split('|').filter(|c| !c.is_empty()).map(String::from).collect(),
            })
        })
        .collect()
}

/// (class, rows tagged) sorted by count desc, then name.
pub fn class_counts(rows: &[NumberRow]) -> Vec<(String, usize)> {
    let mut m: BTreeMap<&str, usize> = BTreeMap::new();
    for r in rows {
        for c in &r.classes {
            *m.entry(c).or_default() += 1;
        }
    }
    let mut v: Vec<(String, usize)> = m.into_iter().map(|(k, n)| (k.to_string(), n)).collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

/// (prime, composite, neither) by Wikidata tag.
///
/// A row tagged both "prime number" and "composite number" counts as prime. "Neither" is every other
/// row; in the fixture that is only 1, which is neither prime nor composite.
pub fn prime_split(rows: &[NumberRow]) -> (usize, usize, usize) {
    let has = |r: &NumberRow, c: &str| r.classes.iter().any(|x| x == c);
    let p = rows.iter().filter(|r| has(r, "prime number")).count();
    let c = rows
        .iter()
        .filter(|r| !has(r, "prime number") && has(r, "composite number"))
        .count();
    (p, c, rows.len() - p - c)
}

/// Median sitelinks in half-units (2 × median), so an even count stays exact.
///
/// The median of an even count is the mean of the two middle values, which may end in .5; doubling
/// it keeps the result an integer. `render::fixtures` halves it again for display ("99" or "99.5").
/// None for no rows.
pub fn median_sitelinks_x2(rows: &[NumberRow]) -> Option<u32> {
    let mut v: Vec<u32> = rows.iter().map(|r| r.sitelinks).collect();
    v.sort_unstable();
    let n = v.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(v[n / 2] * 2),
        _ => Some(v[n / 2 - 1] + v[n / 2]),
    }
}

/// The row with the most sitelinks, or None for no rows; a tie goes to the smallest `n`.
pub fn most_linked(rows: &[NumberRow]) -> Option<&NumberRow> {
    // `max_by` keeps the last of equal maxima, so the tie-break compares `n` reversed to rank a
    // smaller n higher
    rows.iter()
        .max_by(|a, b| a.sitelinks.cmp(&b.sitelinks).then(b.n.cmp(&a.n)))
}

/// Unit tests against the real fixture file and small hand-made rows.
#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture, compiled in from data/.
    const FILE: &str = include_str!("../../data/numbers.dat");

    /// Row count, class count, top class counts, the 25/74/1 split and 3-59 classes per row are the
    /// figures data/README.md states; a change to the file or the parser shows up here.
    #[test]
    fn shape_matches_the_data_readme() {
        let rows = parse(FILE).unwrap();
        assert_eq!(rows.len(), 100);
        assert_eq!(
            rows.iter().map(|r| r.n).collect::<Vec<_>>(),
            (1..=100).collect::<Vec<_>>()
        );
        let counts = class_counts(&rows);
        assert_eq!(counts.len(), 144);
        let get = |c: &str| counts.iter().find(|(k, _)| k == c).map(|x| x.1);
        assert_eq!(counts[0], ("positive integer".to_string(), 100));
        assert_eq!(get("composite number"), Some(74));
        assert_eq!(get("odd number"), Some(50));
        assert_eq!(get("even number"), Some(50));
        assert_eq!(get("harshad number"), Some(33));
        assert_eq!(get("prime number"), Some(25));
        assert_eq!(prime_split(&rows), (25, 74, 1));
        let per_row: Vec<usize> = rows.iter().map(|r| r.classes.len()).collect();
        assert_eq!((per_row.iter().min(), per_row.iter().max()), (Some(&3), Some(&59)));
    }

    /// 1 (Q199) is the most linked with 221, 100 has 142, and the median lies in the README's 73-221 range.
    #[test]
    fn sitelink_stats() {
        let rows = parse(FILE).unwrap();
        let top = most_linked(&rows).unwrap();
        assert_eq!((top.n, top.item.as_str(), top.sitelinks), (1, "Q199", 221));
        assert_eq!(rows.iter().find(|r| r.n == 100).unwrap().sitelinks, 142);
        let m = median_sitelinks_x2(&rows).unwrap();
        assert!((73 * 2..=221 * 2).contains(&m));
    }

    /// Odd count: 2 × the middle (unsorted input); even count: the sum of the two middles; empty: None.
    #[test]
    fn median_halves() {
        let row = |s| NumberRow {
            n: 1,
            item: String::new(),
            sitelinks: s,
            classes: vec![],
        };
        assert_eq!(median_sitelinks_x2(&[row(3), row(1), row(2)]), Some(4));
        assert_eq!(median_sitelinks_x2(&[row(1), row(2)]), Some(3));
        assert_eq!(median_sitelinks_x2(&[]), None);
    }

    /// A bad number and a short line are errors naming the line; comments and CRLF blanks parse to nothing.
    #[test]
    fn bad_lines_are_reported() {
        assert!(parse("1\tQ1\tx\ta").unwrap_err().contains("line 1"));
        assert!(parse("1\tQ1").is_err());
        assert_eq!(parse("# only a comment\r\n\r\n").unwrap().len(), 0);
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=a31c9cce7db4e822e265857d7c14d37ff0e42235e39c949d78490b9c580d9daf ====
