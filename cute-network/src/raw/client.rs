use std::sync::Arc;
use cute_core::{CuteError, DataStream};
use crate::NetworkConfig;
use crate::raw::packet::CutePacket;
use crate::raw::stub::CuteRawServiceClient;

#[derive(Debug)]
pub struct RawClient<C>
where C : Send + Sync + 'static,
{
    config : NetworkConfig,
    client : CuteRawServiceClient<CutePacket>,
    context : Arc<tokio::sync::RwLock<C>>,
    protocol_name_map : std::collections::HashMap<Box<str>, u32>
}

impl<C> RawClient<C>
where C : Send + Sync + 'static, {

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

    pub async fn get_unary_data(&mut self, key: Box<str>, parameter: Option<Vec<u8>>) -> Result<Vec<u8>, CuteError> {
        todo!()

    }

    pub async fn get_stream_data(&mut self, key: Box<str>, parameter: Option<Vec<u8>>) -> Result<DataStream<Vec<u8>>, CuteError> {
        todo!()
    }

    pub async fn close_stream(&mut self, key: Box<str>) -> Result<(), CuteError> {
        todo!()
    }

    pub async fn close_stream_all(&mut self) -> Result<(), CuteError> {
        todo!()
    }
}