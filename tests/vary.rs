use actix_web::http::header::{HeaderMap, HeaderValue, VARY};
use htmx_four_actix::{VaryError, vary_on_htmx};

fn vary(headers: &HeaderMap) -> &str {
    headers.get(VARY).unwrap().to_str().unwrap()
}

#[test]
fn sets_both_htmx_names_when_vary_is_absent() {
    let mut headers = HeaderMap::new();
    vary_on_htmx(&mut headers).unwrap();
    assert_eq!(vary(&headers), "hx-request, hx-request-type");
}

#[test]
fn folds_existing_lines_into_one_value() {
    let mut headers = HeaderMap::new();
    headers.append(VARY, HeaderValue::from_static("accept"));
    headers.append(VARY, HeaderValue::from_static("accept-language"));

    vary_on_htmx(&mut headers).unwrap();

    assert_eq!(
        vary(&headers),
        "accept, accept-language, hx-request, hx-request-type"
    );
    assert_eq!(headers.get_all(VARY).count(), 1);
}

#[test]
fn does_not_duplicate_names_it_already_has() {
    let mut headers = HeaderMap::new();
    headers.insert(
        VARY,
        HeaderValue::from_static("HX-Request, HX-Request-Type"),
    );

    vary_on_htmx(&mut headers).unwrap();

    assert_eq!(vary(&headers), "HX-Request, HX-Request-Type");
}

#[test]
fn leaves_a_star_in_place() {
    let mut headers = HeaderMap::new();
    headers.insert(VARY, HeaderValue::from_static("accept, *"));

    vary_on_htmx(&mut headers).unwrap();

    assert_eq!(vary(&headers), "*");
}

#[test]
fn leaves_a_non_ascii_value_unchanged() {
    let mut headers = HeaderMap::new();
    headers.insert(VARY, HeaderValue::from_bytes(b"accept\xfa").unwrap());

    let error = vary_on_htmx(&mut headers).unwrap_err();

    assert_eq!(error, VaryError);
    assert_eq!(error.to_string(), "vary value is not visible ascii");
    assert_eq!(headers.get(VARY).unwrap().as_bytes(), b"accept\xfa");
}
