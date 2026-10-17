# Data Dash

The ACME Inc. dashboard. Rust compiled to WebAssembly, with a plain HTML page and a four-line JavaScript
loader. Charts are hand-built SVG. ACME Inc. and everyone in it is fictional.

Two tabs:

- **Business:** revenue (hero figure, by month, by category, top products), buyers, top customers, and
  tickets by status.
  - **Standalone:** the bundled `mock/` snapshot (3,000 orders, 400 tickets).
  - **Connected:** switch *Data* to *Live CRUD hub* (or open `?source=hub`) to read every row of
    `orders`, `products`, `customers` and `tickets` through the hub API, paged with
    `?limit=500&offset=N`. If the hub is down, the page says so and falls back to the snapshot.
- **Numbers & tracks:** the two fixture files in `data/`, charted as `data/README.md` describes, with a
  filterable, paged table view.

## Build and run

Needs Rust with the `wasm32-unknown-unknown` target and `wasm-pack` (`cargo install wasm-pack`).

```bash
wasm-pack build --target web --out-dir www/pkg --no-typescript
```

```bash
python -m http.server 8711 --bind 127.0.0.1 --directory www
```

Open `http://127.0.0.1:8711/` (`?tab=fixtures` opens the second tab; `?hub=http://host:port` points at
another hub).

## Tests

```bash
cargo test
```

Parsing, aggregation (money summed in whole cents), formatting, chart geometry and the HTML views live
in `src/core` and run on the host. The fixture tests check the counts stated in `data/README.md`.

## Charts

Colors are the validated reference palette's first three categorical slots (light and dark steps in
`www/style.css`); single-series charts use slot 1. Bars are ≤ 24px with a rounded data end, lines are
2px, markers carry a surface ring, every mark has a hover tooltip, multi-series charts have a legend,
and the numbers also appear as text (tiles, tables, value labels).

## Dependencies

| Crate | Why |
|---|---|
| `serde`, `serde_json` | Parse hub rows and the mock snapshot |
| `wasm-bindgen`, `wasm-bindgen-futures`, `js-sys`, `web-sys` | The standard Rust ↔ browser bindings: DOM, events, `fetch`, promises |

No chart or UI library.

## License

AGPL-3.0, as the rest of the portfolio. `data/numbers.dat` is Wikidata content, CC0 1.0.
