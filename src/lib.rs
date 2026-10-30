#![doc = include_str!("../README.md")]

mod hx_request;
mod hx_trigger;
mod response;
mod vary;

pub use hx_request::{HxRequest, HxRequestType};
pub use hx_trigger::{
    HxTriggerError, HxTriggerErrorKind, WithHxTrigger, WithHxTriggerError, append_hx_trigger,
    append_hx_trigger_detail, trigger_without_swap, with_hx_trigger_detail,
};
pub use response::{
    HxHeaderError, set_hx_location, set_hx_push_url, set_hx_redirect, set_hx_refresh,
    set_hx_replace_url, set_hx_reselect, set_hx_reswap, set_hx_retarget,
};
pub use vary::{VaryError, vary_on_htmx};

const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<HxRequest>();
    assert_send_sync::<HxHeaderError>();
    assert_send_sync::<HxTriggerError>();
    assert_send_sync::<VaryError>();
};
