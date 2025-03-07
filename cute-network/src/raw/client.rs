use std::sync::Arc;
use async_stream::stream;
use tokio_stream::StreamExt;
use cute_core::{CuteError, DataStream};
use crate::NetworkConfig;
use crate::raw::CutePacketTrait;
use crate::raw::stub::CuteRawServiceClient;

#[derive(Debug)]
pub struct RawClient<C,P>
where C : Send + Sync + 'static,
    P : CutePacketTrait + Send
{
    config : NetworkConfig,
    client : CuteRawServiceClient<P>,
    context : Arc<tokio::sync::RwLock<C>>,
    protocol_name_map : std::collections::HashMap<Box<str>, u32>
}

impl<C,P> RawClient<C,P>
where C : Clone + Send + Sync + 'static,
      P : CutePacketTrait + Send
{
    pub async fn new(config : NetworkConfig, context : Arc<tokio::sync::RwLock<C>>) -> Result<Self,CuteError> {
        let client = CuteRawServiceClient::connect(config.host_address).await?;
        let protocol_name_map = std::collections::HashMap::new();

        Ok(Self {
            config,
            client,
            context,
            protocol_name_map,
        })
    }

    pub async fn get_unary_data(&mut self, key: u32, parameter: Option<Vec<u8>>) -> Result<Vec<u8>, CuteError> {
        match self.client.client_unary(key,parameter).await {
            Ok(res) => {
                Ok(res.get_payload())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn get_stream_data(&mut self, key: u32, parameter: Option<Vec<u8>>) -> Result<DataStream<Vec<u8>>, CuteError> {

        match self.client.client_stream(key,parameter).await {
            Ok(mut res_stream) => {
                Ok(Box::pin(stream! {
                    let mut flat_vec = Vec::new();
                    while let Some(packet) = res_stream.next().await {
                        match packet {
                            Ok(value) => {
                                if value.get_chuck_idx() == 0 {
                                    flat_vec.clear()
                                }
                                flat_vec.extend_from_slice(&value.get_payload());
                                if value.get_chuck_idx() + 1 == value.get_chuck_size() {
                                    yield Ok(flat_vec.clone())
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }))
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    pub async fn close_stream(&mut self, key: u32) -> Result<(), CuteError> {
        self.client.close_stream(key).await
    }

    pub async fn close_stream_all(&mut self) -> Result<(), CuteError> {
        self.client.close_stream_all().await
    }
}