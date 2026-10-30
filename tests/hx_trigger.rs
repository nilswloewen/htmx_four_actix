use actix_web::HttpResponse;
use actix_web::http::StatusCode;
use actix_web::http::header::{HeaderMap, HeaderName, HeaderValue};
use htmx_four::response_headers::{HX_RESWAP, HX_TRIGGER};
use htmx_four_actix::{
    HxTriggerErrorKind, WithHxTrigger as _, append_hx_trigger, append_hx_trigger_detail,
    trigger_without_swap, with_hx_trigger_detail,
};

fn hx_trigger_name() -> HeaderName {
    HeaderName::from_static(HX_TRIGGER)
}

fn trigger(headers: &HeaderMap) -> &str {
    headers.get(hx_trigger_name()).unwrap().to_str().unwrap()
}

#[test]
fn appends_a_name_to_an_empty_header() {
    let mut headers = HeaderMap::new();
    append_hx_trigger(&mut headers, "saved").unwrap();
    assert_eq!(trigger(&headers), "saved");
}

#[test]
fn appends_a_second_name_once() {
    let mut headers = HeaderMap::new();
    append_hx_trigger(&mut headers, "saved").unwrap();
    append_hx_trigger(&mut headers, "refresh").unwrap();
    append_hx_trigger(&mut headers, "saved").unwrap();
    assert_eq!(trigger(&headers), "saved, refresh");
    assert_eq!(headers.get_all(hx_trigger_name()).count(), 1);
}

#[test]
fn joins_separate_name_lines_into_one() {
    let mut headers = HeaderMap::new();
    headers.append(hx_trigger_name(), HeaderValue::from_static("saved"));
    headers.append(hx_trigger_name(), HeaderValue::from_static("refresh"));

    append_hx_trigger(&mut headers, "closed").unwrap();

    assert_eq!(trigger(&headers), "saved, refresh, closed");
    assert_eq!(headers.get_all(hx_trigger_name()).count(), 1);
}

#[test]
fn merges_a_new_event_into_json() {
    let mut headers = HeaderMap::new();
    headers.insert(
        hx_trigger_name(),
        HeaderValue::from_static(r#"{"notification":{"level":"info"}}"#),
    );

    append_hx_trigger(&mut headers, "refresh").unwrap();

    let value: serde_json::Value = serde_json::from_str(trigger(&headers)).unwrap();
    assert_eq!(value["notification"]["level"], "info");
    assert_eq!(value["refresh"], true);
}

#[test]
fn leaves_an_existing_json_key_in_place() {
    let mut headers = HeaderMap::new();
    let original = r#"{"saved":{"id":7}}"#;
    headers.insert(hx_trigger_name(), HeaderValue::from_static(original));

    append_hx_trigger(&mut headers, "saved").unwrap();

    assert_eq!(trigger(&headers), original);
}

#[test]
fn rejects_a_json_value_it_cannot_merge() {
    let mut headers = HeaderMap::new();
    headers.insert(
        hx_trigger_name(),
        HeaderValue::from_static(r#"{"saved":true}, refresh"#),
    );

    let error = append_hx_trigger(&mut headers, "closed").unwrap_err();

    assert_eq!(error.kind(), HxTriggerErrorKind::Json);
    assert_eq!(error.to_string(), "hx-trigger json value is not an object");
    assert_eq!(trigger(&headers), r#"{"saved":true}, refresh"#);
}

#[test]
fn rejects_an_event_name_that_is_not_one_token() {
    let mut headers = HeaderMap::new();
    headers.insert(hx_trigger_name(), HeaderValue::from_static("saved"));

    for event in [
        "",
        " saved",
        "saved ",
        "saved,refresh",
        "{saved",
        "saved\nrefresh",
    ] {
        let error = append_hx_trigger(&mut headers, event).unwrap_err();
        assert_eq!(error.kind(), HxTriggerErrorKind::EventName, "{event:?}");
        assert_eq!(trigger(&headers), "saved");
    }
}

#[test]
fn with_hx_trigger_returns_the_response_on_failure() {
    let error = HttpResponse::Ok()
        .body("page")
        .with_hx_trigger("bad,name")
        .unwrap_err();

    assert_eq!(error.error().kind(), HxTriggerErrorKind::EventName);
    assert_eq!(error.to_string(), "invalid hx-trigger event name");
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get(hx_trigger_name()).is_none());
}

#[test]
fn trigger_without_swap_sets_both_headers() {
    let response = trigger_without_swap("saved").unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(hx_trigger_name())
            .unwrap()
            .to_str()
            .unwrap(),
        "saved"
    );
    assert_eq!(
        response.headers().get(HX_RESWAP).unwrap().to_str().unwrap(),
        "none"
    );
}

#[test]
fn trigger_without_swap_rejects_a_bad_event_name() {
    let error = trigger_without_swap("saved,refresh").unwrap_err();
    assert_eq!(error.kind(), HxTriggerErrorKind::EventName);
}

#[test]
fn a_detail_turns_a_name_list_into_json() {
    let mut headers = HeaderMap::new();
    append_hx_trigger(&mut headers, "refresh").unwrap();
    append_hx_trigger_detail(
        &mut headers,
        "notification",
        &serde_json::json!({"level": "info"}),
    )
    .unwrap();

    let value: serde_json::Value = serde_json::from_str(trigger(&headers)).unwrap();
    assert_eq!(value["refresh"], true);
    assert_eq!(value["notification"]["level"], "info");
}

#[test]
fn a_string_detail_stays_a_string() {
    let mut headers = HeaderMap::new();
    append_hx_trigger_detail(&mut headers, "notification", &serde_json::json!("Saved")).unwrap();

    assert_eq!(trigger(&headers), r#"{"notification":"Saved"}"#);
}

#[test]
fn with_hx_trigger_detail_returns_the_response_on_failure() {
    let error = with_hx_trigger_detail(
        HttpResponse::Ok().body("page"),
        "bad,name",
        &serde_json::json!(true),
    )
    .unwrap_err();

    assert_eq!(error.error().kind(), HxTriggerErrorKind::EventName);
    let response = error.into_response();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get(hx_trigger_name()).is_none());
}

#[test]
fn an_existing_detail_stays_byte_for_byte() {
    let mut headers = HeaderMap::new();
    let original = r#"{"saved":{"id":7}}"#;
    headers.insert(hx_trigger_name(), HeaderValue::from_static(original));

    append_hx_trigger_detail(&mut headers, "saved", &serde_json::json!({"id": 8})).unwrap();

    assert_eq!(trigger(&headers), original);
}
