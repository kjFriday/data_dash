//! Hand-built SVG charts. Colors are CSS custom properties (see www/style.css), so light and dark
//! steps swap in one place. Marks follow one spec: bars <= 24px with a 4px rounded data end and a
//! square baseline, 2px lines, >= 8px markers with a 2px surface ring, hairline grid, and a
//! `<title>` on every mark for the hover tooltip. Text never wears a series color.

use super::render::escape;

/// One categorical slot (1-based) as a CSS variable.
pub fn slot(n: usize) -> String {
    format!("var(--series-{n})")
}

/// Clean axis ticks from 0 to at least `max`: steps of 1, 2 or 5 × 10^k, about `want` of them.
pub fn ticks(max: f64, want: usize) -> Vec<f64> {
    let max = if max.is_finite() && max > 0.0 { max } else { 1.0 };
    let raw = max / want.max(1) as f64;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0].iter().map(|m| m * mag).find(|s| *s >= raw).unwrap_or(10.0 * mag);
    let top = (max / step).ceil() as usize;
    (0..=top).map(|i| i as f64 * step).collect()
}

fn f(v: f64) -> String {
    let v = if v.is_finite() { v } else { 0.0 };
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(String::from).unwrap_or(s)
}

/// A bar from the baseline with only its data end rounded (r = 4, or less for a short bar).
fn hbar_path(x: f64, y: f64, w: f64, h: f64) -> String {
    let r = 4f64.min(w / 2.0).min(h / 2.0).max(0.0);
    format!(
        "M{} {}H{}Q{} {} {} {}V{}Q{} {} {} {}H{}Z",
        f(x), f(y), f(x + w - r), f(x + w), f(y), f(x + w), f(y + r), f(y + h - r), f(x + w), f(y + h), f(x + w - r), f(y + h), f(x)
    )
}

fn vbar_path(x: f64, base: f64, w: f64, h: f64) -> String {
    let r = 4f64.min(w / 2.0).min(h / 2.0).max(0.0);
    let top = base - h;
    format!(
        "M{} {}V{}Q{} {} {} {}H{}Q{} {} {} {}V{}Z",
        f(x), f(base), f(top + r), f(x), f(top), f(x + r), f(top), f(x + w - r), f(x + w), f(top), f(x + w), f(top + r), f(base)
    )
}

pub struct Bar {
    pub label: String,
    pub value: f64,
    pub shown: String, // the value as printed at the tip and in the tooltip
}

/// Horizontal bars, one series: label on the left, value at the tip.
pub fn hbars(bars: &[Bar], width: f64, label_w: f64) -> String {
    let row = 26.0;
    let thick = 16.0;
    let value_w = 64.0;
    let plot = (width - label_w - value_w).max(10.0);
    let max = bars.iter().map(|b| b.value).fold(0.0, f64::max);
    let max = if max > 0.0 { max } else { 1.0 };
    let h = row * bars.len() as f64 + 8.0;
    let mut s = format!(r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#, f(width), f(h));
    s.push_str(&format!(r#"<line class="axis" x1="{x}" x2="{x}" y1="0" y2="{}"/>"#, f(h - 4.0), x = f(label_w)));
    for (i, b) in bars.iter().enumerate() {
        let y = 4.0 + i as f64 * row;
        let w = (b.value.max(0.0) / max) * plot;
        s.push_str(&format!(
            r#"<g class="mark"><title>{l}: {v}</title><rect class="hit" x="0" y="{y}" width="{W}" height="{row}"/><text class="label" x="{lx}" y="{ty}" text-anchor="end">{l}</text><path d="{p}" fill="{c}"/><text class="value" x="{vx}" y="{ty}">{v}</text></g>"#,
            l = escape(&b.label),
            v = escape(&b.shown),
            y = f(y),
            W = f(width),
            row = f(row),
            lx = f(label_w - 8.0),
            ty = f(y + row / 2.0 + 4.0),
            p = hbar_path(label_w, y + (row - thick) / 2.0, w, thick),
            c = slot(1),
            vx = f(label_w + w + 6.0),
        ));
    }
    s.push_str("</svg>");
    s
}

/// Vertical columns, one series, with y ticks and hairline grid. `every` labels every n-th x tick.
pub fn columns(bars: &[Bar], width: f64, height: f64, every: usize, fmt: fn(f64) -> String) -> String {
    let (left, bottom, top) = (48.0, 22.0, 10.0);
    let plot_w = width - left - 8.0;
    let plot_h = height - bottom - top;
    let max = bars.iter().map(|b| b.value).fold(0.0, f64::max);
    let t = ticks(max, 4);
    let ymax = *t.last().unwrap_or(&1.0);
    let base = top + plot_h;
    let band = plot_w / bars.len().max(1) as f64;
    let w = (band - 2.0).clamp(1.0, 24.0);
    let mut s = format!(r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#, f(width), f(height));
    for v in &t {
        let y = base - v / ymax * plot_h;
        s.push_str(&format!(r#"<line class="grid" x1="{}" x2="{}" y1="{y}" y2="{y}"/><text class="tick" x="{}" y="{}" text-anchor="end">{}</text>"#, f(left), f(width - 8.0), f(left - 6.0), f(y + 4.0), escape(&fmt(*v)), y = f(y)));
    }
    s.push_str(&format!(r#"<line class="axis" x1="{}" x2="{}" y1="{b}" y2="{b}"/>"#, f(left), f(width - 8.0), b = f(base)));
    for (i, b) in bars.iter().enumerate() {
        let x = left + i as f64 * band + (band - w) / 2.0;
        let h = b.value.max(0.0) / ymax * plot_h;
        s.push_str(&format!(
            r#"<g class="mark"><title>{l}: {v}</title><rect class="hit" x="{hx}" y="{top}" width="{bw}" height="{ph}"/><path d="{p}" fill="{c}"/></g>"#,
            l = escape(&b.label),
            v = escape(&b.shown),
            hx = f(left + i as f64 * band),
            top = f(top),
            bw = f(band),
            ph = f(plot_h),
            p = vbar_path(x, base, w, h),
            c = slot(1),
        ));
        if every > 0 && i % every == 0 {
            s.push_str(&format!(r#"<text class="tick" x="{}" y="{}" text-anchor="middle">{}</text>"#, f(x + w / 2.0), f(height - 6.0), escape(&b.label)));
        }
    }
    s.push_str("</svg>");
    s
}

/// A donut over up to three slices (slots 1-3), with a 2px surface gap between segments.
pub fn donut(slices: &[(String, f64)], size: f64, center: &str) -> String {
    let total: f64 = slices.iter().map(|s| s.1.max(0.0)).sum();
    let (cx, r, thick) = (size / 2.0, size / 2.0 - 8.0, 22.0);
    let rm = r - thick / 2.0;
    let mut s = format!(r#"<svg class="chart donut" viewBox="0 0 {sz} {sz}" role="img">"#, sz = f(size));
    let mut a0 = -std::f64::consts::FRAC_PI_2;
    for (i, (label, v)) in slices.iter().enumerate() {
        let frac = if total > 0.0 { v.max(0.0) / total } else { 0.0 };
        if frac <= 0.0 {
            continue;
        }
        let pct = format!("{:.0}%", frac * 100.0);
        let title = format!("<title>{}: {} ({pct})</title>", escape(label), f(*v));
        if frac >= 0.9999 {
            s.push_str(&format!(r#"<g class="mark">{title}<circle cx="{c}" cy="{c}" r="{}" fill="none" stroke="{}" stroke-width="{}"/></g>"#, f(rm), slot(i + 1), f(thick), c = f(cx)));
            break;
        }
        let a1 = a0 + frac * std::f64::consts::TAU;
        let (x0, y0) = (cx + rm * a0.cos(), cx + rm * a0.sin());
        let (x1, y1) = (cx + rm * a1.cos(), cx + rm * a1.sin());
        let large = if a1 - a0 > std::f64::consts::PI { 1 } else { 0 };
        s.push_str(&format!(
            r#"<g class="mark">{title}<path d="M{} {} A{r} {r} 0 {large} 1 {} {}" fill="none" stroke="{}" stroke-width="{}"/><line class="gap" x1="{}" y1="{}" x2="{}" y2="{}"/></g>"#,
            f(x0), f(y0), f(x1), f(y1), slot(i + 1), f(thick),
            f(cx + (r - thick - 1.0) * a1.cos()), f(cx + (r - thick - 1.0) * a1.sin()), f(cx + (r + 1.0) * a1.cos()), f(cx + (r + 1.0) * a1.sin()),
            r = f(rm),
        ));
        a0 = a1;
    }
    s.push_str(&format!(r#"<text class="center" x="{c}" y="{}" text-anchor="middle">{}</text></svg>"#, f(cx + 6.0), escape(center), c = f(cx)));
    s
}

/// One line series over x, with the highest point marked and labeled.
pub fn line(points: &[(f64, f64)], width: f64, height: f64, x_ticks: &[f64], label: &str) -> String {
    let (left, bottom, top, right) = (40.0, 22.0, 14.0, 12.0);
    let (pw, ph) = (width - left - right, height - top - bottom);
    let xmin = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let xmax = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let (xmin, xmax) = if xmin.is_finite() && xmax > xmin { (xmin, xmax) } else { (0.0, 1.0) };
    let t = ticks(points.iter().map(|p| p.1).fold(0.0, f64::max), 4);
    let ymax = *t.last().unwrap_or(&1.0);
    let sx = |x: f64| left + (x - xmin) / (xmax - xmin) * pw;
    let sy = |y: f64| top + ph - y / ymax * ph;
    let mut s = format!(r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#, f(width), f(height));
    for v in &t {
        s.push_str(&format!(r#"<line class="grid" x1="{}" x2="{}" y1="{y}" y2="{y}"/><text class="tick" x="{}" y="{}" text-anchor="end">{}</text>"#, f(left), f(width - right), f(left - 6.0), f(sy(*v) + 4.0), f(*v), y = f(sy(*v))));
    }
    for x in x_ticks {
        s.push_str(&format!(r#"<text class="tick" x="{}" y="{}" text-anchor="middle">{}</text>"#, f(sx(*x)), f(height - 6.0), f(*x)));
    }
    let d: Vec<String> = points.iter().enumerate().map(|(i, p)| format!("{}{} {}", if i == 0 { "M" } else { "L" }, f(sx(p.0)), f(sy(p.1)))).collect();
    s.push_str(&format!(r#"<path class="line" d="{}" stroke="{}"/>"#, d.join(""), slot(1)));
    let band = pw / points.len().max(1) as f64;
    for p in points {
        s.push_str(&format!(r#"<g class="mark"><title>{} {}: {}</title><rect class="hit" x="{}" y="{}" width="{}" height="{}"/></g>"#, escape(label), f(p.0), f(p.1), f(sx(p.0) - band / 2.0), f(top), f(band), f(ph)));
    }
    if let Some(top_pt) = points.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1)) {
        s.push_str(&format!(r#"<circle class="dot" cx="{}" cy="{}" r="4" fill="{}"/><text class="value" x="{}" y="{}">{} {}: {}</text>"#, f(sx(top_pt.0)), f(sy(top_pt.1)), slot(1), f(sx(top_pt.0) + 8.0), f(sy(top_pt.1) + 4.0), escape(label), f(top_pt.0), f(top_pt.1)));
    }
    s.push_str("</svg>");
    s
}

pub struct Path {
    pub label: String,
    pub points: Vec<(f64, f64)>,
    /// categorical slot, or None for the muted ink (a reference mark, not a series)
    pub slot: Option<usize>,
}

/// x/y paths on fixed axes, so shapes keep their proportions.
pub fn paths(series: &[Path], xr: (f64, f64), yr: (f64, f64), width: f64) -> String {
    let pad = 30.0;
    let pw = width - 2.0 * pad;
    let ph = pw * (yr.1 - yr.0) / (xr.1 - xr.0);
    let height = ph + 2.0 * pad;
    let sx = |x: f64| pad + (x - xr.0) / (xr.1 - xr.0) * pw;
    let sy = |y: f64| pad + ph - (y - yr.0) / (yr.1 - yr.0) * ph;
    let mut s = format!(r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#, f(width), f(height));
    for x in ticks(xr.1, 3).iter().flat_map(|v| [*v, -*v]) {
        s.push_str(&format!(r#"<line class="grid" x1="{x}" x2="{x}" y1="{}" y2="{}"/><text class="tick" x="{x}" y="{}" text-anchor="middle">{}</text>"#, f(pad), f(pad + ph), f(height - 10.0), f(x_val(x)), x = f(sx(x))));
    }
    for y in ticks(yr.1, 2).iter().flat_map(|v| [*v, -*v]) {
        s.push_str(&format!(r#"<line class="grid" x1="{}" x2="{}" y1="{y}" y2="{y}"/><text class="tick" x="{}" y="{}" text-anchor="end">{}</text>"#, f(pad), f(pad + pw), f(pad - 4.0), f(sy(y) + 4.0), f(x_val(y)), y = f(sy(y))));
    }
    for p in series {
        let color = p.slot.map_or("var(--muted)".to_string(), slot);
        if p.points.len() > 1 {
            let d: Vec<String> = p.points.iter().enumerate().map(|(i, q)| format!("{}{} {}", if i == 0 { "M" } else { "L" }, f(sx(q.0)), f(sy(q.1)))).collect();
            s.push_str(&format!(r#"<path class="line" d="{}" stroke="{color}"/>"#, d.join("")));
        }
        for (i, q) in p.points.iter().enumerate() {
            s.push_str(&format!(
                r#"<g class="mark"><title>{} sample {}: ({}, {})</title><circle class="hit" cx="{x}" cy="{y}" r="9"/><circle class="dot" cx="{x}" cy="{y}" r="4" fill="{color}"/></g>"#,
                escape(&p.label), i + 1, f(q.0), f(q.1), x = f(sx(q.0)), y = f(sy(q.1))
            ));
        }
    }
    s.push_str("</svg>");
    s
}

fn x_val(v: f64) -> f64 {
    if v == 0.0 { 0.0 } else { v } // no "-0"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bars(vals: &[f64]) -> Vec<Bar> {
        vals.iter().enumerate().map(|(i, v)| Bar { label: format!("b{i}<"), value: *v, shown: f(*v) }).collect()
    }

    #[test]
    fn ticks_are_clean() {
        assert_eq!(ticks(95.0, 4), vec![0.0, 50.0, 100.0]);
        assert_eq!(ticks(221.0, 4), vec![0.0, 100.0, 200.0, 300.0]);
        assert_eq!(ticks(0.0, 4), vec![0.0, 0.5, 1.0]);
        assert_eq!(ticks(f64::NAN, 4), vec![0.0, 0.5, 1.0]);
    }

    #[test]
    fn hbars_one_path_per_bar_and_escaped() {
        let svg = hbars(&bars(&[3.0, 0.0, 10.0]), 400.0, 120.0);
        assert_eq!(svg.matches("<path").count(), 3);
        assert_eq!(svg.matches("<title>").count(), 3);
        assert!(svg.contains("b0&lt;"));
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }

    #[test]
    fn columns_and_all_zero_input() {
        let svg = columns(&bars(&[0.0, 0.0]), 300.0, 160.0, 1, f);
        assert_eq!(svg.matches(r#"<g class="mark">"#).count(), 2);
        assert!(!svg.contains("NaN"));
    }

    #[test]
    fn donut_segments_and_gaps() {
        let svg = donut(&[("Prime".into(), 25.0), ("Composite".into(), 74.0), ("Neither".into(), 1.0)], 200.0, "100");
        assert_eq!(svg.matches("<path").count(), 3);
        assert_eq!(svg.matches(r#"class="gap""#).count(), 3);
        assert!(svg.contains("Composite: 74 (74%)"));
        let whole = donut(&[("All".into(), 5.0)], 200.0, "5");
        assert!(whole.contains("<circle"));
        assert!(!donut(&[], 200.0, "0").contains("NaN"));
    }

    #[test]
    fn line_and_paths_have_no_nan() {
        let l = line(&[(1.0, 5.0), (2.0, 9.0), (3.0, 4.0)], 400.0, 160.0, &[1.0, 3.0], "n");
        assert!(l.contains("n 2: 9"));
        assert!(!line(&[], 400.0, 160.0, &[], "n").contains("NaN"));
        let p = paths(&[Path { label: "Track 1".into(), points: vec![(0.0, 0.0), (10.0, 10.0)], slot: Some(1) }, Path { label: "Fixed".into(), points: vec![(60.0, 40.0)], slot: None }], (-300.0, 300.0), (-200.0, 200.0), 600.0);
        assert_eq!(p.matches(r#"class="dot""#).count(), 3);
        assert_eq!(p.matches(r#"class="line""#).count(), 1);
        assert!(!p.contains("NaN") && !p.contains(">-0<"));
    }
}
