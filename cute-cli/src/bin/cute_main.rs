use std::sync::Arc;
use tokio::sync::RwLock;
use cute_core::Task;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let ctx = Arc::new(tokio::sync::RwLock::new(TestContext::default()));
    let mut task = EchoTask::new(None)?;
    let mut count = 0;
    loop {
        if count > 10 {
            break;
        }
        let output = task.execute(ctx.clone()).await?;
        println!("output : {:?}", output);
        count += 1;
    }
    task.destroy().await;
    println!("string count : {}", Arc::strong_count(&ctx));
    Ok(())
}

#[derive(Clone,Copy,Debug,Default)]
struct TestContext {
    test : i32,
}

struct EchoTask;

#[derive(serde::Serialize,serde::Deserialize,Debug,Copy, Clone)]
struct EchoData {
    data : i32
}

#[async_trait::async_trait]
impl Task<TestContext> for EchoTask {
    type Output = EchoData;

    fn new(input: Option<Box<[u8]>>) -> Result<Box<Self>, std::io::Error>
    where
        Self: Sized
    {
        Ok(Box::new(Self {}))
    }

    async fn execute(&mut self, ctx: Arc<RwLock<TestContext>>) -> Result<Self::Output, std::io::Error> {
        let mut writer = ctx.write().await;
        writer.test += 1;
        drop(writer);

        let reader = ctx.read().await;
        let echo = EchoData { data : reader.test };
        drop(reader);

        println!("string count : {}", Arc::strong_count(&ctx));
        Ok(echo)
    }

    async fn destroy(&mut self) {
    }
}