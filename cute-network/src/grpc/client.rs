use std::sync::Arc;
use tokio_stream::StreamExt;
use tonic::transport::Endpoint;
use cute_core::{CuteError, DataStream};
use crate::grpc::convert_status_to_cute_error;
use crate::grpc::proto::cute::cute_service_client::CuteServiceClient;
use crate::grpc::proto::cute::{Empty, Input};
use crate::NetworkConfig;

#[derive(Debug)]
pub struct GRPCClient<C>
where C : Send + Sync + 'static,
{
    config : NetworkConfig,
    client : CuteServiceClient<tonic::transport::Channel>,
    context : Arc<tokio::sync::RwLock<C>>,
}

impl<C> GRPCClient<C>
where C : Clone + Send + Sync + 'static
{
    pub async fn new(config: NetworkConfig, ctx : Arc<tokio::sync::RwLock<C>>) -> Result<Self, CuteError> {
        let url = format!("http://{}", config.host_address);
        let endpoint = Endpoint::from_shared(url)
            .map_err(|e| CuteError::internal(e.to_string()))?
            .connect_timeout(tokio::time::Duration::from_secs(config.keep_alive_time_out))
            .timeout(tokio::time::Duration::from_secs(config.time_out));

        Ok(Self {
            config,
            client: CuteServiceClient::connect(endpoint).await.map_err(|e| CuteError::internal(e.to_string()))?,
            context: ctx,
        })
    }

    pub async fn get_service_names(&mut self) {
        match self.client.get_services_name(Empty {}).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }

    pub async fn get_unary_data(&mut self, key: u32, parameter: Option<Vec<u8>>) -> Result<Vec<u8>, CuteError>
    {
        match self.client.server_unary(Input {
            protocol: key,
            data: parameter,
        }).await.map_err(|e| convert_status_to_cute_error(e)) {
            Ok(response) => {
                let mut stream = response.into_inner();
                let mut flat_vec = Vec::new();
                while let Some(output) = stream.next().await {
                    match output {
                        Ok(value) => {
                            flat_vec.extend(value.data);
                        }
                        Err(_) => {}
                    }
                }
                Ok(flat_vec)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn get_stream_data(&mut self, key: u32, parameter: Option<Vec<u8>>) -> Result<DataStream<Vec<u8>>, CuteError>
    {
        match self.client.server_stream(Input {
            protocol: key,
            data: parameter,
        }).await.map_err(|e| convert_status_to_cute_error(e)) {
            Ok(response) => {
                let mut stream = response.into_inner();
                let (tx, rx) = tokio::sync::mpsc::channel(self.config.max_channel_size);
                tokio::spawn(async move {
                    let mut flat_vec = Vec::new();
                    while let Some(output) = stream.next().await {
                        match output {
                            Ok(value) => {
                                if value.page_idx == 0 {
                                    flat_vec.clear();
                                }
                                flat_vec.extend(value.data);
                                if value.page_idx + 1 == value.page_size {
                                    match tx.try_send(Ok(flat_vec.clone())) {
                                        Ok(_) => {}
                                        Err(_) => {}
                                    }
                                    flat_vec.clear();
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    drop(tx);
                });
                Ok(Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn close_stream(&mut self, key : u32) -> Result<(), CuteError> {
        match self.client.server_stream_close(Input {
            protocol: key,
            data: None,
        }).await.map_err(|e| convert_status_to_cute_error(e)) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn close_stream_all(&mut self) -> Result<(), CuteError> {
        match self.client.server_stream_all_close(Empty {}).await.map_err(|e| convert_status_to_cute_error(e)) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}