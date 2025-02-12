use crate::ffi::generated::cute_driver_generated::*;
use crate::ffi::input::*;
use crate::ffi::output::*;
use crate::ffi::EmbeddedContext;
use cute_core::{bin_deserialize, bin_serialize, CuteError, Task};
use std::sync::Arc;
use tokio::sync::RwLock;

/*
#[macro_export]
macro_rules! generate_cute_task {
    ($name:ident,$context: ident,$id:expr, $input_driver:ident, $input : ty, $output_driver:ident,$output : ident) => {
        pub struct $name {
            inner: cute_driver_result,
        }

        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    if let Some(destroy_fn) = self.inner.destroy {
                        destroy_fn(&mut self.inner);
                    }
                }
            }
        }

        #[async_trait::async_trait]
        impl Task<$context> for $name {
            fn new(_input: Option<Box<[u8]>>) -> Result<Box<dyn Task<$context> + Send>, CuteError>
            where
                Self: Sized,
            {
                unsafe {
                    match _input {
                        None => Ok(Box::new(Self {
                            inner: create_driver_task($id, std::ptr::null_mut()),
                        })),
                        Some(parameter) => {
                            let input_param = bin_deserialize::<$input>(&parameter)?;
                            let driver_input = $input_driver::from(input_param);
                            let ptr = std::ptr::addr_of!(driver_input) as *mut std::os::raw::c_void;

                            Ok(Box::new(Self {
                                inner: create_driver_task($id, ptr),
                            }))
                        }
                    }
                }
            }

            async fn execute(
                &mut self,
                ctx: Arc<tokio::sync::RwLock<$context>>,
            ) -> Result<Option<Vec<u8>>, CuteError> {
                unsafe {
                    let mut res = execute_driver_task($id, &mut self.inner);

                    let driver_output = $output_driver::from(res);
                    let output_data = $output::try_from(driver_output)?;

                    if let Some(destroy_fn) = res.destroy {
                        destroy_fn(&mut res);
                    }

                    Ok(Some(bin_serialize(output_data)?))
                }
            }

            async fn destroy(&mut self) {
                unsafe {
                    if let Some(destroy_fn) = self.inner.destroy {
                        destroy_fn(&mut self.inner);
                    }
                }
            }
        }
    };
}

generate_cute_task!(EmbeddedEchoTask,
    EmbeddedContext,
    0,
    EmbeddedEchoInput,
    EchoInput,
    EmbeddedEchoOutput,
    EchoOutput);
*/

pub struct EmbeddedEchoTask {
    inner: Box<cute_driver_result>,
}
impl Drop for EmbeddedEchoTask {
    fn drop(&mut self) {
        unsafe {
            if let Some(destroy_fn) = self.inner.destroy {
                destroy_fn(self.inner.as_mut());
            }
        }
    }
}
#[async_trait::async_trait]
impl Task<EmbeddedContext> for EmbeddedEchoTask {
    fn new(_input: Option<Box<[u8]>>) -> Result<Box<dyn Task<EmbeddedContext> + Send>, CuteError>
    where
        Self: Sized,
    {
        unsafe {
            match _input {
                None => Ok(Box::new(Self {
                    inner: Box::new(create_driver_task(0, std::ptr::null_mut())),
                })),
                Some(parameter) => {
                    let input_param = bin_deserialize::<EchoInput>(&parameter)?;
                    let driver_input = EmbeddedEchoInput::from(input_param);
                    let ptr = &raw const driver_input as *mut std::os::raw::c_void;
                    Ok(Box::new(Self {
                        inner: Box::new(create_driver_task(0, ptr)),
                    }))
                }
            }
        }
    }
    async fn execute(
        &mut self,
        ctx: Arc<tokio::sync::RwLock<EmbeddedContext>>,
    ) -> Result<Option<Vec<u8>>, CuteError> {
        unsafe {
            let mut res = execute_driver_task(0, self.inner.as_mut());
            let driver_output = EmbeddedEchoOutput::from(res);
            let output_data = EchoOutput::try_from(driver_output)?;
            if let Some(destroy_fn) = res.destroy {
                destroy_fn(&mut res);
            }
            Ok(Some(bin_serialize(output_data)?))
        }
    }
    async fn destroy(&mut self) {
        unsafe {
            if let Some(destroy_fn) = self.inner.destroy {
                destroy_fn(self.inner.as_mut());
            }
        }
    }
}
