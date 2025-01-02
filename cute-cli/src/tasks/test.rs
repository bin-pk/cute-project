use std::sync::Arc;
use serde::{Deserialize, Serialize};
use cute_core::{bin_serialize, CuteError, Task};
use crate::context::TestContext;

pub struct EchoTask;

#[derive(Serialize, Deserialize,Debug,Copy, Clone)]
pub struct EchoData {
    data : i32
}

#[derive(Serialize, Deserialize,Debug,Copy, Clone)]
pub struct TestData {
    data : i32
}


#[async_trait::async_trait]
impl Task<TestContext> for EchoTask {
    fn new(_input: Option<Box<[u8]>>) -> Result<Box<dyn Task<TestContext> + Send>, CuteError>
    where
        Self: Sized
    {
        Ok(Box::new(Self {}))
    }

    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<TestContext>>) -> Result<Option<Vec<u8>>, CuteError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

        let mut writer = ctx.write().await;
        writer.test += 1;
        drop(writer);

        let reader = ctx.read().await;
        let echo = EchoData { data : reader.test };
        drop(reader);

        Ok(Some(bin_serialize(echo)?))
    }

    async fn destroy(&mut self) {
    }
}

pub struct TestTask;

#[async_trait::async_trait]
impl Task<TestContext> for TestTask {

    fn new(_input: Option<Box<[u8]>>) -> Result<Box<dyn Task<TestContext> + Send>, CuteError>
    where
        Self: Sized
    {
        Ok(Box::new(Self {}))
    }

    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<TestContext>>) -> Result<Option<Vec<u8>>, CuteError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let reader = ctx.read().await;
        let echo = TestData { data : reader.test * 2 };
        drop(reader);

        Ok(Some(bin_serialize(echo)?))
    }

    async fn destroy(&mut self) {
    }
}

/*
#[derive(Debug, Clone, Default)]
pub struct EchoTaskConstructor;

#[async_trait::async_trait]
impl TaskConstructor<TestContext> for EchoTaskConstructor {
    fn create(&self, input: Option<Box<[u8]>>) -> Result<Box<dyn Task<TestContext> + Send>, CuteError> {
        EchoTask::new(input)
    }
}
*/