#![allow(
    warnings,
    unused,
    unused_qualifications,
    non_camel_case_types,
    unreachable_pub,
    nonstandard_style,
    trivial_casts,
    future_incompatible
)]

pub(crate) mod cute_driver_generated;

unsafe impl Send for cute_driver_generated::cute_driver_result {}