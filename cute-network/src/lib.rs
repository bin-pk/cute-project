use std::net::SocketAddr;
use std::sync::Arc;
use cute_core::{CuteError, DataStream, Procedure};
use crate::grpc::GRPCClient;
use crate::raw::RawClient;

mod grpc;
mod raw;

#[derive(Debug, Clone, Copy)]
pub struct NetworkConfig {
    pub max_page_byte_size: usize,
    pub max_channel_size : usize,
    pub request_limit_milli_second : usize,
    pub host_address: SocketAddr,
    pub time_out : u64,
    pub keep_alive_time_out : u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            max_page_byte_size: 262_144,
            max_channel_size: 128,
            request_limit_milli_second: 0,
            host_address: SocketAddr::from(([0,0,0,0], 7777)),
            time_out: 30,
            keep_alive_time_out: 60,
        }
    }
}

pub enum Server {
    GRPC(NetworkConfig),
    Raw(NetworkConfig),
}

impl Server {
    pub fn create_grpc(config : NetworkConfig) -> Self {
        Server::GRPC(config)
    }

    pub fn create_raw(config : NetworkConfig) -> Self { Server::Raw(config) }

    pub async fn start_server<R, P, C>(&self, procedure : R, context : Arc<tokio::sync::RwLock<C>>) -> Result<(),std::io::Error>
    where R : AsRef<P> + Send + Sync + 'static,
          P : Procedure<C> + Send + Sync + 'static,
          C : Default + Clone + Send + Sync + 'static,
    {
        match self {
            Server::GRPC(config) => {
                grpc::GRPCServer::start(procedure, *config,context).await
            }
            Server::Raw(config) => {
                raw::CuteRawServer::start(procedure, *config,context).await
            }
        }
    }
}
pub enum Client<C>
where C : Default + Clone + Send + Sync + 'static,
{
    GRPC(GRPCClient<C>),
    Raw(RawClient<C>)
}

impl<C> Client<C>
where C : Default + Clone + Send + Sync + 'static
{
    pub async fn create_grpc(config : NetworkConfig, context : Arc<tokio::sync::RwLock<C>>) -> Result<Self,CuteError> {

        Ok(Client::GRPC(GRPCClient::new(config,context).await?))
    }
    pub async fn create_raw(config : NetworkConfig, context : Arc<tokio::sync::RwLock<C>> ) -> Result<Self,CuteError> {
        Ok(Client::Raw(RawClient::new(config,context).await?))
    }

    pub async fn get_service_names(&mut self) -> Result<Vec<u8>, CuteError>
    {
        match self {
            Client::GRPC(client) => {
                client.get_service_names().await;
                Ok(vec![])
            }
            Client::Raw(client) => {
                Ok(vec![])
            }
        }
    }


    //task_constructor : Box<dyn TaskConstructor<T,C>>
    pub async fn get_unary(&mut self, key : Box<str>,parameter : Option<Vec<u8>>) -> Result<Vec<u8>, CuteError>
    {
        match self {
            Client::GRPC(client) => {
                client.get_unary_data(key,parameter).await
            }
            Client::Raw(client) => {
                client.get_unary_data(key,parameter).await
            }
        }
    }

    pub async fn get_stream(&mut self, key : Box<str>,parameter : Option<Vec<u8>>) -> Result<DataStream<Vec<u8>>, CuteError>
    {
        match self {
            Client::GRPC(client) => {
                client.get_stream_data(key,parameter).await
            }
            Client::Raw(client) => {
                client.get_stream_data(key,parameter).await
            }
        }
    }

    pub async fn close_stream(&mut self, key : Box<str>) -> Result<(),CuteError> {
        match self {
            Client::GRPC(client) => {
                client.close_stream(key).await
            }
            Client::Raw(client) => {
                client.close_stream(key).await
            }
        }
    }

    pub async fn close_stream_all(&mut self) -> Result<(),CuteError> {
        match self {
            Client::GRPC(client) => {
                client.close_stream_all().await
            }
            Client::Raw(client) => {
                client.close_stream_all().await
            }
        }
    }
}