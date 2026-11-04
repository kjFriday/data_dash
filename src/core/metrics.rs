// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/core/metrics.rs
// About: Business metrics over hub rows.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Business metrics over hub rows. Money is summed in whole cents, never as floats.
//!
//! The rows are the four Generic Company tables (`orders`, `products`, `customers`, `tickets`) as JSON
//! arrays, either read page by page from the desktop program's `/api/{table}` (`web.rs::fetch_all`) or
//! taken from the bundled `mock/*.json` snapshot. `render::business` is the caller.
//!
//! Flow:
//! 1. `Data::from_json` parses the four arrays into `Order`, `Product`, `Customer` and `Ticket` rows.
//!    Columns the dashboard does not use (email, price, title ...) are ignored.
//! 2. `cents` turns each order's float `total` into whole cents once; every sum after that is integer
//!    arithmetic, so 0.10 + 0.20 adds up to exactly 30 cents.
//! 3. `kpis`, `revenue_by_month`, `revenue_by_category`, `top_products`, `top_customers` and
//!    `tickets_by_status` aggregate.
//! 4. `money`, `group` and `compact` format the results for tiles, tables and axis ticks.
//!
//! Money becomes `f64` again only for chart geometry (bar lengths), never for a printed figure.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

/// One row of the `orders` table. Every column but `id` may be null or missing.
#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: i64,
    /// `customers.id` of the buyer.
    #[serde(default)]
    pub customer_id: Option<i64>,
    /// `products.id` of the item bought.
    #[serde(default)]
    pub product_id: Option<i64>,
    #[serde(default)]
    pub quantity: Option<i64>,
    /// Order total in dollars as SQLite `REAL`; read once through `cents`.
    #[serde(default)]
    pub total: Option<f64>,
    /// Date and time as text, "YYYY-MM-DDTHH:MM" (for example "2026-06-25T23:37").
    #[serde(default)]
    pub ordered_at: Option<String>,
}

/// One row of the `products` table: the name and category the charts label.
#[derive(Debug, Clone, Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    /// None is charted as "Uncategorised".
    #[serde(default)]
    pub category: Option<String>,
}

/// One row of the `customers` table: the name and city the top-customers table shows.
#[derive(Debug, Clone, Deserialize)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub city: Option<String>,
}

/// One row of the `tickets` table; only the status matters here.
#[derive(Debug, Clone, Deserialize)]
pub struct Ticket {
    pub id: i64,
    /// "open", "in_progress" or "closed" on the hub's board; None counts as "open" (the column's default).
    #[serde(default)]
    pub status: Option<String>,
}

/// All four tables at once: the input to every metric in this file.
#[derive(Debug, Clone, Default)]
pub struct Data {
    pub orders: Vec<Order>,
    pub products: Vec<Product>,
    pub customers: Vec<Customer>,
    pub tickets: Vec<Ticket>,
}

// parsing from the JSON the API and the mock snapshot share
impl Data {
    /// Parses the four tables' JSON arrays -> `Data`.
    ///
    /// Errors: the first serde error, prefixed with its table name ("orders: missing field `id` ...").
    /// Extra columns are ignored, so the full rows from `/api/{table}` and `mock/*.json` both fit.
    pub fn from_json(orders: &str, products: &str, customers: &str, tickets: &str) -> Result<Data, String> {
        Ok(Data {
            orders: serde_json::from_str(orders).map_err(|e| format!("orders: {e}"))?,
            products: serde_json::from_str(products).map_err(|e| format!("products: {e}"))?,
            customers: serde_json::from_str(customers).map_err(|e| format!("customers: {e}"))?,
            tickets: serde_json::from_str(tickets).map_err(|e| format!("tickets: {e}"))?,
        })
    }
}

/// A dollar amount -> whole cents; None, NaN and infinities count as 0.
///
/// The float is rounded, not truncated: 10.10 is stored as 10.0999..., so `10.10 * 100.0` is
/// 1009.99... and only `round` gives 1010. `as i64` saturates on out-of-range values.
pub fn cents(amount: Option<f64>) -> i64 {
    amount
        .filter(|a| a.is_finite())
        .map_or(0, |a| (a * 100.0).round() as i64)
}

/// "$1,234.56"
///
/// `cents` -> dollars with thousands groups and two decimals; a negative amount gets "-" before the "$".
pub fn money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    // unsigned_abs: plain abs() would overflow on i64::MIN
    let c = cents.unsigned_abs();
    format!("{sign}${}.{:02}", group(c / 100), c % 100)
}

/// 1234567 -> "1,234,567"
///
/// Thousands groups with commas, as in en-US; the page does not localise numbers.
pub fn group(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        // a comma goes before each digit that has a multiple of 3 digits from it to the end
        // (the string is ASCII digits, so byte length = digit count)
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// 950 -> "950", 12_900 -> "12.9K", 4_200_000 -> "4.2M" (axis ticks and tiles).
///
/// Steps:
/// 1. Pick the unit from the magnitude: M from one million, K from one thousand, else none.
/// 2. Divide by it and print with no decimal when there is no unit, when the fraction is under 0.05
///    (2_000 -> "2K", not "2.0K"), or when the scaled value is 100 or more ("250K"); one decimal otherwise.
///
/// Known limitation: the 0.05 test looks at the fraction before rounding, so 1_960 prints "2.0K", and
/// 999_950 stays in K and prints "1000K".
pub fn compact(v: f64) -> String {
    let a = v.abs();
    let (d, suffix) = if a >= 1e6 {
        (1e6, "M")
    } else if a >= 1e3 {
        (1e3, "K")
    } else {
        (1.0, "")
    };
    let x = v / d;
    if suffix.is_empty() || (x.fract().abs() < 0.05) || x.abs() >= 100.0 {
        format!("{:.0}{suffix}", x)
    } else {
        format!("{:.1}{suffix}", x)
    }
}

/// The headline figures of the Business tab (hero and tiles).
pub struct Kpis {
    /// Every order row, including ones with no total.
    pub orders: usize,
    pub revenue_cents: i64,
    /// `revenue_cents / orders`, truncated toward zero; 0 with no orders.
    pub avg_order_cents: i64,
    /// Distinct customer ids that placed at least one order (not the size of the customers table).
    pub customers: usize,
    /// Tickets whose status is anything but "closed".
    pub open_tickets: usize,
}

/// Computes the headline figures over all of `d`.
///
/// Known limitation: `open_tickets` counts every status other than "closed", so an unknown status (or
/// none) counts as open, while the tile's note says "open or in progress".
pub fn kpis(d: &Data) -> Kpis {
    let revenue: i64 = d.orders.iter().map(|o| cents(o.total)).sum();
    let buyers: std::collections::HashSet<i64> = d.orders.iter().filter_map(|o| o.customer_id).collect();
    Kpis {
        orders: d.orders.len(),
        revenue_cents: revenue,
        avg_order_cents: if d.orders.is_empty() {
            0
        } else {
            revenue / d.orders.len() as i64
        },
        customers: buyers.len(),
        open_tickets: d
            .tickets
            .iter()
            .filter(|t| t.status.as_deref() != Some("closed"))
            .count(),
    }
}

/// Revenue per calendar month ("2026-03"), oldest first.
///
/// -> (month, cents) pairs. The month is the first 7 characters of `ordered_at`; the `BTreeMap` keeps
/// the keys sorted, and "YYYY-MM" sorts as text in time order. Months with no orders do not appear.
pub fn revenue_by_month(d: &Data) -> Vec<(String, i64)> {
    let mut m: BTreeMap<String, i64> = BTreeMap::new();
    for o in &d.orders {
        // orders with no date (or one shorter than 7 bytes) are left out, so the months can sum to less
        // than the hero's revenue
        if let Some(month) = o.ordered_at.as_deref().and_then(|s| s.get(..7)) {
            *m.entry(month.to_string()).or_default() += cents(o.total);
        }
    }
    m.into_iter().collect()
}

/// The map's (name, cents) pairs, biggest first; ties go by name so the order never depends on hashing.
fn sorted_desc(m: HashMap<String, i64>) -> Vec<(String, i64)> {
    let mut v: Vec<(String, i64)> = m.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

/// Revenue per product category -> (category, cents), biggest first.
///
/// A product with no category counts as "Uncategorised"; an order whose product is missing or unknown
/// counts as "Unknown" (with its total, which may be 0).
pub fn revenue_by_category(d: &Data) -> Vec<(String, i64)> {
    let cat: HashMap<i64, &str> = d
        .products
        .iter()
        .map(|p| (p.id, p.category.as_deref().unwrap_or("Uncategorised")))
        .collect();
    let mut m: HashMap<String, i64> = HashMap::new();
    for o in &d.orders {
        let c = o.product_id.and_then(|id| cat.get(&id).copied()).unwrap_or("Unknown");
        *m.entry(c.to_string()).or_default() += cents(o.total);
    }
    sorted_desc(m)
}

/// The `n` products with the most revenue -> (product name, cents), biggest first.
///
/// Revenue is summed by name, so two products that share a name count as one; orders with a missing
/// or unknown product count as "Unknown".
pub fn top_products(d: &Data, n: usize) -> Vec<(String, i64)> {
    let name: HashMap<i64, &str> = d.products.iter().map(|p| (p.id, p.name.as_str())).collect();
    let mut m: HashMap<String, i64> = HashMap::new();
    for o in &d.orders {
        let p = o.product_id.and_then(|id| name.get(&id).copied()).unwrap_or("Unknown");
        *m.entry(p.to_string()).or_default() += cents(o.total);
    }
    sorted_desc(m).into_iter().take(n).collect()
}

/// (name, city, revenue cents, orders) for the biggest spenders.
///
/// Steps:
/// 1. Sum cents and count orders per `customer_id` (orders with none are skipped).
/// 2. Look each id up in `customers`: an unknown id shows as "Customer #id" with no city.
/// 3. Sort by revenue, biggest first (ties by name), and keep the first `n`.
pub fn top_customers(d: &Data, n: usize) -> Vec<(String, String, i64, usize)> {
    let mut m: HashMap<i64, (i64, usize)> = HashMap::new();
    for o in &d.orders {
        if let Some(c) = o.customer_id {
            let e = m.entry(c).or_default();
            e.0 += cents(o.total);
            e.1 += 1;
        }
    }
    let by_id: HashMap<i64, &Customer> = d.customers.iter().map(|c| (c.id, c)).collect();
    let mut v: Vec<(String, String, i64, usize)> = m
        .into_iter()
        .map(|(id, (rev, k))| {
            let c = by_id.get(&id);
            (
                c.map_or(format!("Customer #{id}"), |c| c.name.clone()),
                c.and_then(|c| c.city.clone()).unwrap_or_default(),
                rev,
                k,
            )
        })
        .collect();
    v.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

/// Tickets per status in board order; unknown statuses are counted as they come.
///
/// -> (status, count): "open", "in_progress" and "closed" always come first in that order (0 when
/// absent), then any other status in alphabetical order. A ticket with no status counts as "open".
pub fn tickets_by_status(d: &Data) -> Vec<(String, usize)> {
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    for t in &d.tickets {
        *m.entry(t.status.clone().unwrap_or_else(|| "open".into())).or_default() += 1;
    }
    // the hub's board columns; whatever `remove` leaves in `m` is appended in key order
    let order = ["open", "in_progress", "closed"];
    let mut v: Vec<(String, usize)> = order
        .iter()
        .map(|s| (s.to_string(), m.remove(*s).unwrap_or(0)))
        .collect();
    v.extend(m);
    v
}

/// Unit tests: cent sums, groupings, board order, formatting and the bundled mock snapshot.
#[cfg(test)]
mod tests {
    use super::*;

    /// Four orders (one with a null total and unknown customer/product), two products, two customers
    /// (one without a city) and five tickets (one unknown status, one with none).
    fn data() -> Data {
        Data::from_json(
            r#"[{"id":1,"customer_id":1,"product_id":1,"total":10.10,"ordered_at":"2026-01-05T10:00"},
                {"id":2,"customer_id":2,"product_id":2,"total":0.20,"ordered_at":"2026-01-20T10:00"},
                {"id":3,"customer_id":1,"product_id":2,"total":5.05,"ordered_at":"2026-02-01T10:00"},
                {"id":4,"customer_id":9,"product_id":7,"total":null}]"#,
            r#"[{"id":1,"name":"Ada Toolkit","category":"Compiled"},{"id":2,"name":"Lua Toolkit","category":"Embedded"}]"#,
            r#"[{"id":1,"name":"Maria Garcia","city":"Lyon"},{"id":2,"name":"Kenji Sato"}]"#,
            r#"[{"id":1,"status":"open"},{"id":2,"status":"closed"},{"id":3,"status":"in_progress"},{"id":4,"status":"weird"},{"id":5}]"#,
        )
        .unwrap()
    }

    /// Float totals add up exactly in cents, and the KPIs count null totals, distinct buyers and
    /// non-closed tickets as documented (1535 / 4 truncates to 383).
    #[test]
    fn sums_in_cents() {
        // 0.1 + 0.2 style float drift must not show up
        assert_eq!(cents(Some(10.10)) + cents(Some(0.20)), 1030);
        let k = kpis(&data());
        assert_eq!(
            (
                k.orders,
                k.revenue_cents,
                k.avg_order_cents,
                k.customers,
                k.open_tickets
            ),
            (4, 1535, 383, 3, 4)
        );
    }

    /// Months sort oldest first, the undated order drops out of months but lands in "Unknown", and
    /// the top lists sort by revenue.
    #[test]
    fn groups_by_month_category_and_customer() {
        let d = data();
        assert_eq!(
            revenue_by_month(&d),
            vec![("2026-01".into(), 1030), ("2026-02".into(), 505)]
        );
        assert_eq!(
            revenue_by_category(&d),
            vec![
                ("Compiled".into(), 1010),
                ("Embedded".into(), 525),
                ("Unknown".into(), 0)
            ]
        );
        assert_eq!(top_products(&d, 1), vec![("Ada Toolkit".into(), 1010)]);
        let top = top_customers(&d, 2);
        assert_eq!(top[0], ("Maria Garcia".into(), "Lyon".into(), 1515, 2));
        assert_eq!(top[1].0, "Kenji Sato");
    }

    /// Board statuses come first in board order, a missing status counts as open, unknown ones follow.
    #[test]
    fn status_counts_keep_board_order() {
        let v = tickets_by_status(&data());
        assert_eq!(
            v,
            vec![
                ("open".into(), 2),
                ("in_progress".into(), 1),
                ("closed".into(), 1),
                ("weird".into(), 1)
            ]
        );
    }

    /// Money keeps two decimals and a leading sign; `compact` drops ".0" and picks K or M.
    #[test]
    fn formatting() {
        assert_eq!(money(123456789), "$1,234,567.89");
        assert_eq!(money(-5), "-$0.05");
        assert_eq!(group(0), "0");
        assert_eq!(compact(950.0), "950");
        assert_eq!(compact(12_900.0), "12.9K");
        assert_eq!(compact(2_000.0), "2K");
        assert_eq!(compact(4_200_000.0), "4.2M");
    }

    /// The bundled snapshot parses, has the 3,000 orders the README states, and every order's
    /// customer exists.
    #[test]
    fn mock_snapshot_loads() {
        let d = Data::from_json(
            include_str!("../../mock/orders.json"),
            include_str!("../../mock/products.json"),
            include_str!("../../mock/customers.json"),
            include_str!("../../mock/tickets.json"),
        )
        .unwrap();
        assert_eq!(d.orders.len(), 3000);
        let ids: std::collections::HashSet<i64> = d.customers.iter().map(|c| c.id).collect();
        assert!(d.orders.iter().all(|o| o.customer_id.is_some_and(|c| ids.contains(&c))));
        assert!(kpis(&d).revenue_cents > 0);
    }
}

// ==== stamp v1 | author=kjFriday | hmac-sha256=ee5098729efa49bccb9eeaa5c33f22d8687cbefe5b1d8993672219c56b4acb72 ====
