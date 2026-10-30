# `htmx_four_actix`

Actix Web support for [htmx 4.0.0](https://github.com/bigskysoftware/htmx/tree/v4.0.0). Handlers read [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) and [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type). Responses write [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger), [`HX-Reswap`](https://four.htmx.org/reference/headers/HX-Reswap), and `Vary` in the form htmx 4 and Actix consume.

Header names and [`SwapStyle`](https://four.htmx.org/reference/attributes/hx-swap) come from [`htmx_four`](https://docs.rs/htmx_four). The names are `&str`. This crate turns them into [`actix_web::http::header::HeaderName`](https://docs.rs/actix-web/4/actix_web/http/header/struct.HeaderName.html), the `http` 0.2 type Actix Web 4 re-exports. A handler passes the `&str` straight through, as in `.insert_header((HX_REDIRECT, path))`. Where an API needs an owned `HeaderName`, use `HeaderName::from_static` on that string.

```toml
[dependencies]
htmx_four_actix = "0.1.1"
```

This crate follows `htmx_four`: htmx 4 and newer only. The `htmx-2-compat` feature turns on the same feature there, which exports the header names htmx 4 dropped.

```toml
htmx_four_actix = { version = "0.1.1", features = ["htmx-2-compat"] }
```

`HxRequest` still reads [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) and [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type). An htmx 2 request sends `HX-Request` and no `HX-Request-Type`, so it is `Htmx(Absent)` and `expects_fragment` is true. The feature does not also write `HX-Trigger-After-Swap` or `HX-Trigger-After-Settle`. htmx 2 would fire `HX-Trigger` and those headers as separate events.

## Example

A partial htmx request gets a fragment. A browser navigation and `HX-Request-Type: full` both get the whole page.

```rust
use actix_web::HttpResponse;
use htmx_four_actix::{HxRequest, HxTriggerError, trigger_without_swap};

async fn save(hx: HxRequest) -> Result<HttpResponse, HxTriggerError> {
    if hx.expects_fragment() {
        trigger_without_swap("saved")
    } else {
        Ok(HttpResponse::Ok().body("<html><body><p>Saved</p></body></html>"))
    }
}

# fn main() {
#     let _ = save(HxRequest::Browser);
# }
```

`trigger_without_swap` is a `200` with `HX-Reswap: none` and `HX-Trigger: saved`. htmx does not insert the empty body. Elements listening with `hx-trigger="saved from:body"` fetch fresh HTML.

To add that event to a response you already built:

```rust
use actix_web::HttpResponse;
use htmx_four::response_headers::HX_TRIGGER;
use htmx_four_actix::WithHxTrigger as _;

fn main() -> Result<(), htmx_four_actix::WithHxTriggerError> {
    let response = HttpResponse::Ok().body("<p>Saved</p>").with_hx_trigger("saved")?;
    let trigger = response.headers().get(HX_TRIGGER).and_then(|value| value.to_str().ok());
    assert_eq!(trigger, Some("saved"));
    Ok(())
}
```

## Request

`HxRequest` implements `FromRequest`, and `HxRequest::from_headers` reads a header map directly. `HX-Request` matches by presence. `partial` and `full` are ASCII case-insensitive.

| Headers | `HxRequest` | `expects_fragment` |
|---|---|---|
| `HX-Request` absent | `Browser` | no |
| `HX-Request`, no type | `Htmx(Absent)` | yes |
| `HX-Request-Type: partial` | `Htmx(Partial)` | yes |
| `HX-Request-Type: full` | `Htmx(Full)` | no |
| any other type | `Htmx(Other)` | yes |

`is_htmx` is the [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) check. `expects_fragment` is an htmx request that is not [`full`](https://four.htmx.org/reference/headers/HX-Request-Type).

## `HX-Trigger`

`append_hx_trigger` adds one event to [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger).

htmx 4.0.0 treats a value that starts with `{` as JSON and any other value as a comma-separated name list. The Fetch API joins duplicate header lines with a comma before htmx sees them, so a second `HX-Trigger` line breaks a JSON value. This function writes one line. A name list gains `, event`. A JSON object gains `"event": true`. A name or key that is already present is left in place.

An event name is one header token: non-empty, no comma, no surrounding whitespace, and it does not start with `{`.

## Vary

`vary_on_htmx` records that the representation depends on [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) and [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type).

actix-cors reads the first `Vary` value and replaces the header with that value plus the CORS request headers. A later `Vary` line is dropped. This function folds every `Vary` line into one value, then adds the two htmx names. A `*` entry stays `*`.

## Docs

- [Documentation](https://four.htmx.org/docs)
- [Headers](https://four.htmx.org/reference/headers)
- [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request)
- [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type)
- [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger)
- [`HX-Reswap`](https://four.htmx.org/reference/headers/HX-Reswap)
- [`hx-swap`](https://four.htmx.org/reference/attributes/hx-swap)

Built by htmx CEO [nilswloewen](https://htmx.ceo/) and his trusty clanka's.

![Official CEO of htmx](assets/nils_ceo.jpeg)
