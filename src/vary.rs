use actix_web::http::header::{HeaderMap, HeaderValue, VARY};
use htmx_four::request_headers::{HX_REQUEST, HX_REQUEST_TYPE};
use std::fmt;

/// [`vary_on_htmx`] could not read an existing `Vary` value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VaryError;

impl fmt::Display for VaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("vary value is not visible ascii")
    }
}

impl std::error::Error for VaryError {}

/// Records that the response varies on [`HX-Request`] and [`HX-Request-Type`].
///
/// The same URL can answer with a full page or a fragment, so a shared cache has to
/// include these request headers in its key.
///
/// Both names go in one `Vary` value. actix-cors reads the first `Vary` value and replaces
/// the header with that value plus the CORS request headers, which drops a later `Vary` line.
/// Existing lines are folded into the first value before the htmx names are added. A `*`
/// entry stays `*`.
///
/// [`HX-Request`]: https://four.htmx.org/reference/headers/HX-Request
/// [`HX-Request-Type`]: https://four.htmx.org/reference/headers/HX-Request-Type
///
/// # Errors
///
/// Returns [`VaryError`] when an existing `Vary` value is not visible ASCII. The map is unchanged.
pub fn vary_on_htmx(headers: &mut HeaderMap) -> Result<(), VaryError> {
    let value = combined_vary(headers)?;
    headers.insert(VARY, value);
    Ok(())
}

fn combined_vary(headers: &HeaderMap) -> Result<HeaderValue, VaryError> {
    let mut names = Vec::new();
    let mut star = false;

    for value in headers.get_all(VARY) {
        let text = value.to_str().map_err(|_| VaryError)?;
        for token in text.split(',') {
            let token = token.trim();
            if token.is_empty() {
                continue;
            }
            if token == "*" {
                star = true;
            }
            if !names
                .iter()
                .any(|name: &String| name.eq_ignore_ascii_case(token))
            {
                names.push(token.to_owned());
            }
        }
    }

    if star {
        return Ok(HeaderValue::from_static("*"));
    }

    for name in [HX_REQUEST, HX_REQUEST_TYPE] {
        if !names
            .iter()
            .any(|existing| existing.eq_ignore_ascii_case(name))
        {
            names.push(name.to_owned());
        }
    }

    HeaderValue::from_str(&names.join(", ")).map_err(|_| VaryError)
}
