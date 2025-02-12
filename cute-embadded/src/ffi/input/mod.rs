use serde::{Deserialize, Serialize};
use crate::ffi::generated;

pub use generated::cute_driver_generated::cute_echo_input as EmbeddedEchoInput;

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoInput {
    not_used : i32,
}

impl From<EchoInput> for generated::cute_driver_generated::cute_echo_input {
    fn from(value: EchoInput) -> Self {
        Self {
            not_used: value.not_used,
        }
    }
}

