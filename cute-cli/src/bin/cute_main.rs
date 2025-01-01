use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio_stream::StreamExt;
use cute_core::{bin_deserialize, BasicTaskConstructor, BasicWorker, CuteError, ProcManager, Task, Worker};

#[tokio::main]
async fn main() -> Result<(), CuteError> {

    tokio::spawn(async move {
        let mut proc_map = ProcManager::new();

        proc_map.insert("echo".into(), Box::new(BasicWorker::new(BasicTaskConstructor::<EchoTask,TestContext>::new())));
        match cute_network::Server::create_grpc(cute_network::NetworkConfig::default()).start_server(Box::new(proc_map)).await {
            Ok(_) => {}
            Err(_) => {}
        }
    });

    let mut client = cute_network::Client::<TestContext>::create_grpc(cute_network::NetworkConfig {
        max_page_byte_size: 65536,
        max_channel_size: 128,
        request_limit_milli_second : 262_144,
        host_address:  std::net::SocketAddr::from(([127,0,0,1], 7777)),
        time_out: 30,
        keep_alive_time_out: 60,
    }).await.unwrap();

    match client.get_stream("echo".into(),None).await {
        Ok(mut stream) => {
            while let Some(res_output) = stream.next().await {
                if let Ok(output) = res_output {
                    let convert_result = bin_deserialize::<EchoData>(&*output);
                    println!("{:?}", convert_result);
                }
            }
        }
        Err(_) => {}
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

    fn new(_input: Option<Box<[u8]>>) -> Result<Box<Self>, CuteError>
    where
        Self: Sized
    {
        Ok(Box::new(Self {}))
    }

    async fn execute(&mut self, ctx: Arc<tokio::sync::RwLock<TestContext>>) -> Result<Self::Output, CuteError> {
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

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