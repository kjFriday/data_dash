//! Page wiring. The Business view reads the embedded mock snapshot (standalone) or the live CRUD hub;
//! the Fixtures view always reads data/numbers.dat and data/tracks.dat, compiled in.

use crate::core::metrics::Data;
use crate::core::numbers::{self, NumberRow};
use crate::core::render;
use crate::core::tracks;
use serde_json::Value;
use std::cell::RefCell;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::{spawn_local, JsFuture};
use web_sys::{Element, Event, HtmlInputElement, HtmlSelectElement, Request, RequestInit, Response};

const MOCK: [&str; 4] = [
    include_str!("../mock/orders.json"),
    include_str!("../mock/products.json"),
    include_str!("../mock/customers.json"),
    include_str!("../mock/tickets.json"),
];
const NUMBERS: &str = include_str!("../data/numbers.dat");
const TRACKS: &str = include_str!("../data/tracks.dat");
const DEFAULT_HUB: &str = "http://127.0.0.1:8700";
const PAGE: usize = 500; // the hub's largest ?limit

struct App {
    numbers: Vec<NumberRow>,
    class: String,
    page: usize,
}

thread_local! {
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

fn with<R>(f: impl FnOnce(&mut App) -> R) -> R {
    APP.with(|a| f(a.borrow_mut().as_mut().expect("app started")))
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let numbers = numbers::parse(NUMBERS).map_err(|e| JsValue::from_str(&e))?;
    let samples = tracks::parse(TRACKS).map_err(|e| JsValue::from_str(&e))?;
    el("fixture-charts").set_inner_html(&render::fixtures(&numbers, &samples));
    APP.with(|a| *a.borrow_mut() = Some(App { numbers, class: String::new(), page: 0 }));
    draw_table();
    input("hub-url").set_value(&query_param("hub").unwrap_or_else(|| DEFAULT_HUB.into()));
    wire()?;
    if query_param("tab").as_deref() == Some("fixtures") {
        show_tab("fixtures");
    }
    if query_param("source").as_deref() == Some("hub") {
        select("source").set_value("hub");
        spawn_local(load_hub());
    } else {
        use_mock("Standalone: mock snapshot");
    }
    Ok(())
}

fn use_mock(why: &str) {
    select("source").set_value("mock");
    match Data::from_json(MOCK[0], MOCK[1], MOCK[2], MOCK[3]) {
        Ok(d) => {
            el("business").set_inner_html(&render::business(&d, "Mock snapshot bundled with this app"));
            set_mode(false);
            status(&format!("{why}: {} orders, {} tickets.", d.orders.len(), d.tickets.len()), false);
        }
        Err(e) => status(&format!("Mock snapshot unreadable: {e}"), true),
    }
}

async fn load_hub() {
    let hub = input("hub-url").value().trim().trim_end_matches('/').to_string();
    status(&format!("Connecting to {hub} ..."), false);
    let loaded = async {
        let mut parts = Vec::new();
        for table in ["orders", "products", "customers", "tickets"] {
            status(&format!("Reading {hub}/api/{table} ..."), false);
            parts.push(Value::Array(fetch_all(&hub, table).await?).to_string());
        }
        Data::from_json(&parts[0], &parts[1], &parts[2], &parts[3])
    }
    .await;
    match loaded {
        Ok(d) => {
            el("business").set_inner_html(&render::business(&d, &format!("Live from {hub}")));
            set_mode(true);
            status(&format!("Live hub {hub}: {} orders, {} tickets, read with GET /api/{{table}}?limit={PAGE}&offset=N.", d.orders.len(), d.tickets.len()), false);
        }
        Err(err) => {
            use_mock("Hub unreachable");
            status(&format!("Hub unreachable at {hub} ({err}). Showing the mock snapshot."), true);
        }
    }
}

async fn fetch_all(hub: &str, table: &str) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    loop {
        let url = format!("{hub}/api/{table}?limit={PAGE}&offset={}", rows.len());
        let init = RequestInit::new();
        init.set_method("GET");
        let req = Request::new_with_str_and_init(&url, &init).map_err(js_err)?;
        let win = web_sys::window().ok_or("no window")?;
        let resp: Response = JsFuture::from(win.fetch_with_request(&req)).await.map_err(js_err)?.dyn_into().map_err(js_err)?;
        let text = JsFuture::from(resp.text().map_err(js_err)?).await.map_err(js_err)?.as_string().unwrap_or_default();
        if resp.status() != 200 {
            return Err(format!("{} on {table}", resp.status()));
        }
        let page: Vec<Value> = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let done = page.len() < PAGE;
        rows.extend(page);
        if done {
            return Ok(rows);
        }
    }
}

fn js_err(e: JsValue) -> String {
    if let Some(err) = e.dyn_ref::<js_sys::Error>() {
        return err.message().into();
    }
    e.as_string().unwrap_or_else(|| "request failed".into())
}

// ---- DOM ----

fn el(id: &str) -> Element {
    web_sys::window().unwrap().document().unwrap().get_element_by_id(id).unwrap_or_else(|| panic!("#{id} missing"))
}

fn input(id: &str) -> HtmlInputElement {
    el(id).dyn_into().unwrap()
}

fn select(id: &str) -> HtmlSelectElement {
    el(id).dyn_into().unwrap()
}

fn status(msg: &str, error: bool) {
    let s = el("status");
    s.set_text_content(Some(msg));
    s.set_class_name(if error { "error" } else { "" });
}

fn set_mode(live: bool) {
    let m = el("mode");
    m.set_text_content(Some(if live { "live hub" } else { "mock" }));
    m.set_class_name(if live { "badge live" } else { "badge" });
}

fn query_param(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search.trim_start_matches('?').split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then(|| js_sys::decode_uri_component(&v.replace('+', " ")).ok().map(String::from)).flatten()
    })
}

fn show_tab(name: &str) {
    for tab in ["business", "fixtures"] {
        let on = tab == name;
        let _ = el(tab).toggle_attribute_with_force("hidden", !on);
        let _ = el(&format!("tab-{tab}")).set_attribute("aria-selected", if on { "true" } else { "false" });
    }
}

fn draw_table() {
    let html = with(|a| {
        let (html, page) = render::number_table(&a.numbers, &a.class, a.page);
        a.page = page;
        html
    });
    el("number-table").set_inner_html(&html);
}

fn on(id: &str, kind: &str, f: impl FnMut(Event) + 'static) -> Result<(), JsValue> {
    let cb = Closure::<dyn FnMut(Event)>::new(f);
    el(id).add_event_listener_with_callback(kind, cb.as_ref().unchecked_ref())?;
    cb.forget(); // listeners live as long as the page
    Ok(())
}

fn wire() -> Result<(), JsValue> {
    let reload = |_| {
        if select("source").value() == "hub" {
            spawn_local(load_hub());
        } else {
            use_mock("Standalone: mock snapshot");
        }
    };
    on("source", "change", reload)?;
    on("reload", "click", reload)?;
    on("tab-business", "click", |_| show_tab("business"))?;
    on("tab-fixtures", "click", |_| show_tab("fixtures"))?;
    on("class-filter", "input", |_| {
        with(|a| {
            a.class = input("class-filter").value();
            a.page = 0;
        });
        draw_table();
    })?;
    on("number-table", "click", |ev| {
        let page = ev
            .target()
            .and_then(|t| t.dyn_into::<Element>().ok())
            .and_then(|t| t.closest("button[data-page]").ok().flatten())
            .and_then(|b| b.get_attribute("data-page"))
            .and_then(|p| p.parse().ok());
        if let Some(p) = page {
            with(|a| a.page = p);
            draw_table();
        }
    })?;
    Ok(())
}
