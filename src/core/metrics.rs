//! Business metrics over hub rows. Money is summed in whole cents, never as floats.

use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Deserialize)]
pub struct Order {
    pub id: i64,
    #[serde(default)]
    pub customer_id: Option<i64>,
    #[serde(default)]
    pub product_id: Option<i64>,
    #[serde(default)]
    pub quantity: Option<i64>,
    #[serde(default)]
    pub total: Option<f64>,
    #[serde(default)]
    pub ordered_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Product {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Customer {
    pub id: i64,
    pub name: String,
    #[serde(default)]
    pub city: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Ticket {
    pub id: i64,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct Data {
    pub orders: Vec<Order>,
    pub products: Vec<Product>,
    pub customers: Vec<Customer>,
    pub tickets: Vec<Ticket>,
}

impl Data {
    pub fn from_json(orders: &str, products: &str, customers: &str, tickets: &str) -> Result<Data, String> {
        Ok(Data {
            orders: serde_json::from_str(orders).map_err(|e| format!("orders: {e}"))?,
            products: serde_json::from_str(products).map_err(|e| format!("products: {e}"))?,
            customers: serde_json::from_str(customers).map_err(|e| format!("customers: {e}"))?,
            tickets: serde_json::from_str(tickets).map_err(|e| format!("tickets: {e}"))?,
        })
    }
}

pub fn cents(amount: Option<f64>) -> i64 {
    amount.filter(|a| a.is_finite()).map_or(0, |a| (a * 100.0).round() as i64)
}

/// "$1,234.56"
pub fn money(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let c = cents.unsigned_abs();
    format!("{sign}${}.{:02}", group(c / 100), c % 100)
}

/// 1234567 -> "1,234,567"
pub fn group(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// 950 -> "950", 12_900 -> "12.9K", 4_200_000 -> "4.2M" (axis ticks and tiles).
pub fn compact(v: f64) -> String {
    let a = v.abs();
    let (d, suffix) = if a >= 1e6 { (1e6, "M") } else if a >= 1e3 { (1e3, "K") } else { (1.0, "") };
    let x = v / d;
    if suffix.is_empty() || (x.fract().abs() < 0.05) || x.abs() >= 100.0 {
        format!("{:.0}{suffix}", x)
    } else {
        format!("{:.1}{suffix}", x)
    }
}

pub struct Kpis {
    pub orders: usize,
    pub revenue_cents: i64,
    pub avg_order_cents: i64,
    pub customers: usize,
    pub open_tickets: usize,
}

pub fn kpis(d: &Data) -> Kpis {
    let revenue: i64 = d.orders.iter().map(|o| cents(o.total)).sum();
    let buyers: std::collections::HashSet<i64> = d.orders.iter().filter_map(|o| o.customer_id).collect();
    Kpis {
        orders: d.orders.len(),
        revenue_cents: revenue,
        avg_order_cents: if d.orders.is_empty() { 0 } else { revenue / d.orders.len() as i64 },
        customers: buyers.len(),
        open_tickets: d.tickets.iter().filter(|t| t.status.as_deref() != Some("closed")).count(),
    }
}

/// Revenue per calendar month ("2026-03"), oldest first.
pub fn revenue_by_month(d: &Data) -> Vec<(String, i64)> {
    let mut m: BTreeMap<String, i64> = BTreeMap::new();
    for o in &d.orders {
        if let Some(month) = o.ordered_at.as_deref().and_then(|s| s.get(..7)) {
            *m.entry(month.to_string()).or_default() += cents(o.total);
        }
    }
    m.into_iter().collect()
}

fn sorted_desc(m: HashMap<String, i64>) -> Vec<(String, i64)> {
    let mut v: Vec<(String, i64)> = m.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

pub fn revenue_by_category(d: &Data) -> Vec<(String, i64)> {
    let cat: HashMap<i64, &str> = d.products.iter().map(|p| (p.id, p.category.as_deref().unwrap_or("Uncategorised"))).collect();
    let mut m: HashMap<String, i64> = HashMap::new();
    for o in &d.orders {
        let c = o.product_id.and_then(|id| cat.get(&id).copied()).unwrap_or("Unknown");
        *m.entry(c.to_string()).or_default() += cents(o.total);
    }
    sorted_desc(m)
}

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
            (c.map_or(format!("Customer #{id}"), |c| c.name.clone()), c.and_then(|c| c.city.clone()).unwrap_or_default(), rev, k)
        })
        .collect();
    v.sort_by(|a, b| b.2.cmp(&a.2).then(a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

/// Tickets per status in board order; unknown statuses are counted as they come.
pub fn tickets_by_status(d: &Data) -> Vec<(String, usize)> {
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    for t in &d.tickets {
        *m.entry(t.status.clone().unwrap_or_else(|| "open".into())).or_default() += 1;
    }
    let order = ["open", "in_progress", "closed"];
    let mut v: Vec<(String, usize)> = order.iter().map(|s| (s.to_string(), m.remove(*s).unwrap_or(0))).collect();
    v.extend(m);
    v
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn sums_in_cents() {
        // 0.1 + 0.2 style float drift must not show up
        assert_eq!(cents(Some(10.10)) + cents(Some(0.20)), 1030);
        let k = kpis(&data());
        assert_eq!((k.orders, k.revenue_cents, k.avg_order_cents, k.customers, k.open_tickets), (4, 1535, 383, 3, 4));
    }

    #[test]
    fn groups_by_month_category_and_customer() {
        let d = data();
        assert_eq!(revenue_by_month(&d), vec![("2026-01".into(), 1030), ("2026-02".into(), 505)]);
        assert_eq!(revenue_by_category(&d), vec![("Compiled".into(), 1010), ("Embedded".into(), 525), ("Unknown".into(), 0)]);
        assert_eq!(top_products(&d, 1), vec![("Ada Toolkit".into(), 1010)]);
        let top = top_customers(&d, 2);
        assert_eq!(top[0], ("Maria Garcia".into(), "Lyon".into(), 1515, 2));
        assert_eq!(top[1].0, "Kenji Sato");
    }

    #[test]
    fn status_counts_keep_board_order() {
        let v = tickets_by_status(&data());
        assert_eq!(v, vec![("open".into(), 2), ("in_progress".into(), 1), ("closed".into(), 1), ("weird".into(), 1)]);
    }

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
