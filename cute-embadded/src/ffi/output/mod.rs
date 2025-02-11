use serde::{Serialize, Deserialize};
use crate::ffi::generated::cute_driver_generated::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct RustEchoOutput {
    count: i32,
}

impl From<cute_echo_output> for RustEchoOutput {
    fn from(value : cute_echo_output) -> Self {
        unsafe {
            Self {
                count: value.count,
            }
        }
    }
}