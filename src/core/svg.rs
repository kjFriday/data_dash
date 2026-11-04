// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/svg.rs
// About: Hand-built SVG charts.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Hand-built SVG charts. Colors are CSS custom properties (see www/style.css), so light and dark
//! steps swap in one place. Marks follow one spec: bars <= 24px with a 4px rounded data end and a
//! square baseline, 2px lines, >= 8px markers with a 2px surface ring, hairline grid, and a
//! `<title>` on every mark for the hover tooltip. Text never wears a series color.
//!
//! Every chart is one `<svg class="chart" viewBox="0 0 W H">` string; `render.rs` places it in a card
//! and style.css stretches it to the card's width, so all sizes here are viewBox units, not pixels.
//! SVG's y axis points down: y = 0 is the top edge, so a value is drawn at `base - height`.
//!
//! The path strings use SVG path commands: `M x y` moves the pen, `L x y` draws a line, `H x` and
//! `V y` draw horizontal and vertical lines, `Q cx cy x y` draws a quadratic curve bent toward the
//! control point (cx, cy) (used for the rounded bar ends), `A` draws an elliptical arc (the donut),
//! and `Z` closes the shape.
//!
//! Each mark is a `<g class="mark">` holding a `<title>` (the browser shows it as a tooltip) and,
//! where the mark is thin, a transparent `.hit` shape that widens the hover target; style.css shades
//! the `.hit` of the hovered mark.
//!
//! Charts: `hbars` (horizontal bars), `columns` (vertical bars with a y axis), `donut`, `line` and
//! `paths` (x/y tracks). Inputs: `Bar` and `XyPath`.

use super::html::escape;

/// One categorical slot (1-based) as a CSS variable.
///
/// style.css defines `--series-1` to `--series-3` for both the light and dark steps, so a chart never
/// names a color itself.
pub fn slot(n: usize) -> String {
    format!("var(--series-{n})")
}

/// Clean axis ticks from 0 to at least `max`: steps of 1, 2 or 5 × 10^k, about `want` of them.
///
/// The usual "nice numbers" method for axis labels:
/// 1. The raw step is `max / want` (95 and 4 -> 23.75).
/// 2. Its power of ten `mag` is 10^floor(log10(raw)) (-> 10).
/// 3. The step is the first of 1, 2, 5 and 10 × `mag` that is at least the raw step (-> 50).
/// 4. Ticks run from 0 in that step up to the first multiple at or above `max` (-> 0, 50, 100).
///
/// The count can come out below `want` (3 here): a round step matters more. A `max` that is 0,
/// negative or not finite is treated as 1, so the result is never empty.
pub fn ticks(max: f64, want: usize) -> Vec<f64> {
    let max = if max.is_finite() && max > 0.0 { max } else { 1.0 };
    let raw = max / want.max(1) as f64;
    let mag = 10f64.powf(raw.log10().floor());
    let step = [1.0, 2.0, 5.0, 10.0]
        .iter()
        .map(|m| m * mag)
        .find(|s| *s >= raw)
        .unwrap_or(10.0 * mag);
    let top = (max / step).ceil() as usize;
    (0..=top).map(|i| i as f64 * step).collect()
}

/// A number as SVG text: one decimal, a trailing ".0" dropped; NaN and infinities print as 0.
///
/// One decimal is finer than a pixel at these sizes and keeps the markup short; mapping non-finite
/// values to 0 means an empty or all-zero input can never put "NaN" into an attribute.
fn f(v: f64) -> String {
    let v = if v.is_finite() { v } else { 0.0 };
    let s = format!("{v:.1}");
    s.strip_suffix(".0").map(String::from).unwrap_or(s)
}

/// `v` with -0.0 made 0.0, so an axis label never reads "-0" (`f` keeps the sign).
fn no_negative_zero(v: f64) -> f64 {
    // -0.0 == 0.0 is true in IEEE 754, so this branch catches both zeros and returns the positive one
    if v == 0.0 {
        0.0
    } else {
        v
    }
}

/// A bar from the baseline with only its data end rounded (r = 4, or less for a short bar).
///
/// The bar's box is (`x`, `y`) to (`x + w`, `y + h`), with the baseline on the left. The pen goes
/// along the top edge, curves round the top-right corner, down the right edge, round the
/// bottom-right corner, and back along the bottom; `Z` closes the square left end. The radius
/// shrinks to half the width or height so a tiny bar never folds over itself.
fn hbar_path(x: f64, y: f64, w: f64, h: f64) -> String {
    let r = 4f64.min(w / 2.0).min(h / 2.0).max(0.0);
    format!(
        "M{} {}H{}Q{} {} {} {}V{}Q{} {} {} {}H{}Z",
        f(x),
        f(y),
        f(x + w - r),
        f(x + w),
        f(y),
        f(x + w),
        f(y + r),
        f(y + h - r),
        f(x + w),
        f(y + h),
        f(x + w - r),
        f(y + h),
        f(x)
    )
}

/// A column standing on `base` with only its top corners rounded: the vertical twin of `hbar_path`.
///
/// `x` is the left edge, `w` the width and `h` the height up from `base` (SVG y grows downward, so the
/// top is `base - h`). The pen goes up the left edge, round both top corners, and down the right edge.
fn vbar_path(x: f64, base: f64, w: f64, h: f64) -> String {
    let r = 4f64.min(w / 2.0).min(h / 2.0).max(0.0);
    let top = base - h;
    format!(
        "M{} {}V{}Q{} {} {} {}H{}Q{} {} {} {}V{}Z",
        f(x),
        f(base),
        f(top + r),
        f(x),
        f(top),
        f(x + r),
        f(top),
        f(x + w - r),
        f(x + w),
        f(top),
        f(x + w),
        f(top + r),
        f(base)
    )
}

/// One bar or column of `hbars` and `columns`.
pub struct Bar {
    /// Category name: beside the bar (or under a column) and in the tooltip; escaped here.
    pub label: String,
    /// Length driver; only its ratio to the largest value matters, plus the axis ticks in `columns`.
    /// Negative values draw as 0.
    pub value: f64,
    pub shown: String, // the value as printed at the tip and in the tooltip
}

/// Horizontal bars, one series: label on the left, value at the tip.
///
/// `width` is the viewBox width; `label_w` is the label column left of the baseline. The height
/// follows from the bar count: one 26-unit row per bar plus a 4-unit margin above and below. Bar
/// lengths are relative to the largest value; there is no axis, since each bar prints its value.
pub fn hbars(bars: &[Bar], width: f64, label_w: f64) -> String {
    let row = 26.0;
    // 16 units thick: under the spec's 24px cap, with 10 units of air between rows
    let thick = 16.0;
    // room right of the longest bar for its value text
    let value_w = 64.0;
    let plot = (width - label_w - value_w).max(10.0);
    let max = bars.iter().map(|b| b.value).fold(0.0, f64::max);
    // all zero (or empty): divide by 1 so every bar is 0 long instead of NaN
    let max = if max > 0.0 { max } else { 1.0 };
    let h = row * bars.len() as f64 + 8.0;
    let mut s = format!(r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#, f(width), f(h));
    s.push_str(&format!(
        r#"<line class="axis" x1="{x}" x2="{x}" y1="0" y2="{}"/>"#,
        f(h - 4.0),
        x = f(label_w)
    ));
    // each mark: a full-width transparent hit row, the label right-aligned 8 units left of the baseline,
    // the bar centred in its row, and the value 6 units past the tip; SVG text sits on its baseline, so
    // text y is the row's middle plus 4 to centre the 11px text roughly
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
///
/// `fmt` prints a y tick value ("$12.9K"); `every` = 0 prints no x labels. Layout:
/// 1. Margins: 48 units left for tick text, 22 below for x labels, 10 above, 8 right.
/// 2. The y axis runs from 0 to the top tick of `ticks(max, 4)`, so the tallest column ends at or
///    under the top grid line.
/// 3. Each column gets an equal band of the plot width; the column is the band less 2 units, held
///    to 1-24 units (the spec's 24px cap), and centred in it.
/// 4. The whole band, top to baseline, is the column's hover target.
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
    let mut s = format!(
        r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#,
        f(width),
        f(height)
    );
    for v in &t {
        let y = base - v / ymax * plot_h;
        s.push_str(&format!(r#"<line class="grid" x1="{}" x2="{}" y1="{y}" y2="{y}"/><text class="tick" x="{}" y="{}" text-anchor="end">{}</text>"#, f(left), f(width - 8.0), f(left - 6.0), f(y + 4.0), escape(&fmt(*v)), y = f(y)));
    }
    s.push_str(&format!(
        r#"<line class="axis" x1="{}" x2="{}" y1="{b}" y2="{b}"/>"#,
        f(left),
        f(width - 8.0),
        b = f(base)
    ));
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
            s.push_str(&format!(
                r#"<text class="tick" x="{}" y="{}" text-anchor="middle">{}</text>"#,
                f(x + w / 2.0),
                f(height - 6.0),
                escape(&b.label)
            ));
        }
    }
    s.push_str("</svg>");
    s
}

/// A donut over up to three slices (slots 1-3), with a 2px surface gap between segments.
///
/// `slices` are (label, value); slice i takes slot i + 1, so more than three would name a
/// `--series-N` that style.css does not define. `size` is the square viewBox side; `center` is the
/// text in the hole.
///
/// How the ring is drawn:
/// 1. The ring's outer radius is `size / 2 - 8` and it is 22 units thick. Each segment is a stroked
///    arc along the ring's middle radius `rm`, so the stroke width fills the ring (a stroke is centred
///    on its path).
/// 2. Angles start at -π/2, which is 12 o'clock; because SVG's y axis points down, a growing angle
///    turns clockwise. Each slice covers `value / total` of a full turn (τ = 2π).
/// 3. Each segment is `M start A rm rm 0 large 1 end`: the arc's radii, no rotation, the large-arc
///    flag (1 when the slice is over half the circle, so the longer way round is taken) and the sweep
///    flag (1 = clockwise).
/// 4. A short radial `.gap` line in the surface color, 1 unit past each ring edge, is drawn at the end
///    of every segment, which cuts the 2px gap.
/// 5. A slice of (almost) the whole total is a plain circle: an arc whose start and end coincide draws
///    nothing.
///
/// Empty or zero slices draw nothing; with no total the ring is empty but the centre text stays.
/// The percentages in the tooltips are rounded separately and may not add up to 100.
pub fn donut(slices: &[(String, f64)], size: f64, center: &str) -> String {
    let total: f64 = slices.iter().map(|s| s.1.max(0.0)).sum();
    let (cx, r, thick) = (size / 2.0, size / 2.0 - 8.0, 22.0);
    let rm = r - thick / 2.0;
    let mut s = format!(
        r#"<svg class="chart donut" viewBox="0 0 {sz} {sz}" role="img">"#,
        sz = f(size)
    );
    let mut a0 = -std::f64::consts::FRAC_PI_2;
    for (i, (label, v)) in slices.iter().enumerate() {
        let frac = if total > 0.0 { v.max(0.0) / total } else { 0.0 };
        if frac <= 0.0 {
            continue;
        }
        let pct = format!("{:.0}%", frac * 100.0);
        let title = format!("<title>{}: {} ({pct})</title>", escape(label), f(*v));
        // a whole ring: an arc from a point back to itself would draw nothing
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
    // + 6: the text's baseline sits a little under the centre so the 13px label looks centred
    s.push_str(&format!(
        r#"<text class="center" x="{c}" y="{}" text-anchor="middle">{}</text></svg>"#,
        f(cx + 6.0),
        escape(center),
        c = f(cx)
    ));
    s
}

/// One line series over x, with the highest point marked and labeled.
///
/// `points` are (x, y) in drawing order; `x_ticks` are the x values to label; `label` names x in the
/// tooltips and the peak label ("n 1: 221"). Layout:
/// 1. x maps linearly from the data's min..max onto the plot width (0..1 when there is no spread).
/// 2. y maps from 0 to the top tick of `ticks(max y, 4)`, with grid lines at each tick.
/// 3. One 2-unit path joins the points (`M` to the first, `L` to each next).
/// 4. Each point gets a hover band of equal width centred on it, the full plot height, with a
///    tooltip.
/// 5. The highest point (the last of equal highs) gets an 8-unit dot and a text label to its right.
pub fn line(points: &[(f64, f64)], width: f64, height: f64, x_ticks: &[f64], label: &str) -> String {
    let (left, bottom, top, right) = (40.0, 22.0, 14.0, 12.0);
    let (pw, ph) = (width - left - right, height - top - bottom);
    let xmin = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let xmax = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let (xmin, xmax) = if xmin.is_finite() && xmax > xmin {
        (xmin, xmax)
    } else {
        (0.0, 1.0)
    };
    let t = ticks(points.iter().map(|p| p.1).fold(0.0, f64::max), 4);
    let ymax = *t.last().unwrap_or(&1.0);
    let sx = |x: f64| left + (x - xmin) / (xmax - xmin) * pw;
    let sy = |y: f64| top + ph - y / ymax * ph;
    let mut s = format!(
        r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#,
        f(width),
        f(height)
    );
    for v in &t {
        s.push_str(&format!(r#"<line class="grid" x1="{}" x2="{}" y1="{y}" y2="{y}"/><text class="tick" x="{}" y="{}" text-anchor="end">{}</text>"#, f(left), f(width - right), f(left - 6.0), f(sy(*v) + 4.0), f(*v), y = f(sy(*v))));
    }
    for x in x_ticks {
        s.push_str(&format!(
            r#"<text class="tick" x="{}" y="{}" text-anchor="middle">{}</text>"#,
            f(sx(*x)),
            f(height - 6.0),
            f(*x)
        ));
    }
    let d: Vec<String> = points
        .iter()
        .enumerate()
        .map(|(i, p)| format!("{}{} {}", if i == 0 { "M" } else { "L" }, f(sx(p.0)), f(sy(p.1))))
        .collect();
    s.push_str(&format!(
        r#"<path class="line" d="{}" stroke="{}"/>"#,
        d.join(""),
        slot(1)
    ));
    let band = pw / points.len().max(1) as f64;
    for p in points {
        s.push_str(&format!(
            r#"<g class="mark"><title>{} {}: {}</title><rect class="hit" x="{}" y="{}" width="{}" height="{}"/></g>"#,
            escape(label),
            f(p.0),
            f(p.1),
            f(sx(p.0) - band / 2.0),
            f(top),
            f(band),
            f(ph)
        ));
    }
    if let Some(top_pt) = points.iter().copied().max_by(|a, b| a.1.total_cmp(&b.1)) {
        s.push_str(&format!(r#"<circle class="dot" cx="{}" cy="{}" r="4" fill="{}"/><text class="value" x="{}" y="{}">{} {}: {}</text>"#, f(sx(top_pt.0)), f(sy(top_pt.1)), slot(1), f(sx(top_pt.0) + 8.0), f(sy(top_pt.1) + 4.0), escape(label), f(top_pt.0), f(top_pt.1)));
    }
    s.push_str("</svg>");
    s
}

/// One labelled path of x/y points for `paths`.
pub struct XyPath {
    pub label: String,
    pub points: Vec<(f64, f64)>,
    /// categorical slot, or None for the muted ink (a reference mark, not a series)
    pub slot: Option<usize>,
}

/// x/y paths on fixed axes, so shapes keep their proportions.
///
/// `xr` and `yr` are the (min, max) data ranges shown; `width` is the viewBox width. The plot height
/// is derived from it so one data unit is the same length on both axes; the chart's height follows.
///
/// Steps:
/// 1. Grid and tick labels: `ticks` on the positive half of each axis (3 for x, 2 for y), mirrored
///    to the negative half.
/// 2. Per series: a line through its points when there are two or more, then a dot per sample,
///    each with a 9-unit hover circle and a tooltip "LABEL sample K: (x, y)".
///
/// Known limitation: the ticks assume a range symmetric about 0 (only `xr.1` and `yr.1` are used),
/// and 0 is drawn twice, once as +0 and once as -0.
pub fn paths(series: &[XyPath], xr: (f64, f64), yr: (f64, f64), width: f64) -> String {
    let pad = 30.0;
    let pw = width - 2.0 * pad;
    // equal scale on both axes: plot height = plot width × (y span / x span)
    let ph = pw * (yr.1 - yr.0) / (xr.1 - xr.0);
    let height = ph + 2.0 * pad;
    let sx = |x: f64| pad + (x - xr.0) / (xr.1 - xr.0) * pw;
    let sy = |y: f64| pad + ph - (y - yr.0) / (yr.1 - yr.0) * ph;
    let mut s = format!(
        r#"<svg class="chart" viewBox="0 0 {} {}" role="img">"#,
        f(width),
        f(height)
    );
    // each tick and its mirror; the mirror of 0 is -0, which `no_negative_zero` prints as "0"
    for x in ticks(xr.1, 3).iter().flat_map(|v| [*v, -*v]) {
        s.push_str(&format!(r#"<line class="grid" x1="{x}" x2="{x}" y1="{}" y2="{}"/><text class="tick" x="{x}" y="{}" text-anchor="middle">{}</text>"#, f(pad), f(pad + ph), f(height - 10.0), f(no_negative_zero(x)), x = f(sx(x))));
    }
    for y in ticks(yr.1, 2).iter().flat_map(|v| [*v, -*v]) {
        s.push_str(&format!(r#"<line class="grid" x1="{}" x2="{}" y1="{y}" y2="{y}"/><text class="tick" x="{}" y="{}" text-anchor="end">{}</text>"#, f(pad), f(pad + pw), f(pad - 4.0), f(sy(y) + 4.0), f(no_negative_zero(y)), y = f(sy(y))));
    }
    for p in series {
        let color = p.slot.map_or("var(--muted)".to_string(), slot);
        if p.points.len() > 1 {
            let d: Vec<String> = p
                .points
                .iter()
                .enumerate()
                .map(|(i, q)| format!("{}{} {}", if i == 0 { "M" } else { "L" }, f(sx(q.0)), f(sy(q.1))))
                .collect();
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

/// Unit tests: tick steps, and each chart's mark count, escaping and freedom from "NaN".
#[cfg(test)]
mod tests {
    use super::*;

    /// One bar per value, labelled "b0<", "b1<" ... so escaping shows up in the output.
    fn bars(vals: &[f64]) -> Vec<Bar> {
        vals.iter()
            .enumerate()
            .map(|(i, v)| Bar {
                label: format!("b{i}<"),
                value: *v,
                shown: f(*v),
            })
            .collect()
    }

    /// 95 and 221 (the top sitelink count) get 1-2-5 steps; zero and NaN fall back to a 0..1 axis.
    #[test]
    fn ticks_are_clean() {
        assert_eq!(ticks(95.0, 4), vec![0.0, 50.0, 100.0]);
        assert_eq!(ticks(221.0, 4), vec![0.0, 100.0, 200.0, 300.0]);
        assert_eq!(ticks(0.0, 4), vec![0.0, 0.5, 1.0]);
        assert_eq!(ticks(f64::NAN, 4), vec![0.0, 0.5, 1.0]);
    }

    /// A zero bar still gets its path and tooltip, labels are escaped, and nothing prints NaN or inf.
    #[test]
    fn hbars_one_path_per_bar_and_escaped() {
        let svg = hbars(&bars(&[3.0, 0.0, 10.0]), 400.0, 120.0);
        assert_eq!(svg.matches("<path").count(), 3);
        assert_eq!(svg.matches("<title>").count(), 3);
        assert!(svg.contains("b0&lt;"));
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }

    /// All-zero columns still draw one mark each and no NaN (the axis falls back to 0..1).
    #[test]
    fn columns_and_all_zero_input() {
        let svg = columns(&bars(&[0.0, 0.0]), 300.0, 160.0, 1, f);
        assert_eq!(svg.matches(r#"<g class="mark">"#).count(), 2);
        assert!(!svg.contains("NaN"));
    }

    /// The fixture's 25/74/1 split gives three arcs and three gaps; one slice is a full circle; no
    /// slices give no NaN.
    #[test]
    fn donut_segments_and_gaps() {
        let svg = donut(
            &[
                ("Prime".into(), 25.0),
                ("Composite".into(), 74.0),
                ("Neither".into(), 1.0),
            ],
            200.0,
            "100",
        );
        assert_eq!(svg.matches("<path").count(), 3);
        assert_eq!(svg.matches(r#"class="gap""#).count(), 3);
        assert!(svg.contains("Composite: 74 (74%)"));
        let whole = donut(&[("All".into(), 5.0)], 200.0, "5");
        assert!(whole.contains("<circle"));
        assert!(!donut(&[], 200.0, "0").contains("NaN"));
    }

    /// The line labels its peak, an empty line has no NaN, and paths draw a line only for a series of
    /// two or more points, a dot per point, and no "-0" tick.
    #[test]
    fn line_and_paths_have_no_nan() {
        let l = line(&[(1.0, 5.0), (2.0, 9.0), (3.0, 4.0)], 400.0, 160.0, &[1.0, 3.0], "n");
        assert!(l.contains("n 2: 9"));
        assert!(!line(&[], 400.0, 160.0, &[], "n").contains("NaN"));
        let p = paths(
            &[
                XyPath {
                    label: "Track 1".into(),
                    points: vec![(0.0, 0.0), (10.0, 10.0)],
                    slot: Some(1),
                },
                XyPath {
                    label: "Fixed".into(),
                    points: vec![(60.0, 40.0)],
                    slot: None,
                },
            ],
            (-300.0, 300.0),
            (-200.0, 200.0),
            600.0,
        );
        assert_eq!(p.matches(r#"class="dot""#).count(), 3);
        assert_eq!(p.matches(r#"class="line""#).count(), 1);
        assert!(!p.contains("NaN") && !p.contains(">-0<"));
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=ec2e2983e4053a56c4e8d6a65c2c8f8af6c50800c6cefc2edc0e07dd20f2b161 ====
