# htmx_four_actix

This crate is the Actix Web integration for [`htmx_four`](https://github.com/nilswloewen/htmx_four). It helps Rust users follow htmx the way the official docs describe it. Keep application code out of it: no OpenAPI crate, no session middleware, and no project-specific event names.

Official docs live at https://four.htmx.org. When you document, name, or change a type, cite the page for that header. Leave the link out when no such page exists. The htmx 4.0.1 source decides how a header value is parsed. Actix Web 4 decides the extractor and header-map types.

When you write or change Rust, follow the BeautifulRust [`rust-best-practices`](https://github.com/morganlinton/BeautifulRust/tree/main/skills/rust-best-practices) skill. Use [`rust-review`](https://github.com/morganlinton/BeautifulRust/tree/main/skills/rust-review) when asked to review. Keep the enums exhaustive, and keep Clippy at the default `cargo clippy --all-targets -- -D warnings` bar.

Depend on `htmx_four` with no features. Its header names are `&str`. Build `actix_web::http::header::HeaderName` with `HeaderName::from_static` on those strings. That is the `HeaderName` Actix Web 4 re-exports. Keep the `&str` constants for header values, including `Vary`.
