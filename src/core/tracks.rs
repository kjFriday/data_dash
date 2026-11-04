// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/tracks.rs
// About: data/tracks.dat: t x y id, space-separated, one sample per row.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! data/tracks.dat: t x y id, space-separated, one sample per row.
//!
//! The fixture holds 48 position samples: three moving tracks (1, 2, 3) and one stationary point
//! (track 7), as data/README.md describes. `web.rs` parses the compiled-in file once at start;
//! `render::fixtures` groups the samples with `by_track`, draws each track as a path, and labels a
//! track that never moves with `is_stationary`.

use std::collections::BTreeMap;

/// One position sample: track `id` was at (`x`, `y`) at time step `t`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    /// Time step, from 0 within each track.
    pub t: f64,
    /// Horizontal position (-280 to 280 in the fixture).
    pub x: f64,
    /// Vertical position (-180 to 172 in the fixture); `svg::paths` draws a larger y higher up.
    pub y: f64,
    pub id: u32,
}

/// Parses tracks.dat -> one `Sample` per data line, in file order.
///
/// Fields split on any run of whitespace, so the CRLF line endings need no special case. Blank lines
/// and lines whose first non-blank character is `#` are skipped. Errors: "line N: ..." for a line
/// without exactly 4 fields, a number that does not parse or is NaN or infinite (the charts would
/// print "NaN"), or an id that is not a `u32`. N counts every line of the file.
pub fn parse(text: &str) -> Result<Vec<Sample>, String> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|(i, l)| {
            let f: Vec<&str> = l.split_whitespace().collect();
            if f.len() != 4 {
                return Err(format!("line {}: expected 4 fields, got {}", i + 1, f.len()));
            }
            let num = |s: &str| {
                s.parse::<f64>()
                    .ok()
                    .filter(|v| v.is_finite())
                    .ok_or(format!("line {}: bad number {s:?}", i + 1))
            };
            Ok(Sample {
                t: num(f[0])?,
                x: num(f[1])?,
                y: num(f[2])?,
                id: f[3].parse().map_err(|e| format!("line {}: {e}", i + 1))?,
            })
        })
        .collect()
}

/// Samples grouped by track id, each in time order.
///
/// The `BTreeMap` keeps the ids ascending, which fixes the legend order and which tracks get the
/// first color slots in `render::fixtures`.
pub fn by_track(samples: &[Sample]) -> BTreeMap<u32, Vec<Sample>> {
    let mut m: BTreeMap<u32, Vec<Sample>> = BTreeMap::new();
    for s in samples {
        m.entry(s.id).or_default().push(*s);
    }
    // total_cmp: f64 has no total order for sort_by; parse already refused NaN, so this is plain numeric order
    for v in m.values_mut() {
        v.sort_by(|a, b| a.t.total_cmp(&b.t));
    }
    m
}

/// A track that never moves (every sample at the same point).
///
/// Compares each neighbouring pair with exact float equality, which suits a fixture that repeats
/// the same printed coordinates. A track of 0 or 1 samples counts as stationary.
pub fn is_stationary(track: &[Sample]) -> bool {
    track.windows(2).all(|w| w[0].x == w[1].x && w[0].y == w[1].y)
}

/// Unit tests against the real fixture file and bad rows.
#[cfg(test)]
mod tests {
    use super::*;

    /// The fixture, compiled in from data/.
    const FILE: &str = include_str!("../../data/tracks.dat");

    /// 48 samples, the per-track counts 15/14/11/8, track 7 fixed, and the x/y ranges are the figures
    /// data/README.md states.
    #[test]
    fn shape_matches_the_data_readme() {
        let s = parse(FILE).unwrap();
        assert_eq!(s.len(), 48);
        let t = by_track(&s);
        let counts: Vec<(u32, usize)> = t.iter().map(|(k, v)| (*k, v.len())).collect();
        assert_eq!(counts, vec![(1, 15), (2, 14), (3, 11), (7, 8)]);
        assert!(is_stationary(&t[&7]));
        assert!(!is_stationary(&t[&1]));
        assert!(s
            .iter()
            .all(|p| (-280.0..=280.0).contains(&p.x) && (-180.0..=172.0).contains(&p.y)));
    }

    /// Too few fields, a NaN coordinate and a non-integer id are each an error.
    #[test]
    fn rejects_bad_rows() {
        assert!(parse("1 2 3").is_err());
        assert!(parse("1 2 NaN 4").is_err());
        assert!(parse("1 2 3 x").is_err());
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=2b003a52710eee85ec82fd8ae2285ada8658d4f146500e8afd4661917d156343 ====
