// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/render.rs
// About: HTML for the two views.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! HTML for the two views. Every value from data passes `escape` (core/html.rs).
//!
//! `web.rs` calls three entry points and sets each result as an element's `innerHTML`:
//! - `business(&Data, source)` -> the Business tab: a hero figure, four tiles and five cards (revenue
//!   by month, by category, top products, tickets by status, top customers).
//! - `fixtures(rows, samples)` -> the charts of the Numbers & tracks tab: four tiles and five cards.
//! - `number_table(rows, filter, page)` -> that tab's filterable, paged table, plus the page actually
//!   shown.
//!
//! The markup uses the classes www/style.css defines (`.hero`, `.tiles`, `.tile`, `.grid`, `.card`,
//! `.sub`, `.donut-row`, `.legend`, `.num`, `.classes`, `.pager`); renaming one here needs the same
//! rename there. The charts come from `svg.rs`. Start reading at `business`.

use super::html::escape;
use super::metrics::{self, compact, group, money, Data};
use super::numbers::{self, NumberRow};
use super::svg::{self, Bar, XyPath};
use super::tracks::{self, Sample};

/// Rows per page of the number table (100 rows -> 7 pages).
pub const PAGE_ROWS: usize = 15;

// Chart sizes in SVG viewBox units: style.css stretches `svg.chart` to its card's width and shows `svg.donut`
// 200px wide. The label widths are the column left of each bar chart's bars.
/// Width of every card chart; heights of bar charts follow from their row count.
const CHART_W: f64 = 560.0;
/// Height of the revenue-by-month columns.
const COLUMNS_H: f64 = 220.0;
/// Height of the sitelinks line chart.
const LINE_H: f64 = 200.0;
/// Width and height of a donut (matches the 200px `svg.donut` in style.css, so 1 unit = 1px).
const DONUT_SIZE: f64 = 200.0;
/// Label column for product categories.
const CATEGORY_LABEL_W: f64 = 120.0;
/// Label column for product names (the longest labels on the Business tab).
const PRODUCT_LABEL_W: f64 = 150.0;
/// Label column for Wikidata class names ("positive integer", "composite number" ...).
const CLASS_LABEL_W: f64 = 190.0;
/// Label column for "Track N".
const TRACK_LABEL_W: f64 = 90.0;
/// Month labels under the revenue columns: every n-th month is labelled, so about this many show.
const MONTH_LABELS: usize = 6;
/// Series colors in style.css (`--series-1` to `--series-3`). The status donut and its legend show this many
/// statuses (the board's three); the first this many moving tracks get a color, later ones and fixed points
/// take the muted ink.
const SLOTS: usize = 3;
/// Classes listed in a number-table row; the rest are counted as " +N".
const CLASSES_SHOWN: usize = 6;

/// One KPI tile: a label, a big value and a small note under it (all three escaped).
pub fn tile(label: &str, value: &str, note: &str) -> String {
    format!(
        r#"<div class="tile"><div class="tile-label">{}</div><div class="tile-value">{}</div><div class="tile-note">{}</div></div>"#,
        escape(label),
        escape(value),
        escape(note)
    )
}

/// One card: an `<h2>` title, a subtitle line and `body`.
///
/// The title and subtitle are escaped; `body` is markup the caller built (a chart or a table) and goes
/// in as is.
fn card(title: &str, sub: &str, body: &str) -> String {
    format!(
        r#"<section class="card"><h2>{}</h2><p class="sub">{}</p>{body}</section>"#,
        escape(title),
        escape(sub)
    )
}

/// Legend with a swatch beside each label (identity never rests on color alone).
///
/// `items` are (label, slot): a 1-based series slot (`svg::slot`) or None for the muted ink. The
/// label carries the meaning; the swatch only links it to the chart.
pub fn legend(items: &[(String, Option<usize>)]) -> String {
    let mut s = String::from(r#"<ul class="legend">"#);
    for (label, slot) in items {
        let color = slot.map_or("var(--muted)".to_string(), svg::slot);
        s.push_str(&format!(
            r#"<li><i style="background:{color}"></i>{}</li>"#,
            escape(label)
        ));
    }
    s.push_str("</ul>");
    s
}

/// A ticket status as people read it ("in_progress" -> "In progress"); unknown statuses pass through.
fn status_label(s: &str) -> String {
    match s {
        "open" => "Open".into(),
        "in_progress" => "In progress".into(),
        "closed" => "Closed".into(),
        other => other.to_string(),
    }
}

/// The Business tab for `d`; `source` names the data under the hero figure ("Shared database PATH" or
/// the mock snapshot).
///
/// Steps:
/// 1. Hero: total revenue and order count, then four tiles from `metrics::kpis`.
/// 2. Revenue by month as columns, with about `MONTH_LABELS` month labels under them.
/// 3. Revenue by category and the top 10 products as horizontal bars.
/// 4. Tickets by status as a donut of the board's three statuses, with a legend.
/// 5. The top 10 customers as a table.
///
/// The five cards of steps 2-5 sit in one `.grid`, which style.css flows into as many columns as fit.
pub fn business(d: &Data, source: &str) -> String {
    let k = metrics::kpis(d);
    let mut out = String::from(r#"<div class="hero">"#);
    out.push_str(&format!(r#"<div class="hero-label">Revenue, {} orders</div><div class="hero-value">{}</div><div class="tile-note">{}</div></div>"#, group(k.orders as u64), money(k.revenue_cents), escape(source)));
    out.push_str(r#"<div class="tiles">"#);
    out.push_str(&tile("Orders", &group(k.orders as u64), "in this data"));
    out.push_str(&tile("Average order", &money(k.avg_order_cents), "revenue / orders"));
    out.push_str(&tile(
        "Buying customers",
        &group(k.customers as u64),
        "placed at least one order",
    ));
    out.push_str(&tile(
        "Open tickets",
        &group(k.open_tickets as u64),
        "open or in progress",
    ));
    out.push_str("</div>");

    // ---- revenue by month ----
    // value in dollars here, not cents: the y-axis ticks print `value` through `compact`
    let months = metrics::revenue_by_month(d);
    let bars: Vec<Bar> = months
        .iter()
        .map(|(m, c)| Bar {
            label: m.clone(),
            value: *c as f64 / 100.0,
            shown: money(*c),
        })
        .collect();
    // 12 months -> every 2nd labelled; under 12 months every one is
    let every = (bars.len() / MONTH_LABELS).max(1);
    let mut grid = card(
        "Revenue by month",
        "Order totals per calendar month",
        &svg::columns(&bars, CHART_W, COLUMNS_H, every, |v| format!("${}", compact(v))),
    );

    // ---- category and product bars ----
    // value in cents: bar charts have no axis, only lengths relative to the longest bar, and `shown`
    // prints the money
    let cats: Vec<Bar> = metrics::revenue_by_category(d)
        .into_iter()
        .map(|(c, v)| Bar {
            label: c,
            value: v as f64,
            shown: money(v),
        })
        .collect();
    grid.push_str(&card(
        "Revenue by product category",
        "All orders in this data",
        &svg::hbars(&cats, CHART_W, CATEGORY_LABEL_W),
    ));

    let prods: Vec<Bar> = metrics::top_products(d, 10)
        .into_iter()
        .map(|(c, v)| Bar {
            label: c,
            value: v as f64,
            shown: money(v),
        })
        .collect();
    grid.push_str(&card(
        "Top 10 products",
        "By revenue",
        &svg::hbars(&prods, CHART_W, PRODUCT_LABEL_W),
    ));

    // ---- tickets by status ----
    // Known limitation: only the first SLOTS statuses (the board's three) become slices and legend
    // entries, while the centre total counts every ticket, so tickets with an unknown status are in
    // the total but in no slice.
    let st = metrics::tickets_by_status(d);
    let slices: Vec<(String, f64)> = st
        .iter()
        .take(SLOTS)
        .map(|(s, n)| (status_label(s), *n as f64))
        .collect();
    let total: usize = st.iter().map(|x| x.1).sum();
    let leg: Vec<(String, Option<usize>)> = st
        .iter()
        .take(SLOTS)
        .enumerate()
        .map(|(i, (s, n))| (format!("{} {}", status_label(s), group(*n as u64)), Some(i + 1)))
        .collect();
    grid.push_str(&card(
        "Tickets by status",
        "Every ticket in this data",
        &format!(
            r#"<div class="donut-row">{}{}</div>"#,
            svg::donut(&slices, DONUT_SIZE, &format!("{} tickets", group(total as u64))),
            legend(&leg)
        ),
    ));

    // ---- top customers ----
    let mut table = String::from(
        r#"<table><thead><tr><th>Customer</th><th>City</th><th class="num">Orders</th><th class="num">Revenue</th></tr></thead><tbody>"#,
    );
    for (name, city, rev, n) in metrics::top_customers(d, 10) {
        table.push_str(&format!(
            r#"<tr><td>{}</td><td>{}</td><td class="num">{n}</td><td class="num">{}</td></tr>"#,
            escape(&name),
            escape(&city),
            money(rev)
        ));
    }
    table.push_str("</tbody></table>");
    grid.push_str(&card("Top 10 customers", "By revenue", &table));

    out.push_str(&format!(r#"<div class="grid">{grid}</div>"#));
    out
}

/// Rows whose classes contain `class` (case-insensitive substring; empty = all).
pub fn filter_rows<'a>(rows: &'a [NumberRow], class: &str) -> Vec<&'a NumberRow> {
    let q = class.trim().to_lowercase();
    rows.iter()
        .filter(|r| q.is_empty() || r.classes.iter().any(|c| c.to_lowercase().contains(&q)))
        .collect()
}

/// The number table for `rows` filtered by `class`, showing 0-based `page` -> (html, page shown).
///
/// Steps:
/// 1. Filter with `filter_rows` and count pages of `PAGE_ROWS`; no hits still makes one (empty) page.
/// 2. Clamp `page` to the last page; the caller stores the returned page, so a stale page number after
///    the filter narrows never shows an empty table.
/// 3. One row per hit on the page, with the first `CLASSES_SHOWN` classes and " +N" for the rest.
/// 4. A pager: Previous and Next buttons whose `data-page` holds the page they go to (`web.rs` reads it
///    on click), disabled at either end, and "N rows, page P of Q".
pub fn number_table(rows: &[NumberRow], class: &str, page: usize) -> (String, usize) {
    let hits = filter_rows(rows, class);
    let pages = hits.len().div_ceil(PAGE_ROWS).max(1);
    let page = page.min(pages - 1);
    let mut s = String::from(
        r#"<table><thead><tr><th class="num">n</th><th>Item</th><th class="num">Sitelinks</th><th>Classes tagged on Wikidata</th></tr></thead><tbody>"#,
    );
    for r in hits.iter().skip(page * PAGE_ROWS).take(PAGE_ROWS) {
        let shown: Vec<String> = r.classes.iter().take(CLASSES_SHOWN).map(|c| escape(c)).collect();
        let more = if r.classes.len() > CLASSES_SHOWN {
            format!(" +{}", r.classes.len() - CLASSES_SHOWN)
        } else {
            String::new()
        };
        s.push_str(&format!(
            r#"<tr><td class="num">{}</td><td>{}</td><td class="num">{}</td><td class="classes">{}{more}</td></tr>"#,
            r.n,
            escape(&r.item),
            r.sitelinks,
            shown.join(", ")
        ));
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

/// The charts of the Numbers & tracks tab, laid out as data/README.md's "Intended charts" describe.
///
/// Steps:
/// 1. Tiles: row count, distinct classes, median sitelinks and the most linked item.
/// 2. numbers.dat: the top 15 classes as bars, prime/composite/neither as a donut, and sitelinks
///    against n as a line.
/// 3. tracks.dat: every track as an x/y path on fixed axes, then samples per track as bars.
///
/// The table under these cards is `number_table`, drawn separately so filtering redraws only it.
pub fn fixtures(rows: &[NumberRow], samples: &[Sample]) -> String {
    let counts = numbers::class_counts(rows);
    let (p, c, n) = numbers::prime_split(rows);
    // back from half-units: an odd value is a median ending in .5
    let med = numbers::median_sitelinks_x2(rows).unwrap_or(0);
    let med_s = if med.is_multiple_of(2) {
        (med / 2).to_string()
    } else {
        format!("{}.5", med / 2)
    };
    let top = numbers::most_linked(rows);

    let mut out = String::from(r#"<div class="tiles">"#);
    out.push_str(&tile("Rows", &rows.len().to_string(), "numbers 1 to 100"));
    out.push_str(&tile(
        "Distinct classes",
        &counts.len().to_string(),
        "tagged on Wikidata",
    ));
    out.push_str(&tile("Median sitelinks", &med_s, "Wikimedia sites per item"));
    out.push_str(&tile(
        "Most linked",
        &top.map_or("-".into(), |t| format!("{} ({})", t.n, t.item)),
        &top.map_or(String::new(), |t| format!("{} sitelinks", t.sitelinks)),
    ));
    out.push_str("</div>");

    // ---- numbers.dat ----
    let bars: Vec<Bar> = counts
        .iter()
        .take(15)
        .map(|(k, v)| Bar {
            label: k.clone(),
            value: *v as f64,
            shown: v.to_string(),
        })
        .collect();
    let mut grid = card(
        "Top 15 classes",
        "Rows per class, tagged on Wikidata",
        &svg::hbars(&bars, CHART_W, CLASS_LABEL_W),
    );
    let slices = [
        ("Prime".to_string(), p as f64),
        ("Composite".to_string(), c as f64),
        ("Neither".to_string(), n as f64),
    ];
    let leg = vec![
        (format!("Prime {p}"), Some(1)),
        (format!("Composite {c}"), Some(2)),
        (format!("Neither {n}"), Some(3)),
    ];
    grid.push_str(&card(
        "Prime, composite or neither",
        "Tagged on Wikidata",
        &format!(
            r#"<div class="donut-row">{}{}</div>"#,
            svg::donut(&slices, DONUT_SIZE, "100 numbers"),
            legend(&leg)
        ),
    ));
    // x ticks at 1 and every multiple of 10, where data/README.md notes the sitelink bumps
    let pts: Vec<(f64, f64)> = rows.iter().map(|r| (r.n as f64, r.sitelinks as f64)).collect();
    grid.push_str(&card(
        "Sitelinks by number",
        "How many Wikimedia sites link each item",
        &svg::line(
            &pts,
            CHART_W,
            LINE_H,
            &[1.0, 10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0],
            "n",
        ),
    ));

    // ---- tracks.dat ----
    // moving tracks take slots 1..=SLOTS in id order; a fixed point (or a moving track past the
    // slots) is drawn in the muted ink and its legend label says so
    let by = tracks::by_track(samples);
    let mut moving = 0;
    let mut series = Vec::new();
    let mut leg = Vec::new();
    for (id, pts) in &by {
        let still = tracks::is_stationary(pts);
        let slot = if still || moving >= SLOTS {
            None
        } else {
            moving += 1;
            Some(moving)
        };
        let label = if still {
            format!("Track {id} (fixed point)")
        } else {
            format!("Track {id}")
        };
        leg.push((label.clone(), slot));
        series.push(XyPath {
            label,
            points: pts.iter().map(|s| (s.x, s.y)).collect(),
            slot,
        });
    }
    grid.push_str(&card(
        "Track paths",
        "x against y on fixed axes, in time order",
        &format!(
            "{}{}",
            legend(&leg),
            // the fixed -300..300 by -200..200 axes data/README.md asks for, so shapes keep their proportions
            svg::paths(&series, (-300.0, 300.0), (-200.0, 200.0), CHART_W)
        ),
    ));
    let counts: Vec<Bar> = by
        .iter()
        .map(|(id, v)| Bar {
            label: format!("Track {id}"),
            value: v.len() as f64,
            shown: v.len().to_string(),
        })
        .collect();
    grid.push_str(&card(
        "Samples per track",
        "Rows in tracks.dat",
        &svg::hbars(&counts, CHART_W, TRACK_LABEL_W),
    ));
    out.push_str(&format!(r#"<div class="grid">{grid}</div>"#));
    out
}

/// Unit tests: filtering and paging, and both views rendered from the real fixtures and mock snapshot.
#[cfg(test)]
mod tests {
    use super::*;

    /// The 100 rows of data/numbers.dat.
    fn rows() -> Vec<NumberRow> {
        numbers::parse(include_str!("../../data/numbers.dat")).unwrap()
    }

    /// The filter ignores case and outer spaces; a page past the end clamps to the last (10 rows plus
    /// the header row); no hits still gives "page 1 of 1".
    #[test]
    fn filter_and_paging() {
        let r = rows();
        assert_eq!(filter_rows(&r, "").len(), 100);
        assert_eq!(
            filter_rows(&r, " PRIME NUMBER").len(),
            filter_rows(&r, "prime number").len()
        );
        let (html, page) = number_table(&r, "", 99);
        assert_eq!(page, 6); // 100 rows / 15 = 7 pages, clamped to the last
        assert!(html.contains("page 7 of 7"));
        assert_eq!(html.matches("<tr>").count(), 1 + 10);
        let (none, _) = number_table(&r, "no such class", 0);
        assert!(none.contains("0 rows, page 1 of 1"));
    }

    /// Both views render without "NaN": the fixture view shows the 144 classes and labels track 7 as
    /// fixed; the business view escapes `source` and has its five cards.
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

// ==== stamp v1 | author=kjFriday | hmac-sha256=03cb5f14b2335556b30a9187e41f353557cfbf644da559fd7eb9f86c1f2d486d ====
