//! data/tracks.dat: t x y id, space-separated, one sample per row.

use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sample {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub id: u32,
}

pub fn parse(text: &str) -> Result<Vec<Sample>, String> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .map(|(i, l)| {
            let f: Vec<&str> = l.split_whitespace().collect();
            if f.len() != 4 {
                return Err(format!("line {}: expected 4 fields, got {}", i + 1, f.len()));
            }
            let num = |s: &str| s.parse::<f64>().ok().filter(|v| v.is_finite()).ok_or(format!("line {}: bad number {s:?}", i + 1));
            Ok(Sample { t: num(f[0])?, x: num(f[1])?, y: num(f[2])?, id: f[3].parse().map_err(|e| format!("line {}: {e}", i + 1))? })
        })
        .collect()
}

/// Samples grouped by track id, each in time order.
pub fn by_track(samples: &[Sample]) -> BTreeMap<u32, Vec<Sample>> {
    let mut m: BTreeMap<u32, Vec<Sample>> = BTreeMap::new();
    for s in samples {
        m.entry(s.id).or_default().push(*s);
    }
    for v in m.values_mut() {
        v.sort_by(|a, b| a.t.total_cmp(&b.t));
    }
    m
}

/// A track that never moves (every sample at the same point).
pub fn is_stationary(track: &[Sample]) -> bool {
    track.windows(2).all(|w| w[0].x == w[1].x && w[0].y == w[1].y)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = include_str!("../../data/tracks.dat");

    #[test]
    fn shape_matches_the_data_readme() {
        let s = parse(FILE).unwrap();
        assert_eq!(s.len(), 48);
        let t = by_track(&s);
        let counts: Vec<(u32, usize)> = t.iter().map(|(k, v)| (*k, v.len())).collect();
        assert_eq!(counts, vec![(1, 15), (2, 14), (3, 11), (7, 8)]);
        assert!(is_stationary(&t[&7]));
        assert!(!is_stationary(&t[&1]));
        assert!(s.iter().all(|p| (-280.0..=280.0).contains(&p.x) && (-180.0..=172.0).contains(&p.y)));
    }

    #[test]
    fn rejects_bad_rows() {
        assert!(parse("1 2 3").is_err());
        assert!(parse("1 2 NaN 4").is_err());
        assert!(parse("1 2 3 x").is_err());
    }
}
