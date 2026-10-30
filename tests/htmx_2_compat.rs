#![cfg(feature = "htmx-2-compat")]

#[test]
fn htmx_2_header_names_come_from_htmx_four() {
    assert_eq!(htmx_four::request_headers::HX_TRIGGER, "hx-trigger");
    assert_eq!(
        htmx_four::request_headers::HX_TRIGGER_NAME,
        "hx-trigger-name"
    );
    assert_eq!(htmx_four::response_headers::HX_PUSH, "hx-push");
    assert_eq!(
        htmx_four::response_headers::HX_TRIGGER_AFTER_SETTLE,
        "hx-trigger-after-settle"
    );
    assert_eq!(
        htmx_four::response_headers::HX_TRIGGER_AFTER_SWAP,
        "hx-trigger-after-swap"
    );
}
