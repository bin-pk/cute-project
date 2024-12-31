use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use cute_core::{bin_deserialize, BasicWorker, Task, Worker};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let ctx = Arc::new(tokio::sync::RwLock::new(TestContext::default()));
    let worker = BasicWorker::<TestContext,EchoTask>::new();
    let mut count = 1;
    let mut stream = worker.iter_execute(ctx, None).await?;

    while let Some(result) = stream.next().await {
        if count >= 10 {
            let _ = worker.iter_close().await;
        }
        match result {
            Ok(output) => {
                let convert_result = bin_deserialize::<EchoData>(&*output);
                println!("{:?}", convert_result);
            }
            Err(e) => {
                println!("Error: {:?}", e);
            }
        }
        count += 1;
    }

    Ok(())
}

#[derive(Clone,Copy,Debug,Default)]
pub struct TestContext {
    test : i32,
}

pub struct EchoTask;

#[derive(Serialize, Deserialize,Debug,Copy, Clone)]
pub struct EchoData {
    data : i32
}

#[async_trait::async_trait]
impl Task<TestContext> for EchoTask {
    type Output = EchoData;

    fn new(_input: Option<Box<[u8]>>) -> Result<Box<Self>, std::io::Error>
    where
        Self: Sized
    {
        Ok(Box::new(Self {}))
    }

    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<TestContext>>) -> Result<Self::Output, std::io::Error> {
        let mut writer = ctx.write().await;
        writer.test += 1;
        drop(writer);

        let reader = ctx.read().await;
        let echo = EchoData { data : reader.test };
        drop(reader);

        Ok(echo)
    }

    async fn destroy(&mut self) {
    }
}