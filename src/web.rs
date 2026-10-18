//! Page wiring. The Business view reads the shared database, served at the same origin by the desktop
//! program (`/api/...`), or the embedded mock snapshot when there is none (or in a plain browser);
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
const API: &str = ""; // same origin: the desktop program answers /api/... from the shared database
const PAGE: usize = 500; // the largest ?limit

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
    el("version").set_text_content(Some(concat!("ACME Data Dash ", env!("CARGO_PKG_VERSION"))));
    wire()?;
    if query_param("tab").as_deref() == Some("fixtures") {
        show_tab("fixtures");
    }
    if query_param("source").as_deref() == Some("mock") {
        use_mock("Mock snapshot");
    } else {
        select("source").set_value("hub");
        spawn_local(load_hub());
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
    let hub = API.to_string();
    let Some(db) = source_db().await else {
        use_mock("No shared database here, so the mock snapshot");
        return;
    };
    status("Opening the shared database ...", false);
    let loaded = async {
        let mut parts = Vec::new();
        for table in ["orders", "products", "customers", "tickets"] {
            status(&format!("Reading {table} ..."), false);
            parts.push(Value::Array(fetch_all(&hub, table).await?).to_string());
        }
        Data::from_json(&parts[0], &parts[1], &parts[2], &parts[3])
    }
    .await;
    match loaded {
        Ok(d) => {
            el("business").set_inner_html(&render::business(&d, &format!("Shared database {db}")));
            set_mode(true);
            status(&format!("Shared database {db}: {} orders, {} tickets, read with GET /api/{{table}}?limit={PAGE}&offset=N.", d.orders.len(), d.tickets.len()), false);
        }
        Err(err) => {
            use_mock("Database unavailable");
            status(&format!("Could not read the shared database ({err}). Showing the mock snapshot."), true);
        }
    }
}

/// The database the desktop program serves, or None (no database, or a plain browser).
async fn source_db() -> Option<String> {
    let text = get_text("/api/source").await.ok()?;
    serde_json::from_str::<Value>(&text).ok()?["db"].as_str().map(String::from)
}

async fn get_text(url: &str) -> Result<String, String> {
    let init = RequestInit::new();
    init.set_method("GET");
    let req = Request::new_with_str_and_init(url, &init).map_err(js_err)?;
    let win = web_sys::window().ok_or("no window")?;
    let resp: Response = JsFuture::from(win.fetch_with_request(&req)).await.map_err(js_err)?.dyn_into().map_err(js_err)?;
    if resp.status() != 200 {
        return Err(format!("{} on {url}", resp.status()));
    }
    Ok(JsFuture::from(resp.text().map_err(js_err)?).await.map_err(js_err)?.as_string().unwrap_or_default())
}

/// Ask for another database file; the desktop program checks it and saves the choice.
async fn change_db() {
    let win = web_sys::window().unwrap();
    let Ok(Some(path)) = win.prompt_with_message("Path of the shared ACME database (acme.db):") else { return };
    if path.trim().is_empty() {
        return;
    }
    let init = RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(&serde_json::json!({ "path": path.trim() }).to_string()));
    let Ok(req) = Request::new_with_str_and_init("/host/db", &init) else { return };
    let resp = match JsFuture::from(win.fetch_with_request(&req)).await.ok().and_then(|r| r.dyn_into::<Response>().ok()) {
        Some(r) => r,
        None => return status("Choosing a database needs the desktop program.", true),
    };
    if resp.status() == 200 {
        spawn_local(load_hub());
    } else {
        let text = match resp.text() {
            Ok(p) => JsFuture::from(p).await.ok().and_then(|v| v.as_string()).unwrap_or_default(),
            Err(_) => String::new(),
        };
        let msg = serde_json::from_str::<Value>(&text).ok().and_then(|v| v["error"].as_str().map(String::from)).unwrap_or(text);
        status(&format!("Not used: {msg}"), true);
    }
}

async fn about() {
    let db = source_db().await.unwrap_or_else(|| "none (mock snapshot)".into());
    let text = format!(
        "ACME Data Dash {}\nACME Inc. (fictional)\n\nDatabase: {db}\n\nIcon: Silk \"chart_bar\" by Mark James (famfamfam.com/lab/icons/silk), CC BY 2.5, enlarged by nearest-neighbour.\nCode: AGPL-3.0. All data is fictional.",
        env!("CARGO_PKG_VERSION")
    );
    let _ = web_sys::window().unwrap().alert_with_message(&text);
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
    m.set_text_content(Some(if live { "shared database" } else { "mock" }));
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
    on("change-db", "click", |_| spawn_local(change_db()))?;
    on("about", "click", |_| spawn_local(about()))?;
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
