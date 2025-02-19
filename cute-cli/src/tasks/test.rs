use std::sync::Arc;
use serde::{Deserialize, Serialize};
use cute_core::{bin_serialize, CuteError, Task};
use cute_embadded::*;
use crate::context::TestContext;

pub struct EchoTask;

#[derive(Serialize, Deserialize,Debug, Clone)]
pub struct EchoData {
    data : i32
}

#[derive(Serialize, Deserialize,Debug, Clone)]
pub struct TestData {
    empty_data : Vec<u8>,
    pub data : i32
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

pub struct TestTask {
    test : Vec<i32>
}

#[async_trait::async_trait]
impl Task<TestContext> for TestTask {

    fn new(_input: Option<Box<[u8]>>) -> Result<Box<dyn Task<TestContext> + Send>, CuteError>
    where
        Self: Sized
    {
        Ok(Box::new(Self {
            test: Vec::new()
        }))
    }

    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<TestContext>>) -> Result<Option<Vec<u8>>, CuteError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        let reader = ctx.read().await;
        let echo = TestData { data : reader.test * 2, empty_data : vec![0;100_000] };
        drop(reader);

        Ok(Some(bin_serialize(echo)?))
    }

    async fn destroy(&mut self) {
    }
}
