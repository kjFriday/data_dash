// ==== ACME Inc. portfolio ===============================================
// Project: data_dash | File: src/core/numbers.rs
// About: data/numbers.dat: n TAB item TAB sitelinks TAB classes (| separated); `#` lines are...
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! data/numbers.dat: n TAB item TAB sitelinks TAB classes (| separated); `#` lines are comments.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct NumberRow {
    pub n: u32,
    pub item: String,
    pub sitelinks: u32,
    pub classes: Vec<String>,
}

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
pub fn prime_split(rows: &[NumberRow]) -> (usize, usize, usize) {
    let has = |r: &NumberRow, c: &str| r.classes.iter().any(|x| x == c);
    let p = rows.iter().filter(|r| has(r, "prime number")).count();
    let c = rows.iter().filter(|r| !has(r, "prime number") && has(r, "composite number")).count();
    (p, c, rows.len() - p - c)
}

/// Median sitelinks in half-units (2 × median), so an even count stays exact.
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

pub fn most_linked(rows: &[NumberRow]) -> Option<&NumberRow> {
    rows.iter().max_by(|a, b| a.sitelinks.cmp(&b.sitelinks).then(b.n.cmp(&a.n)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = include_str!("../../data/numbers.dat");

    #[test]
    fn shape_matches_the_data_readme() {
        let rows = parse(FILE).unwrap();
        assert_eq!(rows.len(), 100);
        assert_eq!(rows.iter().map(|r| r.n).collect::<Vec<_>>(), (1..=100).collect::<Vec<_>>());
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

    #[test]
    fn sitelink_stats() {
        let rows = parse(FILE).unwrap();
        let top = most_linked(&rows).unwrap();
        assert_eq!((top.n, top.item.as_str(), top.sitelinks), (1, "Q199", 221));
        assert_eq!(rows.iter().find(|r| r.n == 100).unwrap().sitelinks, 142);
        let m = median_sitelinks_x2(&rows).unwrap();
        assert!((73 * 2..=221 * 2).contains(&m));
    }

    #[test]
    fn median_halves() {
        let row = |s| NumberRow { n: 1, item: String::new(), sitelinks: s, classes: vec![] };
        assert_eq!(median_sitelinks_x2(&[row(3), row(1), row(2)]), Some(4));
        assert_eq!(median_sitelinks_x2(&[row(1), row(2)]), Some(3));
        assert_eq!(median_sitelinks_x2(&[]), None);
    }

    #[test]
    fn bad_lines_are_reported() {
        assert!(parse("1\tQ1\tx\ta").unwrap_err().contains("line 1"));
        assert!(parse("1\tQ1").is_err());
        assert_eq!(parse("# only a comment\r\n\r\n").unwrap().len(), 0);
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=9337f51dc8bec9708019f0964455e5303f0da296d41d6b7eb20f68911e03d60c ====
