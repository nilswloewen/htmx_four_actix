use actix_web::HttpResponse;
use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
use htmx_four::{HxEvent, HxTrigger, SwapStyle};
use serde_json::Value;
use std::fmt;

const HX_TRIGGER: HeaderName = HeaderName::from_static(htmx_four::response_headers::HX_TRIGGER);
const HX_RESWAP: HeaderName = HeaderName::from_static(htmx_four::response_headers::HX_RESWAP);

/// [`append_hx_trigger`] could not extend [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HxTriggerError {
    kind: HxTriggerErrorKind,
}

impl HxTriggerError {
    /// Which part of the write failed.
    #[must_use]
    pub fn kind(self) -> HxTriggerErrorKind {
        self.kind
    }

    fn new(kind: HxTriggerErrorKind) -> Self {
        Self { kind }
    }
}

/// Why an [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger) write failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HxTriggerErrorKind {
    /// `event` is empty, has surrounding whitespace, contains a comma, starts with `{`, or is not a header value.
    EventName,
    /// An existing value, or the value this crate would write, is not valid header text.
    HeaderValue,
    /// An existing value starts with `{` and is not a JSON object.
    Json,
}

impl fmt::Display for HxTriggerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self.kind {
            HxTriggerErrorKind::EventName => "invalid hx-trigger event name",
            HxTriggerErrorKind::HeaderValue => "invalid hx-trigger header value",
            HxTriggerErrorKind::Json => "hx-trigger json value is not an object",
        })
    }
}

impl std::error::Error for HxTriggerError {}

/// [`WithHxTrigger::with_hx_trigger`] failed. The response is still here.
///
/// `HttpResponse` is not `Send`: Actix keeps extensions in a `RefCell`. This error is not `Send` either.
#[must_use = "dropping this error drops the response"]
pub struct WithHxTriggerError<B = actix_web::body::BoxBody> {
    response: HttpResponse<B>,
    error: HxTriggerError,
}

impl<B> fmt::Debug for WithHxTriggerError<B>
where
    B: actix_web::body::MessageBody,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WithHxTriggerError")
            .field("error", &self.error)
            .field("response", &self.response)
            .finish()
    }
}

impl<B> WithHxTriggerError<B> {
    /// The header error.
    pub fn error(&self) -> HxTriggerError {
        self.error
    }

    /// The response passed in, unchanged.
    pub fn into_response(self) -> HttpResponse<B> {
        self.response
    }
}

impl<B> fmt::Display for WithHxTriggerError<B> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.error.fmt(formatter)
    }
}

impl<B> std::error::Error for WithHxTriggerError<B>
where
    B: actix_web::body::MessageBody,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

/// Adds one event to [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger) on an [`HttpResponse`].
pub trait WithHxTrigger<B> {
    /// Writes `event` into `HX-Trigger` and returns the response.
    ///
    /// # Errors
    ///
    /// Returns the original response inside [`WithHxTriggerError`] when [`append_hx_trigger`] fails.
    #[must_use = "the updated response is returned"]
    fn with_hx_trigger(self, event: &str) -> Result<Self, WithHxTriggerError<B>>
    where
        Self: Sized;
}

impl<B> WithHxTrigger<B> for HttpResponse<B> {
    fn with_hx_trigger(mut self, event: &str) -> Result<Self, WithHxTriggerError<B>> {
        match append_hx_trigger(self.headers_mut(), event) {
            Ok(()) => Ok(self),
            Err(error) => Err(WithHxTriggerError {
                response: self,
                error,
            }),
        }
    }
}

/// Adds `event` to [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger).
///
/// htmx 4.0.0 parses a value that starts with `{` as a JSON object and any other value as a
/// comma-separated list of event names. The Fetch API joins repeated header lines with a comma
/// before htmx reads them, so a JSON value plus a second line is neither form. This function
/// reads every current line, joins them the same way, and writes one line back.
///
/// A name list gains `, event` unless that name is already there. A JSON object gains
/// `"event": true` unless that key is already there. A rewritten JSON object keeps the existing
/// keys in sorted order and appends the new name. When the name is already present, a single
/// existing line is left as it was. [`append_hx_trigger_detail`] attaches a JSON detail instead
/// of `true`, and one detail makes the whole header a JSON object.
///
/// An event name is one header token: non-empty, no surrounding whitespace, no comma, and it
/// does not start with `{`.
///
/// # Errors
///
/// Returns [`HxTriggerErrorKind::EventName`] when `event` is not one token.
/// Returns [`HxTriggerErrorKind::HeaderValue`] when an existing line is not header text.
/// Returns [`HxTriggerErrorKind::Json`] when a value starts with `{` and is not a JSON object.
/// On error the header map is unchanged.
pub fn append_hx_trigger(headers: &mut HeaderMap, event: &str) -> Result<(), HxTriggerError> {
    validate_event_name(event)?;
    let event =
        HxEvent::name(event).map_err(|_| HxTriggerError::new(HxTriggerErrorKind::EventName))?;
    append_event(headers, event)
}

/// Adds `event` with a JSON [`detail`](https://four.htmx.org/reference/headers/HX-Trigger#event-detail).
///
/// A string detail stays a JSON string. htmx exposes it as `event.detail.value`. An object is
/// copied onto the event detail. A name list becomes one JSON object, and each bare name in that
/// object is `true`. An existing event keeps its first detail.
///
/// # Errors
///
/// Returns the same errors as [`append_hx_trigger`]. On error the header map is unchanged.
pub fn append_hx_trigger_detail(
    headers: &mut HeaderMap,
    event: &str,
    detail: &Value,
) -> Result<(), HxTriggerError> {
    validate_event_name(event)?;
    let event = HxEvent::with_detail(event, detail)
        .map_err(|_| HxTriggerError::new(HxTriggerErrorKind::EventName))?;
    append_event(headers, event)
}

/// Adds `event` and a JSON `detail` to [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger) on `response`.
///
/// # Errors
///
/// Returns the original response inside [`WithHxTriggerError`] when [`append_hx_trigger_detail`] fails.
#[must_use = "the updated response is returned"]
pub fn with_hx_trigger_detail<B>(
    mut response: HttpResponse<B>,
    event: &str,
    detail: &Value,
) -> Result<HttpResponse<B>, WithHxTriggerError<B>> {
    match append_hx_trigger_detail(response.headers_mut(), event, detail) {
        Ok(()) => Ok(response),
        Err(error) => Err(WithHxTriggerError { response, error }),
    }
}

/// `200 OK` with [`HX-Reswap: none`](https://four.htmx.org/reference/headers/HX-Reswap) and one [`HX-Trigger`](https://four.htmx.org/reference/headers/HX-Trigger) event.
///
/// htmx does not insert the empty body. The event is the signal other elements use to fetch
/// fresh HTML (`hx-trigger="<event> from:body"`).
///
/// `HX-Trigger` is not applied to a 3xx response, so this is a 200.
///
/// # Errors
///
/// Returns [`HxTriggerError`] when `event` is not one token. See [`append_hx_trigger`].
pub fn trigger_without_swap(event: &str) -> Result<HttpResponse, HxTriggerError> {
    let mut response = HttpResponse::Ok()
        .insert_header((&HX_RESWAP, SwapStyle::None.as_str()))
        .finish();
    append_hx_trigger(response.headers_mut(), event)?;
    Ok(response)
}

fn validate_event_name(event: &str) -> Result<(), HxTriggerError> {
    let valid = !event.is_empty()
        && !event.starts_with(char::is_whitespace)
        && !event.ends_with(char::is_whitespace)
        && !event.contains(',')
        && !event.starts_with('{')
        && HeaderValue::from_str(event).is_ok();
    if valid {
        Ok(())
    } else {
        Err(HxTriggerError::new(HxTriggerErrorKind::EventName))
    }
}

struct Existing {
    value: String,
    count: usize,
}

fn combined_existing(
    headers: &HeaderMap,
    name: &HeaderName,
) -> Result<Option<Existing>, HxTriggerError> {
    let mut values = headers.get_all(name);
    let Some(first) = values.next() else {
        return Ok(None);
    };

    let mut value = header_str(first)?.to_owned();
    let mut count = 1;
    for next in values {
        value.push_str(", ");
        value.push_str(header_str(next)?);
        count += 1;
    }
    Ok(Some(Existing { value, count }))
}

fn append_event(headers: &mut HeaderMap, event: HxEvent) -> Result<(), HxTriggerError> {
    let Some(existing) = combined_existing(headers, &HX_TRIGGER)? else {
        write_trigger(headers, &singleton(event))?;
        return Ok(());
    };

    let mut trigger = HxTrigger::parse(&existing.value)
        .map_err(|_| HxTriggerError::new(HxTriggerErrorKind::Json))?;
    let added = trigger.insert(event);
    if !added && existing.count <= 1 {
        return Ok(());
    }
    write_trigger(headers, &trigger)
}

fn singleton(event: HxEvent) -> HxTrigger {
    let mut trigger = HxTrigger::new();
    let added = trigger.insert(event);
    debug_assert!(added);
    trigger
}

fn write_trigger(headers: &mut HeaderMap, trigger: &HxTrigger) -> Result<(), HxTriggerError> {
    let text = trigger
        .to_header_value()
        .map_err(|_| HxTriggerError::new(HxTriggerErrorKind::HeaderValue))?;
    headers.insert(HX_TRIGGER.clone(), header_text(&text)?);
    Ok(())
}

fn header_str(value: &HeaderValue) -> Result<&str, HxTriggerError> {
    value
        .to_str()
        .map_err(|_| HxTriggerError::new(HxTriggerErrorKind::HeaderValue))
}

fn header_text(value: &str) -> Result<HeaderValue, HxTriggerError> {
    HeaderValue::from_str(value).map_err(|_| HxTriggerError::new(HxTriggerErrorKind::HeaderValue))
}
