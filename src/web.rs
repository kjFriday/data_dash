// ==== Generic Company Inc. portfolio ====================================
// Project: data_dash | File: src/web.rs
// About: Page wiring.
// Copyright (c) 2026 kjFriday | SPDX-License-Identifier: AGPL-3.0-only
// You may change this file under the AGPL-3.0. The stamp at the end marks
// the version kjFriday wrote. Verify with the portfolio's tools/stamp.py --check.
// Reused code, if any, is mapped in PROVENANCE.md.
// ========================================================================

//! Page wiring. The Business view reads the shared database, served at the same origin by the desktop
//! program (`/api/...`), or the embedded mock snapshot when there is none (or in a plain browser);
//! the Fixtures view always reads data/numbers.dat and data/tracks.dat, compiled in.
//!
//! Compiled only for wasm32 (see `lib.rs`). www/main.js imports the wasm-pack output and calls
//! `init()`, which runs `start`. Flow:
//! 1. `start` parses the two fixture files, renders the Numbers & tracks tab and its table, wires the
//!    DOM events, and honours `?tab=fixtures` and `?source=mock`.
//! 2. `load_db` (async) asks `GET /api/source` which database the desktop program serves. With one,
//!    `fetch_all` reads `orders`, `products`, `customers` and `tickets` in pages of `PAGE` rows, and
//!    `render::business` draws the Business tab from them.
//! 3. With no database, in a plain browser, or on any error, `use_mock` draws the Business tab from
//!    the mock snapshot compiled into the wasm.
//! 4. Database... (`change_db`) POSTs a new path to `/host/db`; on success the page reloads the data.
//!
//! Progress and errors go to the footer status (`status`). main.rs's `--smoke` check waits for that
//! text to contain "Shared database" or "mock snapshot", so keep those words in the finished-load
//! messages. Every element id used here must exist in www/index.html; `el` panics on a missing one.

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

/// The mock snapshot (orders, products, customers, tickets), compiled into the wasm so the Business
/// tab always has something to show.
const MOCK: [&str; 4] = [
    include_str!("../mock/orders.json"),
    include_str!("../mock/products.json"),
    include_str!("../mock/customers.json"),
    include_str!("../mock/tickets.json"),
];
/// data/numbers.dat, compiled in.
const NUMBERS: &str = include_str!("../data/numbers.dat");
/// data/tracks.dat, compiled in.
const TRACKS: &str = include_str!("../data/tracks.dat");
/// Prefix for `/api` URLs; empty means the page's own origin.
const API_ORIGIN: &str = ""; // same origin: the desktop program answers /api/... from the shared database
/// Rows per `/api` page; must equal the host's largest accepted `limit` (host/api.rs answers 400 above 500).
const PAGE: usize = 500; // the largest ?limit
/// `#source`'s option for the shared database in index.html (its value dates from when the CRUD hub served /api).
const DB_OPTION: &str = "hub";

/// The number table's state, shared by the event handlers.
struct App {
    /// The parsed numbers.dat rows.
    numbers: Vec<NumberRow>,
    /// The class filter as typed.
    class: String,
    /// 0-based page, as `render::number_table` last clamped it.
    page: usize,
}

thread_local! {
    /// The one `App`, set by `start`. A page's wasm runs on one thread, so a thread-local `RefCell` is
    /// enough to share it with the event closures (which must be `'static` and cannot borrow it).
    static APP: RefCell<Option<App>> = const { RefCell::new(None) };
}

/// Runs `f` on the `App` with a mutable borrow and returns its result.
///
/// Panics before `start` has set the state, or if called again from inside `f` (a `RefCell`
/// double borrow).
fn with<R>(f: impl FnOnce(&mut App) -> R) -> R {
    APP.with(|a| f(a.borrow_mut().as_mut().expect("app started")))
}

/// Starts the page; wasm-bindgen runs it once when main.js calls `init()`.
///
/// Steps:
/// 1. Parse the fixtures and render the Numbers & tracks charts (an `Err` here is a bad fixture file,
///    and main.js shows it as "Could not start").
/// 2. Store the table state, draw the table, and show the version in the footer.
/// 3. Wire the DOM events.
/// 4. `?tab=fixtures` opens the second tab; `?source=mock` draws the mock snapshot. Otherwise the data
///    menu is set to the shared database and `load_db` runs in the background.
#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let numbers = numbers::parse(NUMBERS).map_err(|e| JsValue::from_str(&e))?;
    let samples = tracks::parse(TRACKS).map_err(|e| JsValue::from_str(&e))?;
    el("fixture-charts").set_inner_html(&render::fixtures(&numbers, &samples));
    APP.with(|a| {
        *a.borrow_mut() = Some(App {
            numbers,
            class: String::new(),
            page: 0,
        })
    });
    draw_table();
    el("version").set_text_content(Some(concat!("Generic Company Data Dash ", env!("CARGO_PKG_VERSION"))));
    wire()?;
    if query_param("tab").as_deref() == Some("fixtures") {
        show_tab("fixtures");
    }
    if query_param("source").as_deref() == Some("mock") {
        use_mock("Mock snapshot");
    } else {
        select("source").set_value(DB_OPTION);
        spawn_local(load_db());
    }
    Ok(())
}

/// Draws the Business tab from the compiled-in mock snapshot; `why` starts the status line.
///
/// Also sets the data menu to "Mock snapshot" and the mode badge to "mock".
fn use_mock(why: &str) {
    select("source").set_value("mock");
    match Data::from_json(MOCK[0], MOCK[1], MOCK[2], MOCK[3]) {
        Ok(d) => {
            el("business").set_inner_html(&render::business(&d, "Mock snapshot bundled with this app"));
            set_mode(false);
            status(
                &format!("{why}: {} orders, {} tickets.", d.orders.len(), d.tickets.len()),
                false,
            );
        }
        Err(e) => status(&format!("Mock snapshot unreadable: {e}"), true),
    }
}

/// Reads all four tables from the shared database and draws the Business tab, else the mock snapshot.
///
/// Steps:
/// 1. `source_db`: no database (or no desktop program) -> the mock snapshot, and stop.
/// 2. Read each table with `fetch_all`, showing "Reading TABLE ..." as it goes, and parse the four
///    arrays with `Data::from_json`.
/// 3. Success: render, set the badge to "shared database" and report the counts. Any error: the mock
///    snapshot, then an error status that names the cause (it replaces `use_mock`'s line).
///
/// Known limitation: nothing cancels a load in progress, so pressing Reload twice runs two loads and
/// the one that finishes last wins.
async fn load_db() {
    let origin = API_ORIGIN.to_string();
    let Some(db) = source_db().await else {
        use_mock("No shared database here, so the mock snapshot");
        return;
    };
    status("Opening the shared database ...", false);
    // an inner async block so `?` can stop at the first failing table and land in `loaded`
    let loaded = async {
        let mut parts = Vec::new();
        for table in ["orders", "products", "customers", "tickets"] {
            status(&format!("Reading {table} ..."), false);
            // back to one JSON string per table, the shape `Data::from_json` shares with the mock files
            parts.push(Value::Array(fetch_all(&origin, table).await?).to_string());
        }
        Data::from_json(&parts[0], &parts[1], &parts[2], &parts[3])
    }
    .await;
    match loaded {
        Ok(d) => {
            el("business").set_inner_html(&render::business(&d, &format!("Shared database {db}")));
            set_mode(true);
            status(
                &format!(
                    "Shared database {db}: {} orders, {} tickets, read with GET /api/{{table}}?limit={PAGE}&offset=N.",
                    d.orders.len(),
                    d.tickets.len()
                ),
                false,
            );
        }
        Err(err) => {
            use_mock("Database unavailable");
            status(
                &format!("Could not read the shared database ({err}). Showing the mock snapshot."),
                true,
            );
        }
    }
}

/// The database the desktop program serves, or None (no database, or a plain browser).
///
/// `/api/source` answers the host's state as JSON; its "db" field is the path, or null in mock mode.
/// Any failure (no such URL, not JSON) counts as None.
async fn source_db() -> Option<String> {
    let text = get_text("/api/source").await.ok()?;
    serde_json::from_str::<Value>(&text).ok()?["db"]
        .as_str()
        .map(String::from)
}

/// GETs `url` with `fetch` -> the response body as text.
///
/// Errors: "STATUS on URL" for any status but 200, or the browser's message when the request fails.
async fn get_text(url: &str) -> Result<String, String> {
    let init = RequestInit::new();
    init.set_method("GET");
    let req = Request::new_with_str_and_init(url, &init).map_err(js_err)?;
    let win = web_sys::window().ok_or("no window")?;
    let resp: Response = JsFuture::from(win.fetch_with_request(&req))
        .await
        .map_err(js_err)?
        .dyn_into()
        .map_err(js_err)?;
    if resp.status() != 200 {
        return Err(format!("{} on {url}", resp.status()));
    }
    Ok(JsFuture::from(resp.text().map_err(js_err)?)
        .await
        .map_err(js_err)?
        .as_string()
        .unwrap_or_default())
}

/// Ask for another database file; the desktop program checks it and saves the choice.
///
/// Steps:
/// 1. `prompt()` for a path; Cancel or a blank answer does nothing.
/// 2. POST `{"path": ...}` to `/host/db`. When the request itself fails, the status says the desktop
///    program is needed.
/// 3. 200: the host has switched databases, so `load_db` reads it. Any other status: the host's
///    `{"error": ...}` text (or the raw body, as from a plain web server) goes to the status as
///    "Not used: ...", and the current data stays.
async fn change_db() {
    let win = web_sys::window().unwrap();
    let Ok(Some(path)) = win.prompt_with_message("Path of the shared Generic Company database (generic_company.db):")
    else {
        return;
    };
    if path.trim().is_empty() {
        return;
    }
    let init = RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(
        &serde_json::json!({ "path": path.trim() }).to_string(),
    ));
    let Ok(req) = Request::new_with_str_and_init("/host/db", &init) else {
        return;
    };
    let resp = match JsFuture::from(win.fetch_with_request(&req))
        .await
        .ok()
        .and_then(|r| r.dyn_into::<Response>().ok())
    {
        Some(r) => r,
        None => return status("Choosing a database needs the desktop program.", true),
    };
    if resp.status() == 200 {
        spawn_local(load_db());
    } else {
        let text = match resp.text() {
            Ok(p) => JsFuture::from(p)
                .await
                .ok()
                .and_then(|v| v.as_string())
                .unwrap_or_default(),
            Err(_) => String::new(),
        };
        let msg = serde_json::from_str::<Value>(&text)
            .ok()
            .and_then(|v| v["error"].as_str().map(String::from))
            .unwrap_or(text);
        status(&format!("Not used: {msg}"), true);
    }
}

/// Shows the About box: version, database in use, icon credit and licence, as a plain `alert()`.
///
/// The icon line is the attribution the icon's CC BY 2.5 licence asks for.
async fn about() {
    let db = source_db().await.unwrap_or_else(|| "none (mock snapshot)".into());
    let text = format!(
        "Generic Company Data Dash {}\nGeneric Company Inc. (fictional)\n\nDatabase: {db}\n\nIcon: Silk \"chart_bar\" by Mark James (famfamfam.com/lab/icons/silk), CC BY 2.5, enlarged by nearest-neighbour.\nCode: AGPL-3.0. All data is fictional.",
        env!("CARGO_PKG_VERSION")
    );
    let _ = web_sys::window().unwrap().alert_with_message(&text);
}

/// Every row of a shared-database table, paging with ?limit/&offset.
///
/// Each request asks for `PAGE` rows starting after the rows already read; a page shorter than
/// `PAGE` is the last one. Rows are JSON objects, kept as `Value`s. Errors: "STATUS on TABLE" for a
/// non-200 answer (503 when the host has no database), the browser's message for a failed request,
/// or a JSON error.
///
/// Known limitation: the host lists rows newest first, so a row added while the pages are read
/// shifts the offsets and one row can be read twice. A table whose size is an exact multiple of
/// `PAGE` costs one extra, empty request.
async fn fetch_all(origin: &str, table: &str) -> Result<Vec<Value>, String> {
    let mut rows = Vec::new();
    loop {
        let url = format!("{origin}/api/{table}?limit={PAGE}&offset={}", rows.len());
        let init = RequestInit::new();
        init.set_method("GET");
        let req = Request::new_with_str_and_init(&url, &init).map_err(js_err)?;
        let win = web_sys::window().ok_or("no window")?;
        let resp: Response = JsFuture::from(win.fetch_with_request(&req))
            .await
            .map_err(js_err)?
            .dyn_into()
            .map_err(js_err)?;
        let text = JsFuture::from(resp.text().map_err(js_err)?)
            .await
            .map_err(js_err)?
            .as_string()
            .unwrap_or_default();
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

/// A JavaScript error value -> its message, for the status line.
///
/// A thrown `Error` gives its `message`; a thrown string is used as is; anything else reads
/// "request failed".
fn js_err(e: JsValue) -> String {
    if let Some(err) = e.dyn_ref::<js_sys::Error>() {
        return err.message().into();
    }
    e.as_string().unwrap_or_else(|| "request failed".into())
}

// ---- DOM ----

/// The element with `id`; panics with "#id missing" if index.html lacks it.
fn el(id: &str) -> Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .get_element_by_id(id)
        .unwrap_or_else(|| panic!("#{id} missing"))
}

/// The `<input>` with `id`; panics if it is missing or another kind of element.
fn input(id: &str) -> HtmlInputElement {
    el(id).dyn_into().unwrap()
}

/// The `<select>` with `id`; panics if it is missing or another kind of element.
fn select(id: &str) -> HtmlSelectElement {
    el(id).dyn_into().unwrap()
}

/// Writes `msg` to the footer status; `error` adds the `error` class (style.css colors it).
fn status(msg: &str, error: bool) {
    let s = el("status");
    s.set_text_content(Some(msg));
    s.set_class_name(if error { "error" } else { "" });
}

/// Sets the header badge: "shared database" (with the `live` class) or "mock".
fn set_mode(live: bool) {
    let m = el("mode");
    m.set_text_content(Some(if live { "shared database" } else { "mock" }));
    m.set_class_name(if live { "badge live" } else { "badge" });
}

/// The first value of `key` in the page's query string, decoded; None when absent.
///
/// Decodes as a form would: `+` is a space, then percent escapes (`decodeURIComponent`). A key with
/// no `=` (as in `?tab`) does not match, and a value that does not decode is None.
fn query_param(key: &str) -> Option<String> {
    let search = web_sys::window()?.location().search().ok()?;
    search.trim_start_matches('?').split('&').find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key)
            .then(|| {
                js_sys::decode_uri_component(&v.replace('+', " "))
                    .ok()
                    .map(String::from)
            })
            .flatten()
    })
}

/// Shows the tab panel `name` ("business" or "fixtures") and hides the other.
///
/// Each panel's `hidden` attribute and its tab button's `aria-selected` change together, so screen
/// readers and the selected-tab style follow.
fn show_tab(name: &str) {
    for tab in ["business", "fixtures"] {
        let on = tab == name;
        let _ = el(tab).toggle_attribute_with_force("hidden", !on);
        let _ = el(&format!("tab-{tab}")).set_attribute("aria-selected", if on { "true" } else { "false" });
    }
}

/// Redraws the number table from the `App` state and stores the page it actually showed.
fn draw_table() {
    let html = with(|a| {
        let (html, page) = render::number_table(&a.numbers, &a.class, a.page);
        a.page = page;
        html
    });
    el("number-table").set_inner_html(&html);
}

/// Adds `f` as the `kind` event listener of element `id`.
///
/// The Rust closure becomes a JavaScript function through `Closure`; `forget` hands its ownership to
/// JavaScript (a deliberate, one-time leak), because dropping it would invalidate the function and the
/// listener would throw when the event fires.
fn on(id: &str, kind: &str, f: impl FnMut(Event) + 'static) -> Result<(), JsValue> {
    let cb = Closure::<dyn FnMut(Event)>::new(f);
    el(id).add_event_listener_with_callback(kind, cb.as_ref().unchecked_ref())?;
    cb.forget(); // listeners live as long as the page
    Ok(())
}

/// Wires every control in index.html to its handler.
///
/// - The data menu and Reload both reload from the chosen source.
/// - Database... runs `change_db`; About runs `about`.
/// - The tab buttons switch tabs.
/// - Typing in the class filter goes back to page 0 and redraws the table.
/// - One click listener on the table container serves the pager buttons, which are replaced on every
///   redraw (event delegation: the click bubbles up from the button).
fn wire() -> Result<(), JsValue> {
    // a capture-free closure is `Copy`, so the same `reload` serves two listeners
    let reload = |_| {
        if select("source").value() == DB_OPTION {
            spawn_local(load_db());
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

// ==== stamp v1 | author=kjFriday | hmac-sha256=6ac77fd5213ab48ea0754e4958054c5dcdcad1bda5c74f62f40a80533872c5df ====
