use std::net::SocketAddr;
use cute_core::{CuteError, DataStream, Procedure};
use crate::grpc::GRPCClient;

mod grpc;

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
    GRPC(NetworkConfig)
}

impl Server {
    pub fn create_grpc(config : NetworkConfig) -> Self {
        Server::GRPC(config)
    }

    pub async fn start_server<R, P, C>(&self, procedure : R) -> Result<(),std::io::Error>
    where R : AsRef<P> + Send + Sync + 'static,
          P : Procedure<C> + Send + Sync + 'static,
          C : Default + Clone + Send + Sync + 'static,
    {
        match self {
            Server::GRPC(config) => {
                grpc::GRPCServer::start(procedure, *config).await
            }
        }
    }
}

pub enum Client<C>
where C : Default + Clone + Send + Sync + 'static,
{
    GRPC(GRPCClient<C>)
}

impl<C> Client<C>
where C : Default + Clone + Send + Sync + 'static
{
    pub async fn create_grpc(config : NetworkConfig) -> Result<Self,CuteError> {

        Ok(Client::GRPC(GRPCClient::new(config).await?))
    }

    //task_constructor : Box<dyn TaskConstructor<T,C>>
    pub async fn get_unary(&mut self, key : Box<str>,parameter : Option<Vec<u8>>) -> Result<Vec<u8>, CuteError>
    {
        match self {
            Client::GRPC(client) => {
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
        }
    }

    pub async fn close_stream(&mut self, key : Box<str>) -> Result<(),CuteError> {
        match self {
            Client::GRPC(client) => {
                client.close_stream(key).await
            }
        }
    }

    pub async fn close_stream_all(&mut self) -> Result<(),CuteError> {
        match self {
            Client::GRPC(client) => {
                client.close_stream_all().await
            }
        }
    }
}