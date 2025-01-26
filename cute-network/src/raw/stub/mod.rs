use std::io::Read;

use std::pin::Pin;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_stream::{Stream, StreamExt};
use cute_core::{CuteError};
use crate::raw::CutePacketTrait;

pub use server::CuteRawServiceServer;
pub use client::CuteRawServiceClient;

#[async_trait::async_trait]
pub trait CuteRawService<P> : Send + Sync + 'static
where P : CutePacketTrait + Send
{
    async fn server_unary(&self,protocol : u32, input: Box<[u8]>) -> Result<Vec<u8>, CuteError>;
    async fn server_stream(&self,protocol : u32, input: Box<[u8]>) -> Result<Pin<Box<dyn tokio_stream::Stream<Item=Result<Vec<u8>, CuteError>> + Send>>, CuteError>;
    async fn server_stream_close(&self, protocol : u32) -> Result<(), CuteError>;
    async fn server_stream_all_close(&self) -> Result<(), CuteError>;
}

mod server;
mod client;