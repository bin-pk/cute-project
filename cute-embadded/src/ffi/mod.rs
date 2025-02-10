mod generated;

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

/*
        match value.code {
            generated::cute_driver_generated::cute_error_code_CUTE_EMPTY=> {
               Self {
                   code: EmbeddedResultCode::ResultEmpty,
                   data: None,
                   err_msg : None,
               }
            }
            generated::cute_driver_generated::cute_error_code_CUTE_STACK_OK=> {
                unsafe {
                    Self {
                        code: EmbeddedResultCode::ResultStackOK,
                        data: Some(value.result.stack_data[..value.len as usize].to_vec()),
                        err_msg : None,
                    }
                }
            }
            generated::cute_driver_generated::cute_error_code_CUTE_HEAP_OK=> {
                unsafe {
                    if !value.result.heap_data.is_null() {
                        let slice = std::slice::from_raw_parts(
                            value.result.heap_data as *const u8,
                            value.len as usize,
                        );

                        Self {
                            code: EmbeddedResultCode::ResultHeapOK,
                            data: Some(slice.to_vec()),
                            err_msg: None,
                        }


                    } else {
                        Self {
                            code: EmbeddedResultCode::ResultHeapOK,
                            data: None,
                            err_msg: None,
                        }
                    }
                }
            }
            generated::cute_driver_generated::cute_error_code_CUTE_DRIVER_ERROR=> {
                unsafe {
                    let mut message = None;
                    let raw_bytes = &value.result.stack_data[..value.len as usize];

                    if let Ok(c_string) = std::ffi::CStr::from_bytes_with_nul(raw_bytes) {
                        if let Ok(str_value) = c_string.to_str() {
                            message = Some(str_value.to_string());
                        } else {
                            message = Some("???????".to_string());
                        }
                    } else {
                        message = Some(String::from_utf8_lossy(raw_bytes).to_string());
                    }

                    Self {
                        code: EmbeddedResultCode::ResultDriverError,
                        data: None,
                        err_msg: message,
                    }
                }
            }
            generated::cute_driver_generated::cute_error_code_CUTE_INTERNAL_ERROR=> {
                unsafe {
                    let mut message = None;
                    let raw_bytes = &value.result.stack_data[..value.len as usize];

                    if let Ok(c_string) = std::ffi::CStr::from_bytes_with_nul(raw_bytes) {
                        if let Ok(str_value) = c_string.to_str() {
                            message = Some(str_value.to_string());
                        } else {
                            message = Some("???????".to_string());
                        }
                    } else {
                        message = Some(String::from_utf8_lossy(raw_bytes).to_string());
                    }

                    Self {
                        code: EmbeddedResultCode::ResultInternalError,
                        data: None,
                        err_msg: message,
                    }
                }
            }
        }
 */

/*
impl EmbeddedResultWrapper {
    pub fn destroy(&mut self, protocol : u32) {
        unsafe {
            generated::cute_driver_generated::destroy_driver_task(protocol, &mut self.inner);
        }
    }
}
*/