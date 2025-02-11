use std::ops::Deref;
use cute_core::{bin_serialize, CuteError};
use crate::ffi::generated::cute_driver_generated::cute_echo_output;
use crate::ffi::output::RustEchoOutput;

mod generated;
mod output;

pub enum EmbeddedResult {
    EmbeddedData(Option<Vec<u8>>),
    EmbeddedErr(Option<String>)
}

pub struct EmbeddedResultWrapper {
    inner : generated::cute_driver_generated::cute_driver_result
}


impl From<generated::cute_driver_generated::cute_driver_result> for EmbeddedResultWrapper {
    fn from(value: generated::cute_driver_generated::cute_driver_result) -> Self {
        Self { inner: value }
    }
}

impl Deref for EmbeddedResultWrapper {
    type Target = generated::cute_driver_generated::cute_driver_result;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

//해당 구조를 macro 로 변환하기.
struct TestTest {
    inner : generated::cute_driver_generated::cute_driver_result
}

impl Drop for TestTest {
    fn drop(&mut self) {
        unsafe {
            generated::cute_driver_generated::destroy_driver_task(Self::TASK_ID, &mut self.inner);
        }
    }
}

impl TestTest {
    const TASK_ID : u32 = 0;
    fn new(_input: Option<Box<[u8]>>) -> Self {
        unsafe {

            match _input {
                None => {
                    Self {
                        inner : generated::cute_driver_generated::create_driver_task(Self::TASK_ID, std::ptr::null_mut())
                    }
                }
                Some(parameter) => {
                    Self {
                        // rust-lang 의 struct 을 from trait 을 통해 c 로 변환시킨후 void pointer 로 넣어주기.
                        inner : generated::cute_driver_generated::create_driver_task(Self::TASK_ID, parameter.as_ptr() as *mut std::os::raw::c_void)
                    }
                }
            }
        }
    }

    async fn execute(&mut self) -> Result<Option<Vec<u8>>, CuteError> {
        unsafe {
            let value = generated::cute_driver_generated::execute_driver_task(Self::TASK_ID, &mut self.inner);

            match value.code {
                generated::cute_driver_generated::cute_error_code_CUTE_INTERNAL_ERROR |
                generated::cute_driver_generated::cute_error_code_CUTE_DRIVER_ERROR => {
                    let mut message = "???????".to_string();
                    let raw_bytes = &value.result.stack_data[..value.len as usize];

                    if let Ok(c_string) = std::ffi::CStr::from_bytes_with_nul(raw_bytes) {
                        if let Ok(str_value) = c_string.to_str() {
                            message = str_value.to_string()
                        }
                    } else {
                        message = String::from_utf8_lossy(raw_bytes).to_string();
                    }
                    Err(CuteError::internal(message))
                }
                generated::cute_driver_generated::cute_error_code_CUTE_STACK_OK => {
                    // rust-lang 의 struct 을 from trait 을 통해 c 의 struct 를 rust-lang 으로 변환시킨후 bincode serialize 수행.
                    // 예시
                    let output_ptr = value.result.stack_data.as_ptr() as *const cute_echo_output;
                    let output = RustEchoOutput::from(std::ptr::read(output_ptr));
                    Ok(Some(bin_serialize(output)?))
                }
                generated::cute_driver_generated::cute_error_code_CUTE_HEAP_OK => {
                    if value.result.heap_data == std::ptr::null_mut() {
                        Err(CuteError::internal("result null ptr"))
                    } else {
                        // rust-lang 의 struct 을 from trait 을 통해 c 의 struct 를 rust-lang 으로 변환시킨후 bincode serialize 수행.
                        // 예시
                        let output_ptr = value.result.heap_data.as_ptr() as *const cute_echo_output;
                        let output = RustEchoOutput::from(std::ptr::read(output_ptr));
                        Ok(Some(bin_serialize(output)?))
                    }
                }
                _ => {
                    Ok(None)
                }
            }

        }
    }
}