use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
use htmx_four::response_headers::{
    HX_LOCATION, HX_PUSH_URL, HX_REDIRECT, HX_REFRESH, HX_REPLACE_URL, HX_RESELECT, HX_RESWAP,
    HX_RETARGET,
};
use htmx_four::{HxLocation, HxUrlHistory, SwapStyle};
use htmx_four_actix::{
    HxHeaderError, set_hx_location, set_hx_push_url, set_hx_redirect, set_hx_refresh,
    set_hx_replace_url, set_hx_reselect, set_hx_reswap, set_hx_retarget,
};

fn text(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned)
}

#[test]
fn location_writes_a_path_or_json() {
    let mut headers = HeaderMap::new();
    let path = HxLocation::builder().path("/orders").build();
    set_hx_location(&mut headers, &path).unwrap();
    assert_eq!(text(&headers, HX_LOCATION).as_deref(), Some("/orders"));

    let mut located = HxLocation::builder()
        .path("/search")
        .target("#results")
        .build();
    located.set_swap_style(SwapStyle::OuterHtml);
    set_hx_location(&mut headers, &located).unwrap();

    let value: serde_json::Value =
        serde_json::from_str(&text(&headers, HX_LOCATION).unwrap()).unwrap();
    assert_eq!(value["path"], "/search");
    assert_eq!(value["target"], "#results");
    assert_eq!(value["swap"], "outerHTML");
    assert_eq!(
        headers
            .get_all(HeaderName::from_static(HX_LOCATION))
            .count(),
        1
    );
}

#[test]
fn an_empty_location_path_leaves_the_map_unchanged() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static(HX_LOCATION),
        HeaderValue::from_static("/stay"),
    );

    let error = set_hx_location(&mut headers, &HxLocation::builder().path("").build()).unwrap_err();

    assert_eq!(error, HxHeaderError::EmptyPath);
    assert_eq!(error.to_string(), "hx-location path is empty");
    assert_eq!(text(&headers, HX_LOCATION).as_deref(), Some("/stay"));
}

#[test]
fn redirect_push_replace_refresh_and_selectors_replace_one_line() {
    let mut headers = HeaderMap::new();
    set_hx_redirect(&mut headers, "/login").unwrap();
    set_hx_push_url(&mut headers, &HxUrlHistory::Skip).unwrap();
    set_hx_replace_url(&mut headers, &HxUrlHistory::Url(String::from("/orders/7"))).unwrap();
    set_hx_refresh(&mut headers);
    set_hx_reswap(&mut headers, SwapStyle::None);
    set_hx_retarget(&mut headers, "#panel").unwrap();
    set_hx_reselect(&mut headers, "#row").unwrap();

    assert_eq!(text(&headers, HX_REDIRECT).as_deref(), Some("/login"));
    assert_eq!(text(&headers, HX_PUSH_URL).as_deref(), Some("false"));
    assert_eq!(text(&headers, HX_REPLACE_URL).as_deref(), Some("/orders/7"));
    assert_eq!(text(&headers, HX_REFRESH).as_deref(), Some("true"));
    assert_eq!(text(&headers, HX_RESWAP).as_deref(), Some("none"));
    assert_eq!(text(&headers, HX_RETARGET).as_deref(), Some("#panel"));
    assert_eq!(text(&headers, HX_RESELECT).as_deref(), Some("#row"));

    set_hx_redirect(&mut headers, "/login").unwrap();
    assert_eq!(
        headers
            .get_all(HeaderName::from_static(HX_REDIRECT))
            .count(),
        1
    );
}

#[test]
fn a_newline_in_a_url_leaves_the_map_unchanged() {
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static(HX_REDIRECT),
        HeaderValue::from_static("/stay"),
    );

    let error = set_hx_redirect(&mut headers, "/login\n/next").unwrap_err();

    assert_eq!(error, HxHeaderError::Value);
    assert_eq!(text(&headers, HX_REDIRECT).as_deref(), Some("/stay"));
}
