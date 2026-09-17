# pointiv-extension-sdk

Rust SDK for [Pointiv](https://pointiv.katkode.com) WASM extensions.

Previously published as `pointiv-extension-api`. Use `pointiv-extension-sdk` for new projects.

## Setup

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
pointiv-extension-sdk = "0.4"
extism-pdk = "1"
```

```rust
use pointiv_extension_sdk::prelude::*;

#[plugin_fn]
pub fn execute(Json(input): Json<Input>) -> FnResult<Json<Output>> {
    Ok(Json(Output::text(format!("Hello, {}!", input.text))))
}
```

Build:

```sh
cargo build --release --target wasm32-wasip1
```

## Input and output

`Input` has `text`, `context` (same as `text`), and `command` (what the user typed in the popup).

`Output` helpers: `text`, `copy`, `type_text`, `error`.

## APIs

Add permissions in `pointiv-extension.json`. Without a permission, calls fail safely (empty string, status 403, or an error JSON field).

| Module | Permission | What it does |
|--------|------------|--------------|
| `storage::` | `storage` | Per-extension key/value store |
| `clipboard::` | `clipboard_read` | Read clipboard |
| `ai::` | `ai` | LLM completion |
| `http::` | `network` | Outbound HTTP (you supply auth headers) |
| `google_calendar::` | `google_calendar` | Create Calendar events (Pointiv injects JWT) |
| `google_gmail::` | `google_gmail` | Send Gmail (Pointiv injects JWT) |
| `log::` | none | Log to `~/.pointiv/trace.jsonl` |

### Storage

```rust
storage::write("key", "value");
let v = storage::read("key");
storage::write_json("key", &my_struct);
let s: Option<MyStruct> = storage::read_json("key");
```

### HTTP

```rust
let resp = http::get("https://api.example.com/data");
let resp = http::post("https://api.example.com", r#"{"x":1}"#);

let resp = http::request(HttpRequest {
    method: "GET".into(),
    url: "https://api.example.com".into(),
    headers: [("Authorization".into(), "Bearer token".into())].into(),
    body: String::new(),
});
```

### Google Calendar

Connect Google in Pointiv Settings first.

```rust
google_calendar::schedule(
    "Team standup",
    "2026-06-01",
    Some("09:00"),
    Some("09:30"),
    Some("Daily sync"),
)?;
```

### Gmail

```rust
google_gmail::send("you@example.com", "Subject", "Body text")?;
```

### Logging

```rust
log::info("started");
log::warn("slow response");
log::error("failed");
```

## Tiles

Extensions can render a declarative tile widget beside the popup command bar. Declare a `"tiles"` block in `pointiv-extension.json` (requires `"runtime": "wasm"`), then export `render_tile`:

```rust
use pointiv_extension_sdk::prelude::*;

#[plugin_fn]
pub fn render_tile(Json(_input): Json<TileRenderInput>) -> FnResult<Json<TileUi>> {
    let tile = TileUi::new("Todos")
        .badge("2 open", TileTone::Warn)
        .row(RowBuilder::new("Buy milk").action("Done", "todo done 1"))
        .footer("Refresh", "todo list");
    Ok(Json(tile))
}
```

```json
"tiles": { "height": 2, "zone": "right", "order": 1 }
```

The host calls `render_tile` when the popup opens and after a tile action runs, with a 3 second budget and storage-only host access. Action commands run through your normal `execute` function. Iterate with the playground in Pointiv Settings, Tiles: paste tile JSON for instant validation and preview, or live-render an installed extension's tile. Full schema, limits, and the component catalog are in TILES.md in the Pointiv repo.

## Manifest

`pointiv-extension.json` at the repo root:

```json
{
  "name": "My Extension",
  "description": "What it does",
  "version": "1.0.0",
  "author": "your-name",
  "keywords": ["tag"],
  "runtime": "wasm",
  "main": "extension.wasm",
  "permissions": ["storage", "network", "google_calendar", "google_gmail"]
}
```

## Migration from pointiv-extension-api

```toml
[dependencies]
pointiv-extension-sdk = "0.4"
```

```rust
use pointiv_extension_sdk::prelude::*;
```

## License

MIT
