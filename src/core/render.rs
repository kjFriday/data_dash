// ==== ACME Inc. portfolio ===============================================
// Project: data_dash | File: src/core/render.rs
// About: HTML for the two views.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! HTML for the two views. Every value from data passes `escape`.

use super::metrics::{self, compact, group, money, Data};
use super::numbers::{self, NumberRow};
use super::svg::{self, Bar, Path};
use super::tracks::{self, Sample};

pub const PAGE_ROWS: usize = 15;

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

pub fn tile(label: &str, value: &str, note: &str) -> String {
    format!(r#"<div class="tile"><div class="tile-label">{}</div><div class="tile-value">{}</div><div class="tile-note">{}</div></div>"#, escape(label), escape(value), escape(note))
}

fn card(title: &str, sub: &str, body: &str) -> String {
    format!(r#"<section class="card"><h2>{}</h2><p class="sub">{}</p>{body}</section>"#, escape(title), escape(sub))
}

/// Legend with a swatch beside each label (identity never rests on color alone).
pub fn legend(items: &[(String, Option<usize>)]) -> String {
    let mut s = String::from(r#"<ul class="legend">"#);
    for (label, slot) in items {
        let color = slot.map_or("var(--muted)".to_string(), svg::slot);
        s.push_str(&format!(r#"<li><i style="background:{color}"></i>{}</li>"#, escape(label)));
    }
    s.push_str("</ul>");
    s
}

fn status_label(s: &str) -> String {
    match s {
        "open" => "Open".into(),
        "in_progress" => "In progress".into(),
        "closed" => "Closed".into(),
        other => other.to_string(),
    }
}

pub fn business(d: &Data, source: &str) -> String {
    let k = metrics::kpis(d);
    let mut out = String::from(r#"<div class="hero">"#);
    out.push_str(&format!(r#"<div class="hero-label">Revenue, {} orders</div><div class="hero-value">{}</div><div class="tile-note">{}</div></div>"#, group(k.orders as u64), money(k.revenue_cents), escape(source)));
    out.push_str(r#"<div class="tiles">"#);
    out.push_str(&tile("Orders", &group(k.orders as u64), "in this data"));
    out.push_str(&tile("Average order", &money(k.avg_order_cents), "revenue / orders"));
    out.push_str(&tile("Buying customers", &group(k.customers as u64), "placed at least one order"));
    out.push_str(&tile("Open tickets", &group(k.open_tickets as u64), "open or in progress"));
    out.push_str("</div>");

    let months = metrics::revenue_by_month(d);
    let bars: Vec<Bar> = months.iter().map(|(m, c)| Bar { label: m.clone(), value: *c as f64 / 100.0, shown: money(*c) }).collect();
    let every = (bars.len() / 6).max(1);
    let mut grid = card("Revenue by month", "Order totals per calendar month", &svg::columns(&bars, 560.0, 220.0, every, |v| format!("${}", compact(v))));

    let cats: Vec<Bar> = metrics::revenue_by_category(d).into_iter().map(|(c, v)| Bar { label: c, value: v as f64, shown: money(v) }).collect();
    grid.push_str(&card("Revenue by product category", "All orders in this data", &svg::hbars(&cats, 560.0, 120.0)));

    let prods: Vec<Bar> = metrics::top_products(d, 10).into_iter().map(|(c, v)| Bar { label: c, value: v as f64, shown: money(v) }).collect();
    grid.push_str(&card("Top 10 products", "By revenue", &svg::hbars(&prods, 560.0, 150.0)));

    let st = metrics::tickets_by_status(d);
    let slices: Vec<(String, f64)> = st.iter().take(3).map(|(s, n)| (status_label(s), *n as f64)).collect();
    let total: usize = st.iter().map(|x| x.1).sum();
    let leg: Vec<(String, Option<usize>)> = st.iter().take(3).enumerate().map(|(i, (s, n))| (format!("{} {}", status_label(s), group(*n as u64)), Some(i + 1))).collect();
    grid.push_str(&card("Tickets by status", "Every ticket in this data", &format!(r#"<div class="donut-row">{}{}</div>"#, svg::donut(&slices, 200.0, &format!("{} tickets", group(total as u64))), legend(&leg))));

    let mut table = String::from(r#"<table><thead><tr><th>Customer</th><th>City</th><th class="num">Orders</th><th class="num">Revenue</th></tr></thead><tbody>"#);
    for (name, city, rev, n) in metrics::top_customers(d, 10) {
        table.push_str(&format!(r#"<tr><td>{}</td><td>{}</td><td class="num">{n}</td><td class="num">{}</td></tr>"#, escape(&name), escape(&city), money(rev)));
    }
    table.push_str("</tbody></table>");
    grid.push_str(&card("Top 10 customers", "By revenue", &table));

    out.push_str(&format!(r#"<div class="grid">{grid}</div>"#));
    out
}

/// Rows whose classes contain `class` (case-insensitive substring; empty = all).
pub fn filter_rows<'a>(rows: &'a [NumberRow], class: &str) -> Vec<&'a NumberRow> {
    let q = class.trim().to_lowercase();
    rows.iter().filter(|r| q.is_empty() || r.classes.iter().any(|c| c.to_lowercase().contains(&q))).collect()
}

pub fn number_table(rows: &[NumberRow], class: &str, page: usize) -> (String, usize) {
    let hits = filter_rows(rows, class);
    let pages = hits.len().div_ceil(PAGE_ROWS).max(1);
    let page = page.min(pages - 1);
    let mut s = String::from(r#"<table><thead><tr><th class="num">n</th><th>Item</th><th class="num">Sitelinks</th><th>Classes tagged on Wikidata</th></tr></thead><tbody>"#);
    for r in hits.iter().skip(page * PAGE_ROWS).take(PAGE_ROWS) {
        let shown: Vec<String> = r.classes.iter().take(6).map(|c| escape(c)).collect();
        let more = if r.classes.len() > 6 { format!(" +{}", r.classes.len() - 6) } else { String::new() };
        s.push_str(&format!(r#"<tr><td class="num">{}</td><td>{}</td><td class="num">{}</td><td class="classes">{}{more}</td></tr>"#, r.n, escape(&r.item), r.sitelinks, shown.join(", ")));
    }
    s.push_str(&format!(
        r#"</tbody></table><div class="pager"><button data-page="{}" {}>Previous</button><span>{} rows, page {} of {pages}</span><button data-page="{}" {}>Next</button></div>"#,
        page.saturating_sub(1),
        if page == 0 { "disabled" } else { "" },
        hits.len(),
        page + 1,
        page + 1,
        if page + 1 >= pages { "disabled" } else { "" },
    ));
    (s, page)
}

pub fn fixtures(rows: &[NumberRow], samples: &[Sample]) -> String {
    let counts = numbers::class_counts(rows);
    let (p, c, n) = numbers::prime_split(rows);
    let med = numbers::median_sitelinks_x2(rows).unwrap_or(0);
    let med_s = if med.is_multiple_of(2) { (med / 2).to_string() } else { format!("{}.5", med / 2) };
    let top = numbers::most_linked(rows);

    let mut out = String::from(r#"<div class="tiles">"#);
    out.push_str(&tile("Rows", &rows.len().to_string(), "numbers 1 to 100"));
    out.push_str(&tile("Distinct classes", &counts.len().to_string(), "tagged on Wikidata"));
    out.push_str(&tile("Median sitelinks", &med_s, "Wikimedia sites per item"));
    out.push_str(&tile("Most linked", &top.map_or("-".into(), |t| format!("{} ({})", t.n, t.item)), &top.map_or(String::new(), |t| format!("{} sitelinks", t.sitelinks))));
    out.push_str("</div>");

    let bars: Vec<Bar> = counts.iter().take(15).map(|(k, v)| Bar { label: k.clone(), value: *v as f64, shown: v.to_string() }).collect();
    let mut grid = card("Top 15 classes", "Rows per class, tagged on Wikidata", &svg::hbars(&bars, 560.0, 190.0));
    let slices = [("Prime".to_string(), p as f64), ("Composite".to_string(), c as f64), ("Neither".to_string(), n as f64)];
    let leg = vec![(format!("Prime {p}"), Some(1)), (format!("Composite {c}"), Some(2)), (format!("Neither {n}"), Some(3))];
    grid.push_str(&card("Prime, composite or neither", "Tagged on Wikidata", &format!(r#"<div class="donut-row">{}{}</div>"#, svg::donut(&slices, 200.0, "100 numbers"), legend(&leg))));
    let pts: Vec<(f64, f64)> = rows.iter().map(|r| (r.n as f64, r.sitelinks as f64)).collect();
    grid.push_str(&card("Sitelinks by number", "How many Wikimedia sites link each item", &svg::line(&pts, 560.0, 200.0, &[1.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0], "n")));

    let by = tracks::by_track(samples);
    let mut moving = 0;
    let mut series = Vec::new();
    let mut leg = Vec::new();
    for (id, pts) in &by {
        let still = tracks::is_stationary(pts);
        let slot = if still || moving >= 3 { None } else { moving += 1; Some(moving) };
        let label = if still { format!("Track {id} (fixed point)") } else { format!("Track {id}") };
        leg.push((label.clone(), slot));
        series.push(Path { label, points: pts.iter().map(|s| (s.x, s.y)).collect(), slot });
    }
    grid.push_str(&card("Track paths", "x against y on fixed axes, in time order", &format!("{}{}", legend(&leg), svg::paths(&series, (-300.0, 300.0), (-200.0, 200.0), 560.0))));
    let counts: Vec<Bar> = by.iter().map(|(id, v)| Bar { label: format!("Track {id}"), value: v.len() as f64, shown: v.len().to_string() }).collect();
    grid.push_str(&card("Samples per track", "Rows in tracks.dat", &svg::hbars(&counts, 560.0, 90.0)));
    out.push_str(&format!(r#"<div class="grid">{grid}</div>"#));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<NumberRow> {
        numbers::parse(include_str!("../../data/numbers.dat")).unwrap()
    }

    #[test]
    fn filter_and_paging() {
        let r = rows();
        assert_eq!(filter_rows(&r, "").len(), 100);
        assert_eq!(filter_rows(&r, " PRIME NUMBER").len(), filter_rows(&r, "prime number").len());
        let (html, page) = number_table(&r, "", 99);
        assert_eq!(page, 6); // 100 rows / 15 = 7 pages, clamped to the last
        assert!(html.contains("page 7 of 7"));
        assert_eq!(html.matches("<tr>").count(), 1 + 10);
        let (none, _) = number_table(&r, "no such class", 0);
        assert!(none.contains("0 rows, page 1 of 1"));
    }

    #[test]
    fn views_render() {
        let samples = tracks::parse(include_str!("../../data/tracks.dat")).unwrap();
        let html = fixtures(&rows(), &samples);
        assert!(html.contains("Track 7 (fixed point)"));
        assert!(html.contains(">144<"));
        assert!(!html.contains("NaN"));
        let d = Data::from_json(
            include_str!("../../mock/orders.json"),
            include_str!("../../mock/products.json"),
            include_str!("../../mock/customers.json"),
            include_str!("../../mock/tickets.json"),
        )
        .unwrap();
        let b = business(&d, "mock <snapshot>");
        assert!(b.contains("mock &lt;snapshot&gt;"));
        assert_eq!(b.matches(r#"<section class="card">"#).count(), 5);
        assert!(!b.contains("NaN"));
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=b876c878c3a8a3ecd394517c1900c5a947c5f6fa834c13a09826a431161ca060 ====
