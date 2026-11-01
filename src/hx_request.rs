use actix_web::dev::Payload;
use actix_web::http::header::{HeaderMap, HeaderName};
use actix_web::{FromRequest, HttpRequest};
use htmx_four::HxRequestInfo;
use std::convert::Infallible;
use std::future::{Ready, ready};

const HX_REQUEST: HeaderName = HeaderName::from_static(htmx_four::request_headers::HX_REQUEST);
const HX_REQUEST_TYPE: HeaderName =
    HeaderName::from_static(htmx_four::request_headers::HX_REQUEST_TYPE);

/// What [`HX-Request`] and [`HX-Request-Type`] say about an Actix request.
///
/// Implement [`FromRequest`], or call [`from_headers`](Self::from_headers) on a header map
/// you already hold.
///
/// [`HX-Request`]: https://four.htmx.org/reference/headers/HX-Request
/// [`HX-Request-Type`]: https://four.htmx.org/reference/headers/HX-Request-Type
///
/// # Examples
///
/// ```
/// use actix_web::test::TestRequest;
/// use htmx_four::request_headers::{HX_REQUEST, HX_REQUEST_TYPE};
/// use htmx_four_actix::{HxRequest, HxRequestType};
///
/// let request = TestRequest::default()
///     .insert_header((HX_REQUEST, "true"))
///     .insert_header((HX_REQUEST_TYPE, "partial"))
///     .to_http_request();
///
/// let hx = HxRequest::from_headers(request.headers());
/// assert_eq!(hx, HxRequest::Htmx(HxRequestType::Partial));
/// assert!(hx.expects_fragment());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HxRequest {
    /// No [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) header.
    ///
    /// A normal browser navigation. [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type)
    /// on its own does not make the request an htmx request.
    Browser,
    /// [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) is present.
    Htmx(HxRequestType),
}

/// The [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type) on an htmx request.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HxRequestType {
    /// `partial`. The target is an element other than the document body.
    Partial,
    /// `full`. The target is the body, or the element uses `hx-select`.
    ///
    /// The response is the whole page, the same shape as a browser navigation.
    Full,
    /// `HX-Request` was sent without `HX-Request-Type`.
    ///
    /// htmx 4.0.1 sends the type header on every request. A missing type still
    /// asks for a fragment, which is the check on the
    /// [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) page.
    Absent,
    /// A value other than `partial` or `full`.
    ///
    /// Read [`HX_REQUEST_TYPE`](htmx_four::request_headers::HX_REQUEST_TYPE) for the bytes.
    /// This still asks for a fragment.
    Other,
}

impl HxRequest {
    /// Classifies `headers`.
    ///
    /// `HX-Request` matches by presence. htmx 4.0.1 sends `true`, and any other
    /// value still counts. `partial` and `full` are ASCII case-insensitive.
    /// Surrounding whitespace is not stripped.
    #[must_use]
    pub fn from_headers(headers: &HeaderMap) -> Self {
        if !headers.contains_key(&HX_REQUEST) {
            return Self::Browser;
        }

        let request_type = match headers.get(&HX_REQUEST_TYPE) {
            None => HxRequestType::Absent,
            Some(value) if eq_ignore_ascii_case(value.as_bytes(), b"partial") => {
                HxRequestType::Partial
            }
            Some(value) if eq_ignore_ascii_case(value.as_bytes(), b"full") => HxRequestType::Full,
            Some(_) => HxRequestType::Other,
        };
        Self::Htmx(request_type)
    }

    /// [`HX-Request`](https://four.htmx.org/reference/headers/HX-Request) is present.
    #[must_use]
    pub fn is_htmx(self) -> bool {
        matches!(self, Self::Htmx(_))
    }

    /// The [`HX-Request-Type`](https://four.htmx.org/reference/headers/HX-Request-Type), when this is an htmx request.
    #[must_use]
    pub fn request_type(self) -> Option<HxRequestType> {
        match self {
            Self::Browser => None,
            Self::Htmx(request_type) => Some(request_type),
        }
    }

    /// [`HX-Boosted`](https://four.htmx.org/reference/headers/HX-Boosted),
    /// [`HX-Current-URL`](https://four.htmx.org/reference/headers/HX-Current-URL),
    /// [`HX-History-Restore-Request`](https://four.htmx.org/reference/headers/HX-History-Restore-Request),
    /// [`HX-Source`](https://four.htmx.org/reference/headers/HX-Source), and
    /// [`HX-Target`](https://four.htmx.org/reference/headers/HX-Target).
    ///
    /// A flag is true when that header is present and UTF-8. A non-UTF-8 value does not count.
    /// This does not change [`from_headers`](Self::from_headers).
    #[must_use]
    pub fn info(headers: &HeaderMap) -> HxRequestInfo<'_> {
        HxRequestInfo::lookup(|name| headers.get(name).and_then(|value| value.to_str().ok()))
    }

    /// The handler should return a fragment.
    ///
    /// True when this is an htmx request and the type is not [`HxRequestType::Full`].
    /// A browser navigation and `HX-Request-Type: full` are both false: both want the whole page.
    #[must_use]
    pub fn expects_fragment(self) -> bool {
        self.is_htmx() && self.request_type() != Some(HxRequestType::Full)
    }
}

impl FromRequest for HxRequest {
    type Error = Infallible;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        ready(Ok(Self::from_headers(req.headers())))
    }
}

fn eq_ignore_ascii_case(value: &[u8], expected: &[u8]) -> bool {
    value.len() == expected.len()
        && value
            .iter()
            .zip(expected)
            .all(|(got, want)| got.eq_ignore_ascii_case(want))
}
