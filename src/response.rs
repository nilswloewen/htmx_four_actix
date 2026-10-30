//! Writes one htmx 4 response header, replacing any earlier value of that name.
//!
//! A failed write leaves the header map unchanged.

use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
use htmx_four::response_headers::{
    HX_LOCATION, HX_PUSH_URL, HX_REDIRECT, HX_REFRESH, HX_REPLACE_URL, HX_RESELECT, HX_RESWAP,
    HX_RETARGET,
};
use htmx_four::{HxLocation, HxLocationError, HxUrlHistory, SwapStyle};
use std::fmt;

const LOCATION: HeaderName = HeaderName::from_static(HX_LOCATION);
const PUSH_URL: HeaderName = HeaderName::from_static(HX_PUSH_URL);
const REDIRECT: HeaderName = HeaderName::from_static(HX_REDIRECT);
const REFRESH: HeaderName = HeaderName::from_static(HX_REFRESH);
const REPLACE_URL: HeaderName = HeaderName::from_static(HX_REPLACE_URL);
const RESELECT: HeaderName = HeaderName::from_static(HX_RESELECT);
const RESWAP: HeaderName = HeaderName::from_static(HX_RESWAP);
const RETARGET: HeaderName = HeaderName::from_static(HX_RETARGET);

/// [`set_hx_redirect`], [`set_hx_location`], or another response-header write failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HxHeaderError {
    /// The text is not a valid header value.
    Value,
    /// [`HxLocation::path`] is empty.
    EmptyPath,
    /// [`HxLocation`] could not be written as JSON.
    Json,
}

impl fmt::Display for HxHeaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Value => "invalid htmx header value",
            Self::EmptyPath => "hx-location path is empty",
            Self::Json => "hx-location json failed",
        })
    }
}

impl std::error::Error for HxHeaderError {}

impl From<HxLocationError> for HxHeaderError {
    fn from(error: HxLocationError) -> Self {
        match error {
            HxLocationError::EmptyPath => Self::EmptyPath,
            HxLocationError::Json(_) => Self::Json,
        }
    }
}

/// Sets [`HX-Redirect`](https://four.htmx.org/reference/headers/HX-Redirect) to `url`.
///
/// htmx sets `location.href` to this value. It is not an HTTP redirect.
///
/// # Errors
///
/// Returns [`HxHeaderError::Value`] when `url` is not a valid header value. The map is unchanged.
pub fn set_hx_redirect(headers: &mut HeaderMap, url: &str) -> Result<(), HxHeaderError> {
    insert_text(headers, &REDIRECT, url)
}

/// Sets [`HX-Location`](https://four.htmx.org/reference/headers/HX-Location) from `location`.
///
/// A path alone is written as that path. Any other field is written as one JSON object of
/// [`htmx.ajax()`](https://four.htmx.org/reference/methods/htmx-ajax#options) options.
///
/// # Errors
///
/// Returns [`HxHeaderError::EmptyPath`] when the path is empty.
/// Returns [`HxHeaderError::Json`] when the JSON object cannot be written.
/// Returns [`HxHeaderError::Value`] when the text is not a valid header value.
/// The map is unchanged.
pub fn set_hx_location(
    headers: &mut HeaderMap,
    location: &HxLocation,
) -> Result<(), HxHeaderError> {
    insert_text(headers, &LOCATION, &location.to_header_value()?)
}

/// Sets [`HX-Push-Url`](https://four.htmx.org/reference/headers/HX-Push-Url).
///
/// [`HxUrlHistory::Skip`] writes `false`. [`HxUrlHistory::Url`] writes that URL.
///
/// # Errors
///
/// Returns [`HxHeaderError::Value`] when the URL is not a valid header value. The map is unchanged.
pub fn set_hx_push_url(headers: &mut HeaderMap, url: &HxUrlHistory) -> Result<(), HxHeaderError> {
    insert_text(headers, &PUSH_URL, url.as_str())
}

/// Sets [`HX-Replace-Url`](https://four.htmx.org/reference/headers/HX-Replace-Url).
///
/// [`HxUrlHistory::Skip`] writes `false` and leaves the history entry alone.
/// [`HxUrlHistory::Url`] writes that URL. The text `true` replaces the entry with the response path.
///
/// # Errors
///
/// Returns [`HxHeaderError::Value`] when the URL is not a valid header value. The map is unchanged.
pub fn set_hx_replace_url(
    headers: &mut HeaderMap,
    url: &HxUrlHistory,
) -> Result<(), HxHeaderError> {
    insert_text(headers, &REPLACE_URL, url.as_str())
}

/// Sets [`HX-Refresh`](https://four.htmx.org/reference/headers/HX-Refresh) to `true`.
///
/// htmx reloads the page with `location.reload()`.
pub fn set_hx_refresh(headers: &mut HeaderMap) {
    headers.insert(REFRESH.clone(), HeaderValue::from_static("true"));
}

/// Sets [`HX-Reswap`](https://four.htmx.org/reference/headers/HX-Reswap) to `swap`.
///
/// This writes the style name only. A value with modifiers, such as `innerHTML transition:true`,
/// is the full [`hx-swap`](https://four.htmx.org/reference/attributes/hx-swap) string. Set that
/// string with [`HX_RESWAP`](htmx_four::response_headers::HX_RESWAP) directly.
pub fn set_hx_reswap(headers: &mut HeaderMap, swap: SwapStyle) {
    headers.insert(RESWAP.clone(), HeaderValue::from_static(swap.as_str()));
}

/// Sets [`HX-Retarget`](https://four.htmx.org/reference/headers/HX-Retarget) to `selector`.
///
/// # Errors
///
/// Returns [`HxHeaderError::Value`] when `selector` is not a valid header value. The map is unchanged.
pub fn set_hx_retarget(headers: &mut HeaderMap, selector: &str) -> Result<(), HxHeaderError> {
    insert_text(headers, &RETARGET, selector)
}

/// Sets [`HX-Reselect`](https://four.htmx.org/reference/headers/HX-Reselect) to `selector`.
///
/// # Errors
///
/// Returns [`HxHeaderError::Value`] when `selector` is not a valid header value. The map is unchanged.
pub fn set_hx_reselect(headers: &mut HeaderMap, selector: &str) -> Result<(), HxHeaderError> {
    insert_text(headers, &RESELECT, selector)
}

fn insert_text(
    headers: &mut HeaderMap,
    name: &HeaderName,
    value: &str,
) -> Result<(), HxHeaderError> {
    let value = HeaderValue::from_str(value).map_err(|_| HxHeaderError::Value)?;
    headers.insert(name.clone(), value);
    Ok(())
}
