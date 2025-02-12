use crate::ffi::generated::cute_driver_generated::*;
use cute_core::CuteError;
use serde::{Deserialize, Serialize};

macro_rules! generate_cute_driver_task_output {
    ($name:ident, $inner:ty) => {
        pub struct $name {
            inner: cute_driver_result,
        }

        impl From<cute_driver_result> for $name {
            #[inline]
            #[must_use]
            fn from(inner: cute_driver_result) -> Self {
                Self { inner }
            }
        }

        impl $name {
            #[inline]
            #[must_use]
            pub fn get_inner(&self) -> Result<&$inner, CuteError> {
                unsafe {
                    match self.inner.code {
                        cute_error_code_CUTE_INTERNAL_ERROR | cute_error_code_CUTE_DRIVER_ERROR => {
                            let mut message = "???????".to_string();
                            let raw_bytes =
                                &self.inner.result.stack_data[..self.inner.len as usize];

                            if let Ok(c_string) = std::ffi::CStr::from_bytes_with_nul(raw_bytes) {
                                if let Ok(str_value) = c_string.to_str() {
                                    message = str_value.to_string()
                                }
                            } else {
                                message = String::from_utf8_lossy(raw_bytes).to_string();
                            }
                            Err(CuteError::internal(message))
                        }
                        cute_error_code_CUTE_HEAP_OK => {
                            if self.inner.result.heap_data == std::ptr::null_mut() {
                                Err(CuteError::internal("result null ptr"))
                            } else {
                                // 예시
                                let ptr = self.inner.result.heap_data;
                                let output = &*ptr.cast::<$inner>();
                                Ok(output)
                            }
                        }
                        cute_error_code_CUTE_STACK_OK => {
                            let ptr = self.inner.result.stack_data.as_ptr();
                            let output = &*ptr.cast::<$inner>();
                            Ok(output)
                        }
                        _ => Err(CuteError::internal("Output is Empty!!!".to_string())),
                    }
                }
            }
        }
    };
}

generate_cute_driver_task_output!(EmbeddedEchoOutput, cute_echo_output);

impl EmbeddedEchoOutput {
    pub fn get_count(&self) -> Result<i32, CuteError> {
        let inner = self.get_inner()?;
        unsafe { Ok(inner.count) }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EchoOutput {
    pub count: i32,
}
impl TryFrom<EmbeddedEchoOutput> for EchoOutput {
    type Error = CuteError;

    fn try_from(value: EmbeddedEchoOutput) -> Result<Self, Self::Error> {
        let count = value.get_count()?;

        Ok(Self { count })
    }
}
