#![doc = include_str!("../README.md")]

mod hx_request;
mod hx_trigger;
mod vary;

pub use hx_request::{HxRequest, HxRequestType};
pub use hx_trigger::{
    HxTriggerError, HxTriggerErrorKind, WithHxTrigger, WithHxTriggerError, append_hx_trigger,
    trigger_without_swap,
};
pub use vary::{VaryError, vary_on_htmx};

const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<HxRequest>();
    assert_send_sync::<HxTriggerError>();
    assert_send_sync::<VaryError>();
};
