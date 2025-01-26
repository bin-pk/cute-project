use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use cute_cli::context::TestContext;
use cute_cli::tasks::*;
use cute_core::{bin_deserialize, CuteError, ProcManager};
use log::{info, warn};

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), CuteError> {
    cute_log::init_logger();

    let ctx = Arc::new(tokio::sync::RwLock::new(TestContext::default()));
    tokio::spawn({
        let arc_ctx = ctx.clone();
        async move {
            let mut proc_map = ProcManager::new();
            proc_map.insert(0, Box::new(EchoTaskConstructor::default()));
            proc_map.insert(1, Box::new(TestTaskConstructor::default()));

            match cute_network::Server::create_raw(cute_network::NetworkConfig::default()).start_server(Box::new(proc_map),arc_ctx).await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    });

    tokio::time::sleep(Duration::from_secs(1)).await;

    tokio::spawn({
        let arc_ctx = ctx.clone();
        async move {
            let mut client = cute_network::Client::<TestContext>::create_raw(cute_network::NetworkConfig {
                max_page_byte_size: 65536,
                max_channel_size: 128,
                request_limit_milli_second : 262_144,
                host_address:  std::net::SocketAddr::from(([127,0,0,1], 7777)),
                time_out: 30,
                keep_alive_time_out: 60,
            },arc_ctx).await.unwrap();

            let instant = tokio::time::Instant::now();
            match client.get_stream(0,None).await {
                Ok(mut stream) => {
                    loop {
                        if let Some(res_output) = stream.next().await {
                            if let Ok(output) = res_output {
                                let convert_result = bin_deserialize::<EchoData>(&*output);
                                info!("{:?}", convert_result);
                            }
                        }
                    }
                }
                Err(_) => {}
            }
            client.close_stream_all().await.unwrap();
            info!("client closed");
        }
    });

    tokio::spawn({
        let arc_ctx = ctx.clone();
        async move {
            let mut client = cute_network::Client::<TestContext>::create_raw(cute_network::NetworkConfig {
                max_page_byte_size: 65536,
                max_channel_size: 128,
                request_limit_milli_second : 262_144,
                host_address:  std::net::SocketAddr::from(([127,0,0,1], 7777)),
                time_out: 30,
                keep_alive_time_out: 60,
            },ctx.clone()).await.unwrap();

            let instant = tokio::time::Instant::now();
            match client.get_stream(1,None).await {
                Ok(mut stream) => {
                    while let Some(res_output) = stream.next().await {
                        if instant.elapsed().as_millis() > 30_000 {
                            break;
                        } else {
                            if let Ok(output) = res_output {
                                let convert_result = bin_deserialize::<TestData>(&*output);
                                if let Ok(data) = convert_result {
                                    info!("{:?}", data.data);
                                }
                            }
                        }
                    }
                }
                Err(_) => {}
            }
            if let Err(e) = client.close_stream_all().await {
                warn!("client closed: {:?}", e);
            }
            tokio::time::sleep(Duration::from_secs(10)).await;
            info!("client closed");
        }
    });


    loop {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }

    Ok(())
}


