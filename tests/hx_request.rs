use actix_web::FromRequest;
use actix_web::test::TestRequest;
use htmx_four::request_headers::{HX_REQUEST, HX_REQUEST_TYPE};
use htmx_four_actix::{HxRequest, HxRequestType};

#[test]
fn classifies_the_request_headers() {
    let cases = [
        (None, None, HxRequest::Browser),
        (Some("true"), None, HxRequest::Htmx(HxRequestType::Absent)),
        (
            Some("true"),
            Some("partial"),
            HxRequest::Htmx(HxRequestType::Partial),
        ),
        (
            Some("true"),
            Some("PARTIAL"),
            HxRequest::Htmx(HxRequestType::Partial),
        ),
        (
            Some("true"),
            Some("full"),
            HxRequest::Htmx(HxRequestType::Full),
        ),
        (
            Some("true"),
            Some("Full"),
            HxRequest::Htmx(HxRequestType::Full),
        ),
        (
            Some("true"),
            Some("page"),
            HxRequest::Htmx(HxRequestType::Other),
        ),
        (
            Some("true"),
            Some(" partial "),
            HxRequest::Htmx(HxRequestType::Other),
        ),
        (
            Some("false"),
            Some("full"),
            HxRequest::Htmx(HxRequestType::Full),
        ),
        (None, Some("full"), HxRequest::Browser),
    ];

    for (hx_request, hx_request_type, want) in cases {
        let mut request = TestRequest::default();
        if let Some(value) = hx_request {
            request = request.insert_header((HX_REQUEST, value));
        }
        if let Some(value) = hx_request_type {
            request = request.insert_header((HX_REQUEST_TYPE, value));
        }

        let http = request.to_http_request();
        let got = HxRequest::from_headers(http.headers());
        assert_eq!(
            got, want,
            "hx-request={hx_request:?} hx-request-type={hx_request_type:?}"
        );
        assert_eq!(got.is_htmx(), !matches!(want, HxRequest::Browser));
        assert_eq!(
            got.request_type(),
            match want {
                HxRequest::Browser => None,
                HxRequest::Htmx(request_type) => Some(request_type),
            }
        );
        let fragment = matches!(want, HxRequest::Htmx(kind) if kind != HxRequestType::Full);
        assert_eq!(got.expects_fragment(), fragment);
    }
}

#[test]
fn header_names_are_case_insensitive() {
    let request = TestRequest::default()
        .insert_header(("HX-Request", "true"))
        .insert_header(("HX-Request-Type", "full"))
        .to_http_request();

    assert_eq!(
        HxRequest::from_headers(request.headers()),
        HxRequest::Htmx(HxRequestType::Full)
    );
}

#[test]
fn extractor_matches_from_headers() {
    let request = TestRequest::default()
        .insert_header((HX_REQUEST, "true"))
        .insert_header((HX_REQUEST_TYPE, "partial"))
        .to_http_request();

    let extracted = HxRequest::extract(&request).into_inner().unwrap();
    assert_eq!(extracted, HxRequest::from_headers(request.headers()));
    assert!(extracted.expects_fragment());
}
